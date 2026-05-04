use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{FunctionValue, PointerValue};

use newcp_ir::{IrGlobal, IrModule, IrProcedure};

use crate::error::CodegenError;
use crate::options::CodegenOptions;
use crate::types::TypeLowerer;

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
    /// Interned SHORTCHAR string constant globals, keyed by string content.
    /// Each entry is a `ptr` to the first byte of a null-terminated `[N x i8]` constant.
    pub string_constants: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> GlobalPlanner<'ctx> {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            globals: HashMap::new(),
            module_data_ty: None,
            string_constants: HashMap::new(),
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
}

impl<'ctx> CodegenModule<'ctx> {
    /// Stage 2: create the LLVM context, module, and builder.
    pub fn new(
        context: &'ctx Context,
        ir_module: &IrModule,
        _options: &CodegenOptions,
    ) -> Result<Self, CodegenError> {
        let module = context.create_module(&ir_module.name);
        let builder = context.create_builder();
        let lowerer = TypeLowerer::new(context);
        Ok(Self {
            context,
            module,
            builder,
            planner: GlobalPlanner::new(),
            lowerer,
        })
    }

    /// Stage 3: declare all procedures and globals in the LLVM module so that
    /// forward references work during body emission.
    pub fn plan(
        &mut self,
        ir_module: &IrModule,
        options: &CodegenOptions,
    ) -> Result<(), CodegenError> {
        self.declare_globals(ir_module, options)?;
        // Declare all procedures.
        for proc in &ir_module.procedures {
            self.declare_procedure(proc, options)?;
        }
        // Declare `@__newcp_sys_new(i64) -> ptr` if needed.
        if uses_sys_new(ir_module) {
            self.declare_sys_new();
        }
        Ok(())
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
            let llvm_ty = match self.lowerer.lower_basic(&global.ty) {
                Ok(ty) => ty,
                Err(err) => {
                    if options.strict_unsupported {
                        return Err(err);
                    }
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
        proc: &IrProcedure,
        options: &CodegenOptions,
    ) -> Result<(), CodegenError> {
        // Build parameter type list.
        let mut param_types = Vec::new();
        for (_name, ty) in &proc.params {
            match self.lowerer.lower_basic(ty) {
                Ok(t) => param_types.push(t.into()),
                Err(e) => {
                    if options.strict_unsupported {
                        return Err(e);
                    }
                    // Degrade unsupported param to ptr; codegen will trap at call sites.
                    param_types
                        .push(self.context.ptr_type(inkwell::AddressSpace::default()).into());
                }
            }
        }

        // Build return type.
        let fn_type = match self.lowerer.lower_return_type(&proc.ret_ty)? {
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

        let fn_val = self.module.add_function(&proc.name, fn_type, None);
        self.planner.functions.insert(proc.name.clone(), fn_val);
        Ok(())
    }

    /// Get or emit a private `[N x i8]` string constant global for `s`.
    ///
    /// Returns a `ptr` pointing at element 0 (the first byte of the null-terminated string).
    /// Identical string contents share a single global — the cache is keyed by string value.
    pub fn get_or_emit_string_constant(&mut self, s: &str) -> PointerValue<'ctx> {
        if let Some(&ptr) = self.planner.string_constants.get(s) {
            return ptr;
        }

        // Build a null-terminated byte sequence.
        let bytes: Vec<u8> = s.bytes().chain(std::iter::once(0u8)).collect();
        let i8_ty = self.context.i8_type();
        let array_ty = i8_ty.array_type(bytes.len() as u32);

        // Build the LLVM [N x i8] constant.
        let byte_vals: Vec<_> = bytes
            .iter()
            .map(|&b| i8_ty.const_int(b as u64, false))
            .collect();
        let initializer = i8_ty.const_array(&byte_vals);

        // Generate a stable, unique name.
        let idx = self.planner.string_constants.len();
        let global_name = format!(".str.{idx}");
        let global = self.module.add_global(array_ty, None, &global_name);
        global.set_initializer(&initializer);
        global.set_constant(true);
        global.set_linkage(inkwell::module::Linkage::Private);

        // GEP to element 0 to produce a `ptr`.
        let ptr_ty = self.context.ptr_type(inkwell::AddressSpace::default());
        let zero = self.context.i32_type().const_zero();
        let ptr = unsafe {
            global.as_pointer_value().const_in_bounds_gep(array_ty, &[zero, zero])
        };
        // The GEP result is an i8* in modern LLVM; cast to opaque ptr.
        let _ = ptr_ty;
        self.planner.string_constants.insert(s.to_string(), ptr);
        ptr
    }

    /// Declare `@__newcp_sys_new(i64) -> ptr` — part of the backend/runtime ABI.
    fn declare_sys_new(&mut self) {
        let i64_ty = self.context.i64_type();
        let ptr_ty = self.context.ptr_type(inkwell::AddressSpace::default());
        let fn_ty = ptr_ty.fn_type(&[i64_ty.into()], false);
        self.module
            .add_function("__newcp_sys_new", fn_ty, Some(inkwell::module::Linkage::External));
    }

    /// Return the LLVM textual IR for the completed module.
    pub fn print_to_string(&self) -> String {
        self.module.print_to_string().to_string()
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
                if matches!(instr, Instr::SysNew { .. }) {
                    return true;
                }
            }
        }
    }
    false
}
