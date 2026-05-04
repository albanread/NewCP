/// Errors produced during LLVM code generation.
#[derive(Debug)]
pub enum CodegenError {
    /// Front-end handoff failed: parse or sema error.
    Parse(String),
    /// An IR construct is not yet supported by the backend.
    Unsupported { stage: &'static str, detail: String },
    /// The generated LLVM module failed verification.
    Verify(String),
    /// JIT materialization or symbol lookup failed.
    Jit(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::Parse(msg) => write!(f, "parse/sema error: {msg}"),
            CodegenError::Unsupported { stage, detail } => {
                write!(f, "unsupported at {stage}: {detail}")
            }
            CodegenError::Verify(msg) => write!(f, "LLVM verification failed: {msg}"),
            CodegenError::Jit(msg) => write!(f, "JIT error: {msg}"),
        }
    }
}

impl std::error::Error for CodegenError {}
