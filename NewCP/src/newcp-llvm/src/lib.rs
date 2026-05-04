mod emit;
mod error;
mod jit;
mod module;
mod options;
mod types;

pub use error::CodegenError;
pub use jit::JitModule;
pub use options::{CodegenOptions, OptLevel};

use std::path::Path;

use inkwell::context::Context;

use newcp_ir::{IrModule, IrType};

/// A compiled module: the verified LLVM IR text and exported symbol manifest.
pub struct CompiledModule {
    pub module_name: String,
    /// The textual LLVM IR produced by this compilation. Stable inspection
    /// artifact — matches exactly the IR that was handed to the JIT.
    pub llvm_ir: String,
    pub exported_functions: Vec<ExportedFunction>,
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
    // Initialize LLVM native target once; safe to call multiple times.
    inkwell::targets::Target::initialize_native(&inkwell::targets::InitializationConfig::default())
        .map_err(|e| CodegenError::Jit(format!("target initialization failed: {e}")))?;

    let context = Context::create();
    // Stage 2: context + module + builder.
    let mut cg = module::CodegenModule::new(&context, ir_module, options)?;

    // Stage 3: declare all globals and procedures.
    cg.plan(ir_module, options)?;

    // Stage 4: emit all procedure bodies.
    for proc in &ir_module.procedures {
        let mut emitter = emit::ProcedureEmitter::new(&mut cg, options);
        emitter.emit(proc)?;
    }

    // Stage 5: verify.
    cg.verify()?;

    let llvm_ir = cg.print_to_string();

    // Build exported symbol manifest.
    let exported_functions = ir_module
        .procedures
        .iter()
        .filter(|p| p.exported)
        .map(|p| ExportedFunction {
            public_name: format!("{}.{}", ir_module.name, p.name),
            llvm_name: p.name.clone(),
            params: p.params.iter().map(|(_, ty)| ty.clone()).collect(),
            ret_ty: p.ret_ty.clone(),
        })
        .collect();

    Ok(CompiledModule {
        module_name: ir_module.name.clone(),
        llvm_ir,
        exported_functions,
    })
}

/// Stage 6: materialize a `CompiledModule` into a `JitModule`.
///
/// The `CompiledModule` is consumed; use its `llvm_ir` field for inspection
/// before calling this if needed.
pub fn jit_module<'ctx>(
    context: &'ctx Context,
    compiled: CompiledModule,
    _options: &CodegenOptions,
) -> Result<JitModule<'ctx>, CodegenError> {
    // Re-parse the verified IR string back into an LLVM module so we can hand
    // it to the JIT. This round-trip is intentional: it proves the textual dump
    // is the exact artifact that gets executed.
    let memory_buf = inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
        compiled.llvm_ir.as_bytes(),
        &compiled.module_name,
    );
    let module = context
        .create_module_from_ir(memory_buf)
        .map_err(|e| CodegenError::Jit(format!("IR round-trip parse failed: {e}")))?;

    JitModule::from_module(module, compiled.exported_functions)
}

// ─── Convenience path-based helpers ──────────────────────────────────────────

/// Parse + analyze + lower + compile a source file, returning a `CompiledModule`.
pub fn compile_from_path(
    path: &Path,
    options: &CodegenOptions,
) -> Result<CompiledModule, CodegenError> {
    let ir_module = lower_from_path(path)?;
    compile_ir_module(&ir_module, options)
}

fn lower_from_path(path: &Path) -> Result<IrModule, CodegenError> {
    newcp_ir::lower_from_path(path).map_err(CodegenError::Parse)
}

// ─── Driver-facing string dumps ───────────────────────────────────────────────

/// Produce the LLVM IR text for the module at `path`.
///
/// Uses real LLVM IR produced by `compile_ir_module`, not a placeholder.
pub fn dump_llvm(path: &Path) -> String {
    let options = CodegenOptions::default();
    match compile_from_path(path, &options) {
        Ok(compiled) => compiled.llvm_ir,
        Err(e) => format!("newcp-llvm error\ninput: {}\nerror: {e}", path.display()),
    }
}

/// Produce native assembly text for the module at `path`.
pub fn dump_asm(path: &Path) -> String {
    use inkwell::targets::{CodeModel, FileType, RelocMode, Target, TargetMachine};

    let options = CodegenOptions::default();
    let ir_module = match lower_from_path(path) {
        Ok(m) => m,
        Err(e) => {
            return format!("newcp-llvm assembly error\ninput: {}\nerror: {e}", path.display())
        }
    };

    Target::initialize_native(&inkwell::targets::InitializationConfig::default())
        .unwrap_or_default();

    let context = Context::create();
    let mut cg = match module::CodegenModule::new(&context, &ir_module, &options) {
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

    if let Err(e) = cg.verify() {
        return format!("newcp-llvm assembly error\ninput: {}\nerror: {e}", path.display());
    }

    let triple = TargetMachine::get_default_triple();
    let target = match Target::from_triple(&triple) {
        Ok(t) => t,
        Err(e) => {
            return format!(
                "newcp-llvm assembly error\ninput: {}\nerror: target from triple: {e}",
                path.display()
            )
        }
    };
    let machine = match target.create_target_machine(
        &triple,
        "generic",
        "",
        inkwell::OptimizationLevel::None,
        RelocMode::Default,
        CodeModel::Default,
    ) {
        Some(m) => m,
        None => {
            return format!(
                "newcp-llvm assembly error\ninput: {}\nerror: could not create target machine",
                path.display()
            )
        }
    };

    let module = cg.into_module();
    match machine.write_to_memory_buffer(&module, FileType::Assembly) {
        Ok(buf) => String::from_utf8_lossy(buf.as_slice()).into_owned(),
        Err(e) => format!(
            "newcp-llvm assembly error\ninput: {}\nerror: {e}",
            path.display()
        ),
    }
}
