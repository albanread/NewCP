/// Configuration for one code generation job.
///
/// Constructed once by the driver and passed by reference through all stages.
/// Must not carry mutable state.
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    /// Override the host target triple. `None` means use the native host.
    pub target_triple: Option<String>,
    /// Optimization level. Defaults to `Default` (`O2`).
    pub opt_level: OptLevel,
    /// Reserved for later — must be `false` in the first slice.
    pub emit_debug_info: bool,
    /// When `true`, any `Unsupported` error is fatal.
    /// When `false`, unsupported instructions emit a trap stub instead of
    /// aborting compilation, which is useful for bring-up.
    pub strict_unsupported: bool,
    /// Optional loader-assigned generation for exported CP procedures.
    ///
    /// When present, exported procedures are emitted with generation-qualified
    /// internal LLVM symbol names so multiple generations can coexist safely.
    pub export_generation: Option<u64>,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            target_triple: None,
            opt_level: OptLevel::Default,
            emit_debug_info: false,
            strict_unsupported: false,
            export_generation: None,
        }
    }
}

impl CodegenOptions {
    pub fn exported_symbol_name(&self, module_name: &str, proc_name: &str) -> String {
        match self.export_generation {
            Some(generation) => format!("{module_name}$g{generation}${proc_name}"),
            None => proc_name.to_string(),
        }
    }

    pub fn public_symbol_name(module_name: &str, proc_name: &str) -> String {
        format!("{module_name}.{proc_name}")
    }

    pub fn import_symbol_name(&self, module_name: &str, proc_name: &str) -> String {
        Self::public_symbol_name(module_name, proc_name)
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

impl OptLevel {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "less" => Some(Self::Less),
            "default" => Some(Self::Default),
            "aggressive" => Some(Self::Aggressive),
            _ => None,
        }
    }

    pub fn to_llvm(self) -> inkwell::OptimizationLevel {
        match self {
            Self::None => inkwell::OptimizationLevel::None,
            Self::Less => inkwell::OptimizationLevel::Less,
            Self::Default => inkwell::OptimizationLevel::Default,
            Self::Aggressive => inkwell::OptimizationLevel::Aggressive,
        }
    }
}
