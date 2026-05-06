use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use std::collections::HashMap;

use crate::error::CodegenError;
use crate::options::OptLevel;

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
        opt_level: OptLevel,
        global_mappings: Vec<(FunctionValue<'ctx>, usize)>,
    ) -> Result<Self, CodegenError> {
        let engine = module
            .create_jit_execution_engine(opt_level.to_llvm())
            .map_err(|e| CodegenError::Jit(e.to_string()))?;
        for (function, address) in global_mappings {
            engine.add_global_mapping(&function, address);
        }
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
        let addr = self.lookup_export_address_by_llvm_name(llvm_name).map_err(|e| {
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

    pub fn export_address(&self, public_name: &str) -> Result<usize, CodegenError> {
        let llvm_name = self
            .exported_functions
            .iter()
            .find(|ef| ef.public_name == public_name)
            .map(|ef| ef.llvm_name.as_str())
            .ok_or_else(|| CodegenError::Jit(format!("no exported function '{public_name}'")))?;

        self.lookup_export_address_by_llvm_name(llvm_name)
            .map_err(|e| CodegenError::Jit(format!(
                "symbol lookup failed for '{llvm_name}' (public: '{public_name}'): {e}"
            )))
    }

    fn lookup_export_address_by_llvm_name(&self, llvm_name: &str) -> Result<usize, String> {
        let addr = self
            .engine
            .get_function_address(llvm_name)
            .map_err(|e| e.to_string())?;
        if addr == 0 {
            return Err(format!("function '{llvm_name}' resolved to null address"));
        }
        Ok(addr as usize)
    }
}

pub(crate) fn collect_global_mappings<'ctx>(
    module: &Module<'ctx>,
    extra_symbol_mappings: &HashMap<String, usize>,
) -> Result<Vec<(FunctionValue<'ctx>, usize)>, CodegenError> {
    let mut mappings = Vec::new();
    let debug = std::env::var("NEWCP_JIT_DEBUG").is_ok();

    for function in module.get_functions() {
        if function.get_linkage() != Linkage::External || function.count_basic_blocks() != 0 {
            continue;
        }

        let symbol_name = function
            .get_name()
            .to_str()
            .map_err(|_| CodegenError::Jit("encountered non-UTF8 symbol name".to_string()))?;

        if let Some(address) = extra_symbol_mappings.get(symbol_name).copied() {
            if debug { eprintln!("[jit] map (extra) {symbol_name} -> 0x{address:x}"); }
            mappings.push((function, address));
            continue;
        }

        if let Some(address) = newcp_runtime::runtime_symbol_address(symbol_name) {
            if debug { eprintln!("[jit] map (runtime) {symbol_name} -> 0x{address:x}"); }
            mappings.push((function, address));
            continue;
        }

        if debug { eprintln!("[jit] UNRESOLVED external symbol: {symbol_name}"); }
    }

    Ok(mappings)
}
