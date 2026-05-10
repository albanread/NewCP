use inkwell::execution_engine::ExecutionEngine;
use inkwell::llvm_sys::execution_engine::{LLVMGetGlobalValueAddress, LLVMGetPointerToGlobal};
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::values::AsValueRef;
use inkwell::values::FunctionValue;
use inkwell::values::GlobalValue;
use std::collections::HashMap;
use std::ffi::CString;

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

pub(crate) enum GlobalMapping<'ctx> {
    Function(FunctionValue<'ctx>),
    Global(GlobalValue<'ctx>),
}

impl<'ctx> JitModule<'ctx> {
    /// Stage 6: materialize a verified LLVM module into native code.
    pub(crate) fn from_module(
        module: Module<'ctx>,
        exported_functions: Vec<crate::ExportedFunction>,
        opt_level: OptLevel,
        global_mappings: Vec<(GlobalMapping<'ctx>, usize)>,
        vtable_slot_functions: HashMap<String, Vec<String>>,
        vtable_init_function_name: Option<String>,
        vtable_externs: Vec<(String, usize)>,
        cross_module_method_addresses: &HashMap<String, usize>,
        finalizer_patches: Vec<(String, String)>,
    ) -> Result<Self, CodegenError> {
        // No longer used; the LLVM init-function approach was abandoned
        // because MCJIT does not relocate function pointers in either
        // constant initializers or instructions reliably enough.
        let _ = vtable_init_function_name;
        let _ = vtable_externs;

        let debug = std::env::var("NEWCP_JIT_DEBUG").is_ok();

        // Build a name -> FunctionValue index for every method that the
        // vtables reference. Some entries may name methods inherited from
        // a base in another module — those aren't present in this LLVM
        // module and we leave the corresponding vtable slot NULL. They
        // become a runtime trap if a virtual call ever lands on that
        // slot, but unused inherited slots stay benign.
        let mut method_functions: HashMap<String, FunctionValue<'ctx>> = HashMap::new();
        for slot_fns in vtable_slot_functions.values() {
            for fn_name in slot_fns {
                if method_functions.contains_key(fn_name) {
                    continue;
                }
                if let Some(fn_val) = module.get_function(fn_name) {
                    method_functions.insert(fn_name.clone(), fn_val);
                } else if debug {
                    eprintln!(
                        "[jit] vtable patch: method function '{fn_name}' not in this LLVM module \
                         (likely an inherited cross-module method); slot will be left NULL"
                    );
                }
            }
        }

        let engine = module
            .create_jit_execution_engine(opt_level.to_llvm())
            .map_err(|e| CodegenError::Jit(e.to_string()))?;
        for (symbol, address) in global_mappings {
            match symbol {
                GlobalMapping::Function(function) => engine.add_global_mapping(&function, address),
                GlobalMapping::Global(global) => engine.add_global_mapping(&global, address),
            }
        }

        // Force MCJIT to finalize this module *before* we resolve any
        // addresses or patch any vtables. `LLVMGetPointerToGlobal` and
        // `LLVMGetGlobalValueAddress` would each implicitly trigger
        // finalization on first use, but relying on that side effect makes
        // the ordering fragile — and the implicit path only finalizes the
        // module containing the queried value. If a second module is ever
        // added to the same engine later, MCJIT will NOT retroactively
        // re-finalize earlier modules; it will only finalize the one being
        // queried. The current architecture is single-module-per-engine
        // (see `OwnedJitModule`), so this call simply makes the dependency
        // explicit. If incremental compilation is added (multiple modules
        // per engine, JIT'd in stages), each new module's
        // `JitModule::from_module`-equivalent must call this again before
        // resolving addresses for any global it owns.
        //
        // We don't emit `llvm.global_ctors`, so this only triggers
        // `finalizeObject()`; no user constructors run.
        engine.run_static_constructors();

        // Returns `Ok(Some(addr))` for a resolvable method, `Ok(None)` if
        // the method is inherited from a base in another module *and* its
        // address is not in the cross-module map (the slot then gets the
        // unimpl trap), or `Err` for a true resolution failure on a method
        // that IS in this module.
        let resolve_method_address = |fn_name: &str| -> Result<Option<usize>, CodegenError> {
            if let Some(fn_val) = method_functions.get(fn_name) {
                // SAFETY: `fn_val` belongs to the module now owned by `engine`.
                let addr = unsafe {
                    LLVMGetPointerToGlobal(engine.as_mut_ptr(), fn_val.as_value_ref())
                } as usize;
                if addr == 0 {
                    return Err(CodegenError::Jit(format!(
                        "vtable patch: method {fn_name} resolved to null address"
                    )));
                }
                return Ok(Some(addr));
            }
            // Cross-module fallback: the slot was seeded with a qualified
            // `Module.RecordType_Method` symbol whose body lives in an
            // already-materialized importer chain. The loader populated
            // `cross_module_method_addresses` keyed by that exact symbol.
            if let Some(&addr) = cross_module_method_addresses.get(fn_name) {
                return Ok(Some(addr));
            }
            Ok(None)
        };

        // DIAGNOSTIC: probe whether method functions are findable BEFORE
        // we ask for their addresses for vtable patching. Helps distinguish
        // "function absent from the module" from "materialization returned
        // a null code address".
        if debug {
            let mut all_method_fns: std::collections::BTreeSet<&str> =
                std::collections::BTreeSet::new();
            for slot_fns in vtable_slot_functions.values() {
                for f in slot_fns { all_method_fns.insert(f.as_str()); }
            }
            for fn_name in &all_method_fns {
                match resolve_method_address(fn_name) {
                    Ok(Some(a)) => eprintln!("[jit] probe {fn_name} @ 0x{a:x}"),
                    Ok(None) => eprintln!("[jit] probe {fn_name} (cross-module / NULL slot)"),
                    Err(e) => eprintln!("[jit] probe {fn_name} FAIL: {e}"),
                }
            }
        }

        // Patch every vtable from Rust. The vtables are emitted by codegen
        // as mutable globals with zero initializers; the methods are anchored
        // against DCE via `@llvm.used`. Here we:
        //   1. Resolve each method's address via `LLVMGetPointerToGlobal`
        //      on the concrete `FunctionValue`, bypassing MCJIT's fragile
        //      name-based symbol exposure rules for non-exported methods.
        //   2. Resolve each vtable's storage address via
        //      `LLVMGetGlobalValueAddress`.
        //   3. Write the method addresses into the vtable storage.
        // The TypeDesc.vtable field is a constant initializer holding a
        // pointer to the vtable global — MCJIT correctly relocates this
        // data-pointer-to-data-pointer reference, so dispatch through
        // `obj → tag → desc.vtable → vtable[i]` reads the patched addresses.
        for (vtable_name, slot_fns) in &vtable_slot_functions {
            let cvt_name = match CString::new(vtable_name.as_str()) {
                Ok(s) => s,
                Err(_) => continue,
            };
            // SAFETY: the engine pointer is valid for the lifetime of `engine`;
            // LLVMGetGlobalValueAddress returns 0 for unknown symbols.
            let vt_addr = unsafe {
                LLVMGetGlobalValueAddress(engine.as_mut_ptr(), cvt_name.as_ptr())
            };
            if vt_addr == 0 {
                if debug {
                    eprintln!("[jit] vtable {vtable_name}: address 0, skipping patch");
                }
                continue;
            }
            if debug {
                eprintln!("[jit] vtable {vtable_name} @ 0x{vt_addr:x} ({} slots)", slot_fns.len());
            }
            // Smalltalk-style fallback for inherited concrete methods we
            // can't bind locally: point the slot at
            // `__newcp_unimpl_method_trap`, which aborts with a
            // descriptive message instead of jumping to address 0 / a
            // garbage pointer if anything ever invokes the slot.
            let unimpl_addr = newcp_runtime::runtime_symbol_address(
                "__newcp_unimpl_method_trap",
            )
            .expect("__newcp_unimpl_method_trap is always exposed by newcp-runtime");

            let vt_ptr = vt_addr as *mut usize;
            for (slot_idx, fn_name) in slot_fns.iter().enumerate() {
                match resolve_method_address(fn_name) {
                    Ok(Some(fn_addr)) => {
                        // SAFETY: vt_ptr points at MCJIT-allocated storage of size
                        // [N x ptr] where N >= slot_fns.len(); we write within bounds.
                        unsafe { vt_ptr.add(slot_idx).write(fn_addr); }
                        if debug {
                            eprintln!("[jit]   {vtable_name}[{slot_idx}] = {fn_name} @ 0x{fn_addr:x}");
                        }
                    }
                    Ok(None) => {
                        // Inherited concrete method whose body lives in
                        // another JIT module. Slot is filled with the
                        // doesNotUnderstand stub so a virtual call lands
                        // somewhere safe & loud rather than at address 0.
                        unsafe { vt_ptr.add(slot_idx).write(unimpl_addr); }
                        if debug {
                            eprintln!(
                                "[jit]   {vtable_name}[{slot_idx}] = {fn_name} \
                                 (UNIMPL — points at __newcp_unimpl_method_trap)"
                            );
                        }
                    }
                    Err(e) => {
                        return Err(CodegenError::Jit(format!(
                            "vtable patch: cannot resolve method {fn_name} for {vtable_name}[{slot_idx}]: {e}"
                        )));
                    }
                }
            }
        }

        // Patch each TypeDesc.finalizer slot.  The TypeDesc layout
        // (matching newcp_runtime::gc::TypeDesc) places `finalizer` at
        // offset 16 (size@0, module@8, finalizer@16, ...).
        const FINALIZER_OFFSET: usize = 16;
        for (desc_global_name, fn_name) in &finalizer_patches {
            let cvt_name = match std::ffi::CString::new(desc_global_name.as_str()) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let desc_addr = unsafe {
                LLVMGetGlobalValueAddress(engine.as_mut_ptr(), cvt_name.as_ptr())
            } as usize;
            if desc_addr == 0 {
                if debug {
                    eprintln!(
                        "[jit] finalizer patch: TypeDesc global '{desc_global_name}' has \
                         address 0; skipping (function {fn_name} won't run as finalizer)"
                    );
                }
                continue;
            }
            match resolve_method_address(fn_name) {
                Ok(Some(fn_addr)) => {
                    let slot = (desc_addr + FINALIZER_OFFSET) as *mut usize;
                    // SAFETY: TypeDesc is a JIT-allocated #[repr(C)]
                    // struct with finalizer at offset 16; storage is
                    // writable (set_constant(false) at emit time).
                    unsafe { slot.write(fn_addr) };
                    if debug {
                        eprintln!(
                            "[jit] finalizer {desc_global_name}.finalizer = {fn_name} @ 0x{fn_addr:x}"
                        );
                    }
                }
                Ok(None) | Err(_) => {
                    if debug {
                        eprintln!(
                            "[jit] finalizer for {desc_global_name}: {fn_name} not resolvable; slot left null"
                        );
                    }
                }
            }
        }

        Ok(Self {
            engine,
            exported_functions,
        })
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
            .unwrap_or(public_name);

        self.lookup_export_address_by_llvm_name(llvm_name)
            .map_err(|e| CodegenError::Jit(format!(
                "symbol lookup failed for '{llvm_name}' (public: '{public_name}'): {e}"
            )))
    }

    /// Resolve a symbol by its raw LLVM name. Used by the loader to expose
    /// method-body addresses to importers for cross-module vtable patching.
    pub fn export_address_by_llvm_name(&self, llvm_name: &str) -> Result<usize, CodegenError> {
        self.lookup_export_address_by_llvm_name(llvm_name)
            .map_err(CodegenError::Jit)
    }

    fn lookup_export_address_by_llvm_name(&self, llvm_name: &str) -> Result<usize, String> {
        if let Ok(addr) = self.engine.get_function_address(llvm_name) {
            if addr != 0 {
                return Ok(addr as usize);
            }
        }

        let symbol_name = CString::new(llvm_name)
            .map_err(|_| format!("symbol '{llvm_name}' contains an interior NUL byte"))?;
        let addr = unsafe { LLVMGetGlobalValueAddress(self.engine.as_mut_ptr(), symbol_name.as_ptr()) };
        if addr != 0 {
            return Ok(addr as usize);
        }

        Err(format!("symbol '{llvm_name}' resolved to null address"))
    }
}

pub(crate) fn collect_global_mappings<'ctx>(
    module: &Module<'ctx>,
    extra_symbol_mappings: &HashMap<String, usize>,
) -> Result<Vec<(GlobalMapping<'ctx>, usize)>, CodegenError> {
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
            mappings.push((GlobalMapping::Function(function), address));
            continue;
        }

        if let Some(address) = newcp_runtime::runtime_symbol_address(symbol_name) {
            if debug { eprintln!("[jit] map (runtime) {symbol_name} -> 0x{address:x}"); }
            mappings.push((GlobalMapping::Function(function), address));
            continue;
        }

        if debug { eprintln!("[jit] UNRESOLVED external symbol: {symbol_name}"); }
    }

    for global in module.get_globals() {
        if global.get_linkage() != Linkage::External || global.get_initializer().is_some() {
            continue;
        }

        let symbol_name = global
            .get_name()
            .to_str()
            .map_err(|_| CodegenError::Jit("encountered non-UTF8 symbol name".to_string()))?;

        if let Some(address) = extra_symbol_mappings.get(symbol_name).copied() {
            if debug { eprintln!("[jit] map (extra global) {symbol_name} -> 0x{address:x}"); }
            mappings.push((GlobalMapping::Global(global), address));
            continue;
        }

        if let Some(address) = newcp_runtime::runtime_symbol_address(symbol_name) {
            if debug { eprintln!("[jit] map (runtime global) {symbol_name} -> 0x{address:x}"); }
            mappings.push((GlobalMapping::Global(global), address));
            continue;
        }

        if debug { eprintln!("[jit] UNRESOLVED external global: {symbol_name}"); }
    }

    Ok(mappings)
}
