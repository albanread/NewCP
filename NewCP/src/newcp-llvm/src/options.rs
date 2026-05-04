/// Configuration for one code generation job.
///
/// Constructed once by the driver and passed by reference through all stages.
/// Must not carry mutable state.
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    /// Override the host target triple. `None` means use the native host.
    pub target_triple: Option<String>,
    /// Optimization level. First slice should use `None`.
    pub opt_level: OptLevel,
    /// Reserved for later — must be `false` in the first slice.
    pub emit_debug_info: bool,
    /// When `true`, any `Unsupported` error is fatal.
    /// When `false`, unsupported instructions emit a trap stub instead of
    /// aborting compilation, which is useful for bring-up.
    pub strict_unsupported: bool,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            target_triple: None,
            opt_level: OptLevel::None,
            emit_debug_info: false,
            strict_unsupported: false,
        }
    }
}

/// Optimization level for the code generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
    None,
    Less,
    Default,
    Aggressive,
}
