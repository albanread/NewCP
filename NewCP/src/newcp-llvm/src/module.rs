use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{FunctionValue, GlobalValue, PointerValue};

use newcp_ir::{IrGlobal, IrModule, IrProcedure};

use crate::error::CodegenError;
use crate::options::CodegenOptions;
use crate::types::TypeLowerer;

/// Holds declared LLVM function values, keyed by procedure name.
pub struct GlobalPlanner<'ctx> {
    /// LLVM function value for each procedure in the module, by name.
    pub functions: HashMap<String, FunctionValue<'ctx>>,
    /// LLVM global storage for module-level variables.
    pub globals: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> GlobalPlanner<'ctx> {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            globals: HashMap::new(),
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
        for global in &ir_module.globals {
            if global.is_const {
                continue;
            }
            self.declare_global(global, options)?;
        }
        Ok(())
    }

    fn declare_global(
        &mut self,
        global: &IrGlobal,
        options: &CodegenOptions,
    ) -> Result<(), CodegenError> {
        let llvm_ty = match self.lowerer.lower_basic(&global.ty) {
            Ok(ty) => ty,
            Err(err) => {
                if options.strict_unsupported {
                    return Err(err);
                }
                self.context.ptr_type(inkwell::AddressSpace::default()).into()
            }
        };
        let global_value = self.module.add_global(llvm_ty, None, &global.name);
        initialize_global_to_zero(&global_value, llvm_ty);
        self.planner
            .globals
            .insert(global.name.clone(), global_value.as_pointer_value());
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

fn initialize_global_to_zero(global: &GlobalValue<'_>, ty: BasicTypeEnum<'_>) {
    match ty {
        BasicTypeEnum::ArrayType(t) => global.set_initializer(&t.const_zero()),
        BasicTypeEnum::FloatType(t) => global.set_initializer(&t.const_float(0.0)),
        BasicTypeEnum::IntType(t) => global.set_initializer(&t.const_zero()),
        BasicTypeEnum::PointerType(t) => global.set_initializer(&t.const_null()),
        BasicTypeEnum::StructType(t) => global.set_initializer(&t.const_zero()),
        BasicTypeEnum::VectorType(t) => global.set_initializer(&t.const_zero()),
        BasicTypeEnum::ScalableVectorType(t) => global.set_initializer(&t.const_zero()),
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
