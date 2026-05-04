use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::OptimizationLevel;

use crate::error::CodegenError;

/// An executable module that has been materialized through the LLVM JIT.
///
/// Owns the `ExecutionEngine` (which keeps the native code alive) and
/// the exported symbol manifest.
pub struct JitModule<'ctx> {
    engine: ExecutionEngine<'ctx>,
    pub exported_functions: Vec<crate::ExportedFunction>,
}

impl<'ctx> JitModule<'ctx> {
    /// Stage 6: materialize a verified LLVM module into native code.
    pub fn from_module(
        module: Module<'ctx>,
        exported_functions: Vec<crate::ExportedFunction>,
    ) -> Result<Self, CodegenError> {
        let engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| CodegenError::Jit(e.to_string()))?;
        Ok(Self { engine, exported_functions })
    }

    /// Look up a compiled exported function by its public name.
    ///
    /// # Safety
    ///
    /// The caller must ensure `F` is the correct function pointer type
    /// corresponding to the procedure's parameter and return types.
    pub unsafe fn get_function<F>(&self, public_name: &str) -> Result<F, CodegenError>
    where
        F: Copy,
    {
        // Find the LLVM mangled name for this public name.
        let llvm_name = self
            .exported_functions
            .iter()
            .find(|ef| ef.public_name == public_name)
            .map(|ef| ef.llvm_name.as_str())
            .ok_or_else(|| CodegenError::Jit(format!("no exported function '{public_name}'")))?;

        // SAFETY: delegated to caller to provide the correct type parameter.
        let addr = self.engine.get_function_address(llvm_name).map_err(|e| {
            CodegenError::Jit(format!(
                "symbol lookup failed for '{llvm_name}' (public: '{public_name}'): {e}"
            ))
        })?;

        if addr == 0 {
            return Err(CodegenError::Jit(format!(
                "function '{llvm_name}' resolved to null address"
            )));
        }

        // SAFETY: addr is non-null and caller asserts correct type.
        Ok(unsafe { std::mem::transmute_copy(&addr) })
    }
}
