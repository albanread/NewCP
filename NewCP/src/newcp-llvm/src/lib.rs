mod emit;
mod error;
mod jit;
mod module;
mod options;
mod types;

pub use error::CodegenError;
pub use jit::JitModule;
pub use module::BackendDiagnostic;
pub use options::{CodegenOptions, OptLevel};

use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use inkwell::context::Context;
use inkwell::targets::{CodeModel, RelocMode, Target, TargetMachine};

use newcp_ir::{IrModule, IrType};

/// Process-wide serializer for the parts of the LLVM pipeline that
/// touch global state (target init, MCJIT engine construction, global
/// symbol mapping registration).  LLVM's C bindings are not safe to
/// drive from multiple OS threads without prior `LLVMStartMultithreaded`
/// — cargo test runs every `#[test]` in its own thread by default, and
/// concurrent compiles were observed corrupting function-pointer state
/// across sessions (e.g. `TestSquare` returning `TestCircle`'s value
/// under load).
///
/// Each `LoaderSession`/dump-driver call into `compile_ir_module` or
/// `jit_module_with_symbol_mappings` takes this lock for the duration
/// of the LLVM call.  The held region is short — IR optimisation +
/// MCJIT engine bring-up — so per-thread throughput regression is
/// minimal, while cross-thread isolation is restored.
static LLVM_GLOBAL_LOCK: Mutex<()> = Mutex::new(());

fn llvm_lock() -> MutexGuard<'static, ()> {
    LLVM_GLOBAL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn create_target_machine(options: &CodegenOptions) -> Result<TargetMachine, CodegenError> {
    let triple = options
        .target_triple
        .as_deref()
        .map(inkwell::targets::TargetTriple::create)
        .unwrap_or_else(TargetMachine::get_default_triple);
    let target = Target::from_triple(&triple)
        .map_err(|e| CodegenError::Jit(format!("target from triple failed: {e}")))?;

    // Use the actual host CPU and its feature set so LLVM can emit and
    // optimise for the real instruction set (SSE4.2, AVX2, …) rather than a
    // lowest-common-denominator generic baseline.
    let cpu = TargetMachine::get_host_cpu_name();
    let features = TargetMachine::get_host_cpu_features();

    target
        .create_target_machine(
            &triple,
            cpu.to_str().unwrap_or("generic"),
            features.to_str().unwrap_or(""),
            options.opt_level.to_llvm(),
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| CodegenError::Jit("could not create target machine".to_string()))
}

/// A compiled module: the verified LLVM IR text and exported symbol manifest.
pub struct CompiledModule {
    pub module_name: String,
    pub source_path: Option<PathBuf>,
    /// The textual LLVM IR produced by this compilation. Stable inspection
    /// artifact — matches exactly the IR that was handed to the JIT.
    pub llvm_ir: String,
    pub exported_functions: Vec<ExportedFunction>,
    pub diagnostics: Vec<BackendDiagnostic>,
    /// `<TypeName>.vtable` -> ordered list of LLVM function names occupying its
    /// slots. The JIT layer uses these final emitted LLVM names to patch the
    /// mutable vtable globals after MCJIT materializes the module.
    pub vtable_slot_functions: HashMap<String, Vec<String>>,
    /// Legacy field from the abandoned synthetic-init-function approach.
    /// Always `None` in the current post-JIT vtable patching design.
    pub vtable_init_function_name: Option<String>,
    /// Legacy metadata retained for compatibility with existing plumbing.
    /// The current JIT patches the emitted mutable globals in place rather
    /// than redirecting them through externally allocated storage.
    pub vtable_externs: Vec<(String, usize)>,
    /// LLVM symbol name of the synthetic `<Module>.__init_types`
    /// function this compilation emitted, or `None` if the module
    /// declares no TypeDescs. The loader looks this symbol up by
    /// name and calls it before `<Module>.body`.
    pub init_types_function: Option<String>,
    /// `(typedesc_global_name, finalize_fn_name)` pairs.  The JIT
    /// resolves each `finalize_fn_name` to its address and writes
    /// it into the corresponding TypeDesc's `finalizer` slot
    /// (offset 16) at JIT-init time.
    pub finalizer_patches: Vec<(String, String)>,
}

/// Owns a JIT module together with the LLVM context it depends on.
///
/// `ExecutionEngine` carries a lifetime tied to the `Context`. This wrapper
/// keeps both in one owner so higher layers can retain and retire executable
/// generations safely.
pub struct OwnedJitModule {
    context: Box<Context>,
    jit: ManuallyDrop<JitModule<'static>>,
    /// Unique LLVM symbol names of every method body emitted in this module
    /// — derived from the union of `vtable_slot_functions`'s values, with
    /// any cross-module references (`Module.Foo`) excluded. The loader
    /// uses this to publish each module's method addresses for downstream
    /// importers' vtable patching.
    method_llvm_names: Vec<String>,
}

impl OwnedJitModule {
    pub fn from_compiled(
        compiled: CompiledModule,
        options: &CodegenOptions,
    ) -> Result<Self, CodegenError> {
        Self::from_compiled_with_symbol_mappings(compiled, options, &HashMap::new())
    }

    pub fn from_compiled_with_symbol_mappings(
        compiled: CompiledModule,
        options: &CodegenOptions,
        extra_symbol_mappings: &HashMap<String, usize>,
    ) -> Result<Self, CodegenError> {
        // Snapshot the unique local-method LLVM names *before* compiled is
        // consumed by the JIT pipeline. Names containing `.` are
        // cross-module references and are skipped — they're satisfied via
        // `extra_symbol_mappings`, not by this module's image.
        let mut method_llvm_names: Vec<String> = compiled
            .vtable_slot_functions
            .values()
            .flat_map(|v| v.iter())
            .filter(|n| !n.contains('.'))
            .cloned()
            .collect();
        method_llvm_names.sort();
        method_llvm_names.dedup();

        let context = Box::new(Context::create());
        let jit = jit_module_with_symbol_mappings(&context, compiled, options, extra_symbol_mappings)?;
        let jit = unsafe { std::mem::transmute::<JitModule<'_>, JitModule<'static>>(jit) };

        Ok(Self {
            context,
            jit: ManuallyDrop::new(jit),
            method_llvm_names,
        })
    }

    /// Resolve every locally-emitted method-body symbol to its address.
    /// The loader publishes these as cross-module symbols (keyed
    /// `<ImporterSeesAs>.<llvm_name>`) so importers' vtable slots can be
    /// patched to point at the right body.
    pub fn collect_method_addresses(&self) -> HashMap<String, usize> {
        let mut out = HashMap::new();
        for name in &self.method_llvm_names {
            if let Ok(addr) = self.jit.export_address_by_llvm_name(name) {
                out.insert(name.clone(), addr);
            }
        }
        out
    }

    pub fn exported_function_count(&self) -> usize {
        self.jit.exported_functions.len()
    }

    /// Resolve any LLVM symbol name to its address. Used by the
    /// loader for synthetic codegen-only functions (like
    /// `<Module>.__init_types`) that aren't in the public-export
    /// manifest but live in the JIT-emitted module.
    pub fn export_address_by_llvm_name(&self, llvm_name: &str) -> Result<usize, CodegenError> {
        self.jit.export_address_by_llvm_name(llvm_name)
    }

    pub fn export_address(&self, public_name: &str) -> Result<usize, CodegenError> {
        self.jit.export_address(public_name)
    }

    pub fn exported_public_names(&self) -> Vec<String> {
        self.jit
            .exported_functions
            .iter()
            .map(|export| export.public_name.clone())
            .collect()
    }

    pub unsafe fn get_function<F>(&self, public_name: &str) -> Result<F, CodegenError>
    where
        F: Copy,
    {
        unsafe { self.jit.get_function(public_name) }
    }
}

impl Drop for OwnedJitModule {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.jit);
        }
        let _ = &self.context;
    }
}

impl std::fmt::Debug for OwnedJitModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedJitModule")
            .field("exported_function_count", &self.exported_function_count())
            .finish()
    }
}

/// An exported procedure visible to the JIT symbol lookup.
#[derive(Debug, Clone)]
pub struct ExportedFunction {
    /// Source-level public name (e.g. `Foo.Bar`).
    pub public_name: String,
    /// LLVM symbol name used during JIT lookup.
    pub llvm_name: String,
    pub params: Vec<IrType>,
    pub ret_ty: IrType,
}

// ─── Stage 1+2+3+4+5: IR → CompiledModule ────────────────────────────────────

/// Compile an `IrModule` to a `CompiledModule`.
///
/// This is the real backend entry point. All stages run in sequence; if any
/// stage fails a `CodegenError` is returned.
pub fn compile_ir_module(
    ir_module: &IrModule,
    options: &CodegenOptions,
) -> Result<CompiledModule, CodegenError> {
    // Serialize LLVM-global access (target init, pass-manager
    // construction, etc.) so parallel test threads don't race
    // through unsynchronized C-ABI state.  See `LLVM_GLOBAL_LOCK`.
    let _llvm_guard = llvm_lock();
    // Initialize LLVM native target once; safe to call multiple times.
    inkwell::targets::Target::initialize_native(&inkwell::targets::InitializationConfig::default())
        .map_err(|e| CodegenError::Jit(format!("target initialization failed: {e}")))?;

    let context = Context::create();
    // Stage 2: create target machine first so the module can be stamped with
    // the correct triple and data layout before any passes or optimisations run.
    let machine = create_target_machine(options)?;
    let mut cg = module::CodegenModule::new(&context, ir_module, options, &machine)?;

    // Stage 3: declare all globals and procedures.
    cg.plan(ir_module, options)?;

    // Stage 4: emit all procedure bodies.
    for proc in &ir_module.procedures {
        let mut emitter = emit::ProcedureEmitter::new(&mut cg, options);
        emitter.emit(proc)?;
    }

    // Stage 4b: emit the synthetic per-module `__init_types` function
    // (calls `__newcp_register_type` for every TypeDesc this module
    // declares). The loader runs this before the module's body so
    // `Kernel.ThisType` works without requiring a prior allocation.
    let init_types_name = cg.emit_init_types_function(ir_module, options);

    cg.optimize(options.opt_level, &machine)?;

    // Stage 5: verify.
    cg.verify()?;

    let llvm_ir = cg.print_to_string();

    // Build exported symbol manifest.
    let exported_functions = ir_module
        .procedures
        .iter()
        .filter(|p| p.exported)
        .map(|p| ExportedFunction {
            public_name: CodegenOptions::public_symbol_name(&ir_module.name, &p.name),
            llvm_name: options.exported_symbol_name(&ir_module.name, &p.name),
            params: p.params.iter().map(|(_, ty)| ty.clone()).collect(),
            ret_ty: p.ret_ty.clone(),
        })
        .collect();

    let vtable_slot_functions = cg.planner.vtable_slot_functions.clone();
    let vtable_init_function_name = cg.planner.vtable_init_function_name.clone();
    let vtable_externs = cg.planner.vtable_externs.clone();
    let finalizer_patches = cg.planner.finalizer_patches.clone();

    Ok(CompiledModule {
        module_name: ir_module.name.clone(),
        source_path: None,
        llvm_ir,
        exported_functions,
        diagnostics: cg.diagnostics.clone(),
        vtable_slot_functions,
        vtable_init_function_name,
        vtable_externs,
        init_types_function: init_types_name,
        finalizer_patches,
    })
}

/// Stage 6: materialize a `CompiledModule` into a `JitModule`.
///
/// The `CompiledModule` is consumed; use its `llvm_ir` field for inspection
/// before calling this if needed.
pub fn jit_module<'ctx>(
    context: &'ctx Context,
    compiled: CompiledModule,
    options: &CodegenOptions,
) -> Result<JitModule<'ctx>, CodegenError> {
    jit_module_with_symbol_mappings(context, compiled, options, &HashMap::new())
}

pub fn jit_module_with_symbol_mappings<'ctx>(
    context: &'ctx Context,
    compiled: CompiledModule,
    options: &CodegenOptions,
    extra_symbol_mappings: &HashMap<String, usize>,
) -> Result<JitModule<'ctx>, CodegenError> {
    // Serialize MCJIT engine bring-up + global-symbol mapping
    // registration with other LLVM-touching threads.  See
    // `LLVM_GLOBAL_LOCK`.
    let _llvm_guard = llvm_lock();
    // Capture the vtable slot info and init-function name before consuming
    // `compiled` for IR parsing.
    let vtable_slot_functions = compiled.vtable_slot_functions.clone();
    let vtable_init_function_name = compiled.vtable_init_function_name.clone();
    let vtable_externs = compiled.vtable_externs.clone();
    let finalizer_patches = compiled.finalizer_patches.clone();
    let exported_functions = compiled.exported_functions.clone();
    // Re-parse the verified IR string back into an LLVM module so we can hand
    // it to the JIT. This round-trip is intentional: it proves the textual dump
    // is the exact artifact that gets executed.
    let mut llvm_ir = compiled.llvm_ir.into_bytes();
    llvm_ir.push(0);
    let memory_buf = inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
        &llvm_ir,
        &compiled.module_name,
    );
    let module = context
        .create_module_from_ir(memory_buf)
        .map_err(|e| CodegenError::Jit(format!("IR round-trip parse failed: {e}")))?;
    let global_mappings = jit::collect_global_mappings(&module, extra_symbol_mappings)?;

    // The same `extra_symbol_mappings` map carries cross-module method
    // addresses. The vtable patcher consults it for slot names that don't
    // match any local function (qualified `Module.Foo` form).
    JitModule::from_module(
        module,
        exported_functions,
        options.opt_level,
        global_mappings,
        vtable_slot_functions,
        vtable_init_function_name,
        vtable_externs,
        extra_symbol_mappings,
        finalizer_patches,
    )
}

// ─── Convenience path-based helpers ──────────────────────────────────────────

/// Parse + analyze + lower + compile a source file, returning a `CompiledModule`.
pub fn compile_from_path(
    path: &Path,
    options: &CodegenOptions,
) -> Result<CompiledModule, CodegenError> {
    let ir_module = lower_from_path(path)?;
    let mut compiled = compile_ir_module(&ir_module, options)?;
    compiled.source_path = Some(path.to_path_buf());
    Ok(compiled)
}

fn lower_from_path(path: &Path) -> Result<IrModule, CodegenError> {
    newcp_ir::lower_from_path(path).map_err(CodegenError::Parse)
}

fn render_backend_diagnostics(diagnostics: &[BackendDiagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|diagnostic| format!("warning: {}", diagnostic.render()))
        .collect()
}


// ─── Driver-facing string dumps ───────────────────────────────────────────────

/// Produce the LLVM IR text for the module at `path`.
///
/// Uses real LLVM IR produced by `compile_ir_module`, not a placeholder.
pub fn dump_llvm(path: &Path) -> String {
    dump_llvm_with_options(path, &CodegenOptions::default())
}

pub fn dump_llvm_with_options(path: &Path, options: &CodegenOptions) -> String {
    match compile_from_path(path, &options) {
        Ok(compiled) => {
            if compiled.diagnostics.is_empty() {
                compiled.llvm_ir
            } else {
                let mut lines = render_backend_diagnostics(&compiled.diagnostics);
                lines.push(compiled.llvm_ir);
                lines.join("\n")
            }
        }
        Err(e) => format!("newcp-llvm error\ninput: {}\nerror: {e}", path.display()),
    }
}

/// Produce native assembly text for the module at `path`.
pub fn dump_asm(path: &Path) -> String {
    dump_asm_with_options(path, &CodegenOptions::default())
}

pub fn dump_asm_with_options(path: &Path, options: &CodegenOptions) -> String {
    use inkwell::targets::FileType;

    let ir_module = match lower_from_path(path) {
        Ok(m) => m,
        Err(e) => {
            return format!("newcp-llvm assembly error\ninput: {}\nerror: {e}", path.display())
        }
    };

    Target::initialize_native(&inkwell::targets::InitializationConfig::default())
        .unwrap_or_default();

    let context = Context::create();
    // Create the target machine first so the module triple and data layout
    // are stamped correctly before emission and optimisation.
    let machine = match create_target_machine(options) {
        Ok(machine) => machine,
        Err(e) => {
            return format!("newcp-llvm assembly error\ninput: {}\nerror: {e}", path.display())
        }
    };
    let mut cg = match module::CodegenModule::new(&context, &ir_module, &options, &machine) {
        Ok(c) => c,
        Err(e) => {
            return format!("newcp-llvm assembly error\ninput: {}\nerror: {e}", path.display())
        }
    };

    if let Err(e) = cg.plan(&ir_module, &options) {
        return format!("newcp-llvm assembly error\ninput: {}\nerror: {e}", path.display());
    }

    for proc in &ir_module.procedures {
        let mut emitter = emit::ProcedureEmitter::new(&mut cg, &options);
        if let Err(e) = emitter.emit(proc) {
            return format!("newcp-llvm assembly error\ninput: {}\nerror: {e}", path.display());
        }
    }

    if let Err(e) = cg.optimize(options.opt_level, &machine) {
        return format!("newcp-llvm assembly error\ninput: {}\nerror: {e}", path.display());
    }

    if let Err(e) = cg.verify() {
        return format!("newcp-llvm assembly error\ninput: {}\nerror: {e}", path.display());
    }

    let diagnostic_lines = render_backend_diagnostics(&cg.diagnostics);
    let module = cg.into_module();
    match machine.write_to_memory_buffer(&module, FileType::Assembly) {
        Ok(buf) => {
            let assembly = String::from_utf8_lossy(buf.as_slice()).into_owned();
            if diagnostic_lines.is_empty() {
                assembly
            } else {
                let mut lines = diagnostic_lines;
                lines.push(assembly);
                lines.join("\n")
            }
        }
        Err(e) => format!(
            "newcp-llvm assembly error\ninput: {}\nerror: {e}",
            path.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use inkwell::context::Context;

    use newcp_runtime::console;

    use super::{compile_from_path, jit_module, CodegenOptions};

    fn temp_source_path(file_name: &str) -> PathBuf {
        std::env::temp_dir().join(file_name)
    }

    #[test]
    fn jit_executes_imported_console_calls_against_runtime_capture() {
        let source_path = temp_source_path("newcp-console-exec.cp");
        std::fs::write(
            &source_path,
            concat!(
                "MODULE Demo;\n",
                "IMPORT Console;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Console.WriteInt(21);\n",
                "  Console.WriteLn()\n",
                "END Run;\n",
                "END Demo."
            ),
        )
        .expect("failed to write temporary console execution module");

        console::reset();
        console::begin_capture();

        let options = CodegenOptions::default();
        let compiled = compile_from_path(&source_path, &options)
            .expect("console execution module should compile");
        let context = Context::create();
        let jit = jit_module(&context, compiled, &options)
            .expect("console execution module should JIT materialize");

        type RunFn = unsafe extern "C" fn();
        let run = unsafe { jit.get_function::<RunFn>("Demo.Run") }
            .expect("exported Demo.Run should resolve from the JIT");
        unsafe { run() };

        assert_eq!(console::end_capture(), "21\n");
        console::reset();

        let _ = std::fs::remove_file(&source_path);
    }
}
