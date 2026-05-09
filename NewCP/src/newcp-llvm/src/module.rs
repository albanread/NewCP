use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::TargetMachine;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{FunctionValue, GlobalValue, PointerValue};

use newcp_ir::{IrGlobal, IrModule, IrProcedure, IrType};

use crate::error::CodegenError;
use crate::options::{CodegenOptions, OptLevel};
use crate::types::TypeLowerer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendDiagnostic {
    pub message: String,
}

impl BackendDiagnostic {
    pub fn render(&self) -> String {
        self.message.clone()
    }
}

/// Holds declared LLVM function values, keyed by procedure name.
pub struct GlobalPlanner<'ctx> {
    /// LLVM function value for each procedure in the module, by name.
    pub functions: HashMap<String, FunctionValue<'ctx>>,
    /// GEP-derived `ptr` for each module-level variable, keyed by IR name.
    /// Each pointer addresses the corresponding field inside `@ModuleName.Data`.
    pub globals: HashMap<String, PointerValue<'ctx>>,
    /// The LLVM struct type used for `@ModuleName.Data`, or `None` if the
    /// module has no mutable globals.
    pub module_data_ty: Option<StructType<'ctx>>,
    /// LLVM struct types for named record types, keyed by type name.
    /// Used by `emit_gep` to emit typed `getelementptr` instructions.
    pub named_struct_types: HashMap<String, StructType<'ctx>>,
    /// Interned SHORTCHAR string constant globals, keyed by string content.
    /// Each entry is a `ptr` to the first byte of a null-terminated `[N x i8]` constant.
    pub string_constants: HashMap<String, PointerValue<'ctx>>,
    /// `@TypeName.desc` global constants emitted for each record type that has methods.
    /// Used by `emit_method_call` to locate the static TypeDesc for a type.
    pub type_desc_globals: HashMap<String, GlobalValue<'ctx>>,
    /// For each `@TypeName.vtable` global, the ordered list of LLVM function
    /// names occupying its slots. Recorded so the JIT init step (in jit.rs)
    /// can patch the vtable contents with the actual function addresses
    /// after MCJIT has compiled them — MCJIT does not reliably apply
    /// function-pointer relocations to constant globals on its own.
    pub vtable_slot_functions: HashMap<String, Vec<String>>,
}

impl<'ctx> GlobalPlanner<'ctx> {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            globals: HashMap::new(),
            module_data_ty: None,
            named_struct_types: HashMap::new(),
            string_constants: HashMap::new(),
            type_desc_globals: HashMap::new(),
            vtable_slot_functions: HashMap::new(),
        }
    }
}

/// Central coordinating object for one compilation job.
///
/// Owns the Inkwell `Context`, `Module`, `Builder`, and `GlobalPlanner`.
/// Constructed at the start of Stage 2 and consumed to produce a `CompiledModule`.
pub struct CodegenModule<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub planner: GlobalPlanner<'ctx>,
    pub lowerer: TypeLowerer<'ctx>,
    pub diagnostics: Vec<BackendDiagnostic>,
}

impl<'ctx> CodegenModule<'ctx> {
    /// Stage 2: create the LLVM context, module, and builder.
    ///
    /// The `machine` is used to stamp the module with the correct target triple
    /// and data layout immediately after creation. Without this the new-style
    /// `run_passes` optimisation pipeline has no target metadata and silently
    /// skips all target-dependent transforms, producing identical output at
    /// every optimisation level.
    pub fn new(
        context: &'ctx Context,
        ir_module: &IrModule,
        _options: &CodegenOptions,
        machine: &TargetMachine,
    ) -> Result<Self, CodegenError> {
        let module = context.create_module(&ir_module.name);
        // Stamp triple and data layout so the optimisation pipeline can reason
        // about pointer sizes, alignment, and target-specific transforms.
        module.set_triple(&machine.get_triple());
        module.set_data_layout(&machine.get_target_data().get_data_layout());
        let builder = context.create_builder();
        let lowerer = TypeLowerer::new(context);
        Ok(Self {
            context,
            module,
            builder,
            planner: GlobalPlanner::new(),
            lowerer,
            diagnostics: Vec::new(),
        })
    }

    fn record_diagnostic(&mut self, message: impl Into<String>) {
        self.diagnostics.push(BackendDiagnostic {
            message: message.into(),
        });
    }

    /// Stage 3: declare all procedures and globals in the LLVM module so that
    /// forward references work during body emission.
    pub fn plan(
        &mut self,
        ir_module: &IrModule,
        options: &CodegenOptions,
    ) -> Result<(), CodegenError> {
        // Declare LLVM struct types for named records first so that
        // declare_globals can use them when building @Module.Data.
        self.declare_named_types(ir_module, options);
        self.declare_globals(ir_module, options)?;
        for proc in &ir_module.procedures {
            self.declare_procedure(ir_module, proc, options)?;
        }
        // Declare `@__newcp_sys_new(i64) -> ptr` if needed.
        if uses_sys_new(ir_module) {
            self.declare_sys_new();
        }        // Emit TypeDesc constant globals and vtable arrays for record types with methods.
        self.declare_type_descs(ir_module);        Ok(())
    }

    /// Declare LLVM struct types for all named record types in the module.
    ///
    /// After this call, `planner.named_struct_types` maps each type name to an
    /// LLVM `StructType` whose fields are laid out in the same order as
    /// `ir_module.named_types[name]`.
    fn declare_named_types(&mut self, ir_module: &IrModule, _options: &CodegenOptions) {
        let lowerer = TypeLowerer::new(self.context);
        for (type_name, fields) in &ir_module.named_types {
            let field_types: Vec<BasicTypeEnum<'ctx>> = fields
                .iter()
                .filter_map(|(_, ir_ty)| lowerer.lower_basic(ir_ty, None).ok())
                .collect();
            if field_types.len() != fields.len() {
                // Skip types with unsupported fields for now.
                self.record_diagnostic(format!(
                    "skipping named type '{type_name}' because one or more fields could not be lowered to LLVM"
                ));
                continue;
            }
            let struct_ty = self.context.opaque_struct_type(type_name.as_str());
            struct_ty.set_body(field_types.as_slice(), false);
            self.planner.named_struct_types.insert(type_name.clone(), struct_ty);
        }
    }

    fn declare_globals(
        &mut self,
        ir_module: &IrModule,
        options: &CodegenOptions,
    ) -> Result<(), CodegenError> {
        // Collect the mutable globals and their LLVM types.
        let mut fields: Vec<(&IrGlobal, BasicTypeEnum<'ctx>)> = Vec::new();
        for global in &ir_module.globals {
            if global.is_const {
                continue;
            }
            // Use a local lowerer so we can pass named_struct_types (already populated
            // by declare_named_types which runs before declare_globals).
            let lowerer = TypeLowerer::new(self.context);
            let llvm_ty = match lowerer.lower_basic(&global.ty, Some(&self.planner.named_struct_types)) {
                Ok(ty) => ty,
                Err(err) => {
                    if options.strict_unsupported {
                        return Err(err);
                    }
                    self.record_diagnostic(format!(
                        "global '{}' lowered to opaque ptr fallback: {}",
                        global.name, err
                    ));
                    self.context.ptr_type(inkwell::AddressSpace::default()).into()
                }
            };
            fields.push((global, llvm_ty));
        }

        if fields.is_empty() {
            return Ok(());
        }

        // Build a packed struct type `%ModuleName.Data`.
        let field_types: Vec<BasicTypeEnum<'ctx>> = fields.iter().map(|(_, ty)| *ty).collect();
        let struct_name = format!("{}.Data", ir_module.name);
        let struct_ty = self.context.opaque_struct_type(&struct_name);
        struct_ty.set_body(&field_types, /*packed=*/ false);
        self.planner.module_data_ty = Some(struct_ty);

        // Emit the single `@ModuleName.Data` global initialised to zero.
        let global_val = self
            .module
            .add_global(struct_ty, None, &struct_name);
        global_val.set_initializer(&struct_ty.const_zero());

        // Pre-compute one GEP per field and populate `planner.globals`.
        let i32_ty = self.context.i32_type();
        let base_ptr = global_val.as_pointer_value();
        for (idx, (global, _)) in fields.iter().enumerate() {
            let field_ptr = unsafe {
                base_ptr.const_in_bounds_gep(
                    struct_ty,
                    &[
                        i32_ty.const_zero(),
                        i32_ty.const_int(idx as u64, false),
                    ],
                )
            };
            self.planner.globals.insert(global.name.clone(), field_ptr);
        }

        Ok(())
    }

    fn declare_procedure(
        &mut self,
        ir_module: &IrModule,
        proc: &IrProcedure,
        options: &CodegenOptions,
    ) -> Result<(), CodegenError> {
        // Capture named struct types to pass to the type lowerer.
        let named_types = self.planner.named_struct_types.clone();
        let lowerer = TypeLowerer::new(self.context);

        // Build parameter type list.
        let mut param_types = Vec::new();
        for (_name, ty) in &proc.params {
            match lowerer.lower_basic(ty, Some(&named_types)) {
                Ok(t) => param_types.push(t.into()),
                Err(e) => {
                    if options.strict_unsupported {
                        return Err(e);
                    }
                    // Degrade unsupported param to ptr; codegen will trap at call sites.
                    self.record_diagnostic(format!(
                        "procedure '{}' parameter '{}' lowered to ptr fallback: {}",
                        proc.name,
                        ty.render(),
                        e
                    ));
                    param_types
                        .push(self.context.ptr_type(inkwell::AddressSpace::default()).into());
                }
            }
        }

        // Build return type.
        let fn_type = match lowerer.lower_return_type(&proc.ret_ty, Some(&named_types))? {
            inkwell::types::AnyTypeEnum::VoidType(v) => v.fn_type(&param_types, false),
            inkwell::types::AnyTypeEnum::IntType(i) => i.fn_type(&param_types, false),
            inkwell::types::AnyTypeEnum::FloatType(f) => f.fn_type(&param_types, false),
            inkwell::types::AnyTypeEnum::PointerType(p) => p.fn_type(&param_types, false),
            other => {
                return Err(CodegenError::Unsupported {
                    stage: "procedure_declaration",
                    detail: format!(
                        "return type '{}' produces unsupported LLVM return type {:?}",
                        proc.ret_ty.render(),
                        other
                    ),
                });
            }
        };

        let llvm_name = if proc.exported {
            options.exported_symbol_name(&ir_module.name, &proc.name)
        } else {
            proc.name.clone()
        };

        let fn_val = self.module.add_function(&llvm_name, fn_type, None);
        self.planner.functions.insert(proc.name.clone(), fn_val);
        Ok(())
    }

    /// Get or emit a private string constant global for `s` whose element type
    /// is `elem_ty` (`IrType::Char` → `[N x i32]` UTF-32; `IrType::ShortChar` →
    /// `[N x i8]` Latin-1). Identical (text, element-type) pairs share one global.
    pub fn get_or_emit_string_constant(&mut self, s: &str, elem_ty: &IrType) -> PointerValue<'ctx> {
        let is_short = matches!(elem_ty, IrType::ShortChar | IrType::I8 | IrType::U8);
        let cache_key = if is_short { format!("s:{s}") } else { format!("c:{s}") };
        if let Some(&ptr) = self.planner.string_constants.get(&cache_key) {
            return ptr;
        }

        let (array_ty, initializer) = if is_short {
            // Null-terminated SHORTCHAR (Latin-1) byte sequence.
            let i8_ty = self.context.i8_type();
            let bytes: Vec<u8> = s.chars()
                .map(|c| if (c as u32) <= 0xFF { c as u8 } else { b'?' })
                .chain(std::iter::once(0u8))
                .collect();
            let arr_ty = i8_ty.array_type(bytes.len() as u32);
            let vals: Vec<_> = bytes.iter().map(|&b| i8_ty.const_int(b as u64, false)).collect();
            (arr_ty, i8_ty.const_array(&vals))
        } else {
            // Null-terminated UTF-32 code-point sequence.
            let i32_ty = self.context.i32_type();
            let codepoints: Vec<u32> = s.chars().map(|c| c as u32).chain(std::iter::once(0u32)).collect();
            let arr_ty = i32_ty.array_type(codepoints.len() as u32);
            let vals: Vec<_> = codepoints.iter().map(|&cp| i32_ty.const_int(cp as u64, false)).collect();
            (arr_ty, i32_ty.const_array(&vals))
        };

        let idx = self.planner.string_constants.len();
        let global_name = format!(".str.{idx}");
        let global = self.module.add_global(array_ty, None, &global_name);
        global.set_initializer(&initializer);
        global.set_constant(true);
        global.set_linkage(inkwell::module::Linkage::Private);

        // GEP to element 0 to produce a `ptr`.
        let zero = self.context.i32_type().const_zero();
        let ptr = unsafe {
            global.as_pointer_value().const_in_bounds_gep(array_ty, &[zero, zero])
        };
        self.planner.string_constants.insert(cache_key, ptr);
        ptr
    }

    /// Emit `@TypeName.vtable` and `@TypeName.desc` constant globals for every
    /// record type in `ir_module.type_vtables`.
    ///
    /// # TypeDesc layout (matches `newcp_runtime::gc::TypeDesc` `#[repr(C)]`):
    /// ```text
    /// { i64 size, ptr module, ptr finalizer, ptr base, ptr vtable, i64 vtable_len, [1 x i64] ptroffs }
    /// ```
    /// `size` is computed from the LLVM struct type.  `module`, `finalizer`, and
    /// `base` are null for the first slice (runtime registration happens later).
    fn declare_type_descs(&mut self, ir_module: &IrModule) {
        let i64_ty  = self.context.i64_type();
        let ptr_ty  = self.context.ptr_type(inkwell::AddressSpace::default());

        // %TypeDesc = type { i64, ptr, ptr, ptr, ptr, i64, [1 x i64] }
        // (ptroffs has exactly 1 element — the sentinel -1 — for the first slice,
        // since we don't yet have pointer-field tracking.  Types with pointer fields
        // will need more entries; this is sufficient for non-pointer record types.)
        let ptroffs_arr_ty = i64_ty.array_type(1);
        let type_desc_ty = self.context.struct_type(
            &[
                i64_ty.into(),          // 0: size
                ptr_ty.into(),          // 1: module
                ptr_ty.into(),          // 2: finalizer
                ptr_ty.into(),          // 3: base
                ptr_ty.into(),          // 4: vtable
                i64_ty.into(),          // 5: vtable_len
                ptroffs_arr_ty.into(),  // 6: ptroffs[1] (sentinel -1 only)
            ],
            false,
        );

        // First pass: emit vtable arrays so we have their global addresses for TypeDesc.
        let mut vtable_globals: HashMap<String, GlobalValue<'ctx>> = HashMap::new();
        for (type_name, slot_fns) in &ir_module.type_vtables {
            if slot_fns.is_empty() {
                continue;
            }
            let vtable_ty = ptr_ty.array_type(slot_fns.len() as u32);
            let vtable_name = format!("{}.vtable", type_name);

            // Build the constant initializer with function-pointer references.
            let fn_ptrs: Vec<_> = slot_fns
                .iter()
                .map(|fn_name| {
                    self.module
                        .get_function(fn_name)
                        .map(|f| f.as_global_value().as_pointer_value().into())
                        .unwrap_or_else(|| ptr_ty.const_null().into())
                })
                .collect();
            let vtable_init = ptr_ty.const_array(&fn_ptrs);
            let vtable_global = self.module.add_global(vtable_ty, None, &vtable_name);
            vtable_global.set_initializer(&vtable_init);
            vtable_global.set_constant(true);
            vtable_global.set_linkage(Linkage::Internal);
            vtable_globals.insert(type_name.clone(), vtable_global);
            self.planner
                .vtable_slot_functions
                .insert(vtable_name.clone(), slot_fns.clone());
        }

        // Second pass: emit TypeDesc constants.
        // Sort in topological order (base types before derived) so that when we emit
        // Circle.desc, Shape.desc already exists for the base pointer lookup.
        let mut ordered: Vec<&String> = ir_module.type_vtables.keys().collect();
        ordered.sort_by_key(|name| {
            let mut depth = 0usize;
            let mut current = name.as_str();
            while let Some(Some(base)) = ir_module.type_bases.get(current).map(|b| b.as_deref()) {
                depth += 1;
                current = base;
                if depth > 128 { break; }
            }
            depth
        });
        for type_name in ordered {
            let slot_fns = &ir_module.type_vtables[type_name];
            let desc_name = format!("{}.desc", type_name);

            // Compute payload size via LLVM struct size_of.
            let size_val = self
                .planner
                .named_struct_types
                .get(type_name.as_str())
                .and_then(|st| st.size_of())
                .unwrap_or_else(|| i64_ty.const_int(0, false));

            // Base TypeDesc pointer (null for types with no local base in first slice).
            let base_ptr = ir_module
                .type_bases
                .get(type_name.as_str())
                .and_then(|base_opt| base_opt.as_deref())
                .and_then(|base_name| {
                    // The base desc global may not exist yet if the base has no methods,
                    // or may have already been emitted in this pass.
                    self.module.get_global(&format!("{base_name}.desc")).map(|g| g.as_pointer_value().into())
                })
                .unwrap_or_else(|| ptr_ty.const_null().into());

            // Vtable pointer (null if no slots).
            let (vtable_ptr, vtable_len) = if slot_fns.is_empty() {
                (ptr_ty.const_null(), 0u64)
            } else {
                let vtg = vtable_globals[type_name.as_str()].as_pointer_value();
                (vtg, slot_fns.len() as u64)
            };

            // ptroffs: just the sentinel -1 (no pointer fields tracked yet).
            let ptroffs_init = i64_ty.const_array(&[i64_ty.const_int(u64::MAX, true)]);

            let desc_init = type_desc_ty.const_named_struct(&[
                size_val.into(),
                ptr_ty.const_null().into(),                          // module: null
                ptr_ty.const_null().into(),                          // finalizer: null
                base_ptr,                                            // base
                vtable_ptr.into(),                                   // vtable
                i64_ty.const_int(vtable_len, false).into(),          // vtable_len
                ptroffs_init.into(),                                 // ptroffs
            ]);

            let desc_global = self.module.add_global(type_desc_ty, None, &desc_name);
            desc_global.set_initializer(&desc_init);
            desc_global.set_constant(true);
            desc_global.set_linkage(Linkage::Internal);
            self.planner.type_desc_globals.insert(type_name.clone(), desc_global);
        }
    }

    /// Declare `@__newcp_sys_new(i64) -> ptr` — part of the backend/runtime ABI.
    fn declare_sys_new(&mut self) {
        let ptr_ty = self.context.ptr_type(inkwell::AddressSpace::default());
        let i64_ty2 = self.context.i64_type();
        let fn_ty = ptr_ty.fn_type(&[i64_ty2.into()], false);
        self.module
            .add_function("__newcp_sys_new", fn_ty, Some(inkwell::module::Linkage::External));

        // `@__newcp_new_rec(ptr) -> ptr` — tagged-record allocator. Takes a
        // `*const TypeDesc`, allocates `header_size + typedesc.size` bytes,
        // initializes the BlockHeader with the TypeDesc tag, and returns the
        // payload pointer. Required for `NEW(ptr)` so method dispatch can
        // recover the TypeDesc from `obj_ptr - 16` at runtime.
        let fn_ty_rec = ptr_ty.fn_type(&[ptr_ty.into()], false);
        self.module
            .add_function("__newcp_new_rec", fn_ty_rec, Some(inkwell::module::Linkage::External));
    }

    /// Return the LLVM textual IR for the completed module.
    pub fn print_to_string(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Run the configured optimization pipeline.
    pub fn optimize(&self, opt_level: OptLevel, machine: &TargetMachine) -> Result<(), CodegenError> {
        if opt_level == OptLevel::None {
            return Ok(());
        }

        let pipeline = match opt_level {
            OptLevel::None => return Ok(()),
            OptLevel::Less => "default<O1>",
            OptLevel::Default => "default<O2>",
            OptLevel::Aggressive => "default<O3>",
        };
        let pass_options = PassBuilderOptions::create();
        pass_options.set_verify_each(false);

        self.module
            .run_passes(pipeline, machine, pass_options)
            .map_err(|e| CodegenError::Verify(format!("optimization pipeline '{pipeline}' failed: {e}")))
    }

    /// Verify the module; return the error string on failure.
    pub fn verify(&self) -> Result<(), CodegenError> {
        self.module.verify().map_err(|e| CodegenError::Verify(e.to_string()))
    }

    /// Consume this `CodegenModule` and hand the `Module` to the JIT stage.
    pub fn into_module(self) -> Module<'ctx> {
        self.module
    }
}

/// True if any procedure in the module uses `Instr::SysNew`.
fn uses_sys_new(ir_module: &IrModule) -> bool {
    use newcp_ir::Instr;
    for proc in &ir_module.procedures {
        for block in &proc.blocks {
            for instr in &block.instrs {
                if matches!(instr, Instr::SysNew { .. } | Instr::New { .. }) {
                    return true;
                }
            }
        }
    }
    false
}
