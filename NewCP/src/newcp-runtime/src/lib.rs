pub mod console;
pub mod gc;

#[cfg(windows)]
pub mod igui;

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind {
    ResidentRust,
    JitCompiled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Constant,
    Type,
    Variable,
    Procedure,
    Command,
}

#[derive(Debug, Clone)]
pub struct ExportEntry {
    pub name: String,
    pub kind: ExportKind,
}

impl ExportEntry {
    pub fn command(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ExportKind::Command,
        }
    }

    pub fn procedure(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ExportKind::Procedure,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExportDirectory {
    entries: Vec<ExportEntry>,
}

impl ExportDirectory {
    pub fn new(entries: Vec<ExportEntry>) -> Self {
        let mut directory = Self { entries };
        directory.entries.sort_by(|left, right| left.name.cmp(&right.name));
        directory
    }

    pub fn names(&self) -> Vec<&str> {
        self.entries.iter().map(|entry| entry.name.as_str()).collect()
    }

    pub fn find(&self, name: &str) -> Option<&ExportEntry> {
        self.entries.iter().find(|entry| entry.name == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInvocation {
    pub module_name: String,
    pub command_name: String,
}

impl CommandInvocation {
    pub fn render(&self) -> String {
        format!("invoke {}.{}", self.module_name, self.command_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustCommandHandlerSpec {
    pub command_name: String,
    pub action_summary: String,
}

impl RustCommandHandlerSpec {
    pub fn new(command_name: impl Into<String>, action_summary: impl Into<String>) -> Self {
        Self {
            command_name: command_name.into(),
            action_summary: action_summary.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportDescriptor {
    pub name: String,
    pub kind: ExportKind,
    pub slot_index: u32,
    pub entry_address: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceSymbolDescriptor {
    pub name: String,
    pub kind: ExportKind,
    pub exported: bool,
    pub binding: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceDescriptor {
    pub module_name: String,
    pub imports: Vec<String>,
    pub symbols: Vec<InterfaceSymbolDescriptor>,
}

impl InterfaceDescriptor {
    pub fn render(&self) -> String {
        let imports = if self.imports.is_empty() {
            "<none>".to_string()
        } else {
            self.imports.join(", ")
        };

        let symbols = if self.symbols.is_empty() {
            "<none>".to_string()
        } else {
            self.symbols
                .iter()
                .map(|symbol| format!("{}:{:?}:{}", symbol.name, symbol.kind, symbol.binding))
                .collect::<Vec<_>>()
                .join(", ")
        };

        format!(
            "interface-module: {}\ninterface-imports: {}\ninterface-symbols: {}",
            self.module_name, imports, symbols
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDescriptor {
    pub id: ModuleId,
    pub name: String,
    pub kind: ModuleKind,
    pub import_count: u32,
    pub export_count: u32,
    pub code_address: u64,
    pub data_address: u64,
    pub export_directory_address: u64,
    pub init_entry_address: u64,
    pub code_size: u32,
    pub data_size: u32,
    pub exports: Vec<ExportDescriptor>,
}

#[derive(Debug, Clone)]
pub struct ModuleRecord {
    pub id: ModuleId,
    pub name: String,
    pub kind: ModuleKind,
    pub imports: Vec<String>,
    pub exports: ExportDirectory,
    pub descriptor: ModuleDescriptor,
    pub interface: InterfaceDescriptor,
    pub initialized: bool,
    pub invalidated: bool,
    pub init_routine: String,
}

impl ModuleRecord {
    pub fn resident(
        id: u32,
        name: impl Into<String>,
        descriptor: ModuleDescriptor,
        interface: InterfaceDescriptor,
    ) -> Self {
        Self {
            id: ModuleId(id),
            name: name.into(),
            kind: ModuleKind::ResidentRust,
            imports: vec![],
            exports: ExportDirectory::default(),
            descriptor,
            interface,
            initialized: false,
            invalidated: false,
            init_routine: "bootstrap".to_string(),
        }
    }

    pub fn jit(
        id: u32,
        name: impl Into<String>,
        imports: Vec<String>,
        exports: ExportDirectory,
        descriptor: ModuleDescriptor,
        interface: InterfaceDescriptor,
        init_routine: impl Into<String>,
    ) -> Self {
        Self {
            id: ModuleId(id),
            name: name.into(),
            kind: ModuleKind::JitCompiled,
            imports,
            exports,
            descriptor,
            interface,
            initialized: false,
            invalidated: false,
            init_routine: init_routine.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostedModuleArtifact {
    pub name: String,
    pub imports: Vec<String>,
    pub exports: ExportDirectory,
    pub init_routine: String,
    pub source_summary: String,
    pub command_handlers: Vec<RustCommandHandlerSpec>,
}

impl HostedModuleArtifact {
    pub fn new(
        name: impl Into<String>,
        imports: Vec<String>,
        exports: ExportDirectory,
        init_routine: impl Into<String>,
        source_summary: impl Into<String>,
        command_handlers: Vec<RustCommandHandlerSpec>,
    ) -> Self {
        Self {
            name: name.into(),
            imports,
            exports,
            init_routine: init_routine.into(),
            source_summary: source_summary.into(),
            command_handlers,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NativeExportBinding {
    pub export: ExportEntry,
    pub address: usize,
}

impl NativeExportBinding {
    pub fn procedure(name: impl Into<String>, address: usize) -> Self {
        Self {
            export: ExportEntry::procedure(name),
            address,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NativeModuleArtifact {
    pub hosted: HostedModuleArtifact,
    pub export_bindings: Vec<NativeExportBinding>,
}

impl NativeModuleArtifact {
    pub fn new(hosted: HostedModuleArtifact, export_bindings: Vec<NativeExportBinding>) -> Self {
        Self {
            hosted,
            export_bindings,
        }
    }

    pub fn export_address(&self, export_name: &str) -> Option<usize> {
        self.export_bindings
            .iter()
            .find(|binding| binding.export.name == export_name)
            .map(|binding| binding.address)
    }
}

#[derive(Debug, Clone)]
pub struct CompiledModuleArtifact {
    pub name: String,
    pub imports: Vec<String>,
    pub exports: ExportDirectory,
    pub init_routine: String,
    pub source_summary: String,
}

impl CompiledModuleArtifact {
    pub fn new(
        name: impl Into<String>,
        imports: Vec<String>,
        exports: ExportDirectory,
        init_routine: impl Into<String>,
        source_summary: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            imports,
            exports,
            init_routine: init_routine.into(),
            source_summary: source_summary.into(),
        }
    }
}

pub trait CompilerService {
    fn service_name(&self) -> &str;
    fn host_language(&self) -> &str;
    fn compile_module(
        &self,
        name: &str,
        imports: &[&str],
        exports: Vec<ExportEntry>,
        init_routine: &str,
        source_summary: &str,
    ) -> CompiledModuleArtifact;
}

#[derive(Debug, Clone)]
pub struct ResidentCompiler {
    service_name: String,
    host_language: String,
}

impl ResidentCompiler {
    pub fn bootstrap() -> Self {
        Self {
            service_name: "resident-compiler".to_string(),
            host_language: "Rust".to_string(),
        }
    }
}

impl CompilerService for ResidentCompiler {
    fn service_name(&self) -> &str {
        &self.service_name
    }

    fn host_language(&self) -> &str {
        &self.host_language
    }

    fn compile_module(
        &self,
        name: &str,
        imports: &[&str],
        exports: Vec<ExportEntry>,
        init_routine: &str,
        source_summary: &str,
    ) -> CompiledModuleArtifact {
        CompiledModuleArtifact::new(
            name,
            imports.iter().map(|item| (*item).to_string()).collect(),
            ExportDirectory::new(exports),
            init_routine,
            format!("compiled by {} from {}", self.service_name, source_summary),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootPhase {
    KernelReady,
    InitReady,
    CompilerReady,
    JitReady,
    BaseModulesReady,
}

impl BootPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::KernelReady => "kernel-ready",
            Self::InitReady => "init-ready",
            Self::CompilerReady => "compiler-ready",
            Self::JitReady => "jit-ready",
            Self::BaseModulesReady => "base-modules-ready",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompilerState {
    pub initialized: bool,
    pub host_language: String,
    pub service_name: String,
}

impl CompilerState {
    pub fn bootstrap(service: &dyn CompilerService) -> Self {
        Self {
            initialized: true,
            host_language: service.host_language().to_string(),
            service_name: service.service_name().to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KernelState {
    pub pointer_width_bits: u8,
    pub target_triple: String,
    pub modules: Vec<ModuleRecord>,
    pub command_handlers: HashMap<String, String>,
}

impl KernelState {
    pub fn bootstrap(target_triple: impl Into<String>) -> Self {
        Self {
            pointer_width_bits: 64,
            target_triple: target_triple.into(),
            modules: vec![],
            command_handlers: HashMap::new(),
        }
    }

    pub fn register_resident_module(&mut self, name: impl Into<String>) -> ModuleId {
        let id = ModuleId((self.modules.len() as u32) + 1);
        let name = name.into();
        let interface = Self::build_interface_descriptor(&name, &[], &ExportDirectory::default());
        let descriptor = Self::build_module_descriptor(
            id,
            &name,
            ModuleKind::ResidentRust,
            &[],
            &ExportDirectory::default(),
            "bootstrap",
        );
        self.modules.push(ModuleRecord::resident(id.0, name, descriptor, interface));
        id
    }

    pub fn register_jit_module(
        &mut self,
        name: impl Into<String>,
        imports: Vec<String>,
        exports: ExportDirectory,
        init_routine: impl Into<String>,
    ) -> ModuleId {
        let id = ModuleId((self.modules.len() as u32) + 1);
        let name = name.into();
        let init_routine = init_routine.into();
        let interface = Self::build_interface_descriptor(&name, &imports, &exports);
        let descriptor = Self::build_module_descriptor(
            id,
            &name,
            ModuleKind::JitCompiled,
            &imports,
            &exports,
            &init_routine,
        );
        self.modules
            .push(ModuleRecord::jit(id.0, name, imports, exports, descriptor, interface, init_routine));
        id
    }

    pub fn register_compiled_module(&mut self, artifact: CompiledModuleArtifact) -> ModuleId {
        self.retire_active_module(&artifact.name);
        self.register_jit_module(
            artifact.name,
            artifact.imports,
            artifact.exports,
            artifact.init_routine,
        )
    }

    pub fn register_hosted_module(&mut self, artifact: HostedModuleArtifact) -> ModuleId {
        self.retire_active_module(&artifact.name);

        let id = ModuleId((self.modules.len() as u32) + 1);
        let descriptor = Self::build_module_descriptor(
            id,
            &artifact.name,
            ModuleKind::ResidentRust,
            &artifact.imports,
            &artifact.exports,
            &artifact.init_routine,
        );
        let interface = Self::build_interface_descriptor(&artifact.name, &artifact.imports, &artifact.exports);

        for handler in &artifact.command_handlers {
            self.command_handlers.insert(
                format!("{}.{}", artifact.name, handler.command_name),
                handler.action_summary.clone(),
            );
        }

        self.modules.push(ModuleRecord {
            id,
            name: artifact.name,
            kind: ModuleKind::ResidentRust,
            imports: artifact.imports,
            exports: artifact.exports,
            descriptor,
            interface,
            initialized: false,
            invalidated: false,
            init_routine: artifact.init_routine,
        });

        id
    }

    pub fn register_native_module(&mut self, artifact: NativeModuleArtifact) -> ModuleId {
        self.register_hosted_module(artifact.hosted)
    }

    pub fn this_mod(&self, name: &str) -> Option<&ModuleRecord> {
        self.modules
            .iter()
            .find(|module| module.name == name && !module.invalidated)
    }

    pub fn this_descriptor(&self, name: &str) -> Option<&ModuleDescriptor> {
        self.this_mod(name).map(|module| &module.descriptor)
    }

    pub fn this_interface(&self, name: &str) -> Option<&InterfaceDescriptor> {
        self.this_mod(name).map(|module| &module.interface)
    }

    pub fn this_command(&self, module_name: &str, command_name: &str) -> Option<CommandInvocation> {
        let module = self.this_mod(module_name)?;
        let export = module.exports.find(command_name)?;

        if export.kind == ExportKind::Command {
            Some(CommandInvocation {
                module_name: module_name.to_string(),
                command_name: command_name.to_string(),
            })
        } else {
            None
        }
    }

    pub fn load_mod(&mut self, name: &str) -> Result<Vec<String>, LoadError> {
        let index = self
            .modules
            .iter()
            .position(|module| module.name == name && !module.invalidated)
            .ok_or_else(|| LoadError::ModuleNotFound(name.to_string()))?;

        let mut log = Vec::new();
        self.initialize_module(index, &mut log)?;
        Ok(log)
    }

    pub fn invoke_command(&mut self, path: &str) -> Result<Vec<String>, CommandError> {
        let (module_name, command_name) = parse_command_path(path)?;
        let mut log = self.load_mod(module_name).map_err(CommandError::Load)?;

        let command = self
            .this_command(module_name, command_name)
            .ok_or_else(|| CommandError::CommandNotFound(path.to_string()))?;
        log.push(command.render());

        if let Some(action_summary) = self.command_handlers.get(path) {
            log.push(format!("rust-hosted {path}: {action_summary}"));
        }

        Ok(log)
    }

    pub fn describe_interface(&self, name: &str) -> Result<String, LoadError> {
        let interface = self
            .this_interface(name)
            .ok_or_else(|| LoadError::ModuleNotFound(name.to_string()))?;
        Ok(interface.render())
    }

    fn initialize_module(&mut self, index: usize, log: &mut Vec<String>) -> Result<(), LoadError> {
        if self.modules[index].invalidated {
            return Err(LoadError::ModuleInvalidated(self.modules[index].name.clone()));
        }

        if self.modules[index].initialized {
            return Ok(());
        }

        let imports = self.modules[index].imports.clone();
        for import_name in imports {
            let import_index = self
                .modules
                .iter()
                .position(|module| module.name == import_name && !module.invalidated)
                .ok_or_else(|| LoadError::ModuleNotFound(import_name.clone()))?;
            self.initialize_module(import_index, log)?;
        }

        let module = &mut self.modules[index];
        if !module.initialized {
            module.initialized = true;
            log.push(format!("init {} via {}", module.name, module.init_routine));
        }

        Ok(())
    }

    fn build_module_descriptor(
        id: ModuleId,
        name: &str,
        kind: ModuleKind,
        imports: &[String],
        exports: &ExportDirectory,
        init_routine: &str,
    ) -> ModuleDescriptor {
        let base_address = 0x0001_0000_0000_0000u64 + (u64::from(id.0) * 0x0000_0000_0010_0000u64);
        let export_descriptors = exports
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| ExportDescriptor {
                name: entry.name.clone(),
                kind: entry.kind,
                slot_index: index as u32,
                entry_address: base_address + 0x400 + ((index as u64) * 0x20),
            })
            .collect::<Vec<_>>();

        let code_size = 0x100u32 + ((name.len() as u32) * 4);
        let data_size = 0x80u32 + ((imports.len() as u32) * 8);
        let init_bias = (init_routine.len() as u64) & 0xFF;

        ModuleDescriptor {
            id,
            name: name.to_string(),
            kind,
            import_count: imports.len() as u32,
            export_count: export_descriptors.len() as u32,
            code_address: base_address,
            data_address: base_address + 0x2000,
            export_directory_address: base_address + 0x3000,
            init_entry_address: base_address + 0x100 + init_bias,
            code_size,
            data_size,
            exports: export_descriptors,
        }
    }

    fn build_interface_descriptor(
        name: &str,
        imports: &[String],
        exports: &ExportDirectory,
    ) -> InterfaceDescriptor {
        InterfaceDescriptor {
            module_name: name.to_string(),
            imports: imports.to_vec(),
            symbols: exports
                .entries
                .iter()
                .map(|entry| InterfaceSymbolDescriptor {
                    name: entry.name.clone(),
                    kind: entry.kind,
                    exported: true,
                    binding: match entry.kind {
                        ExportKind::Command => "command".to_string(),
                        ExportKind::Procedure => "procedure".to_string(),
                        ExportKind::Type => "type".to_string(),
                        ExportKind::Variable => "variable".to_string(),
                        ExportKind::Constant => "constant".to_string(),
                    },
                })
                .collect(),
        }
    }

    fn retire_active_module(&mut self, name: &str) {
        for module in &mut self.modules {
            if module.name == name && !module.invalidated {
                module.invalidated = true;
            }
        }

        let prefix = format!("{name}.");
        self.command_handlers
            .retain(|key, _| !key.starts_with(&prefix));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    ModuleNotFound(String),
    ModuleInvalidated(String),
}

impl LoadError {
    pub fn render(&self) -> String {
        match self {
            Self::ModuleNotFound(name) => format!("module not found: {name}"),
            Self::ModuleInvalidated(name) => format!("module invalidated: {name}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    InvalidPath(String),
    Load(LoadError),
    CommandNotFound(String),
}

impl CommandError {
    pub fn render(&self) -> String {
        match self {
            Self::InvalidPath(path) => format!("invalid command path: {path}"),
            Self::Load(error) => error.render(),
            Self::CommandNotFound(path) => format!("command not found: {path}"),
        }
    }
}

fn parse_command_path(path: &str) -> Result<(&str, &str), CommandError> {
    let mut parts = path.split('.');
    let Some(module_name) = parts.next() else {
        return Err(CommandError::InvalidPath(path.to_string()));
    };
    let Some(command_name) = parts.next() else {
        return Err(CommandError::InvalidPath(path.to_string()));
    };

    if parts.next().is_some() || module_name.is_empty() || command_name.is_empty() {
        return Err(CommandError::InvalidPath(path.to_string()));
    }

    Ok((module_name, command_name))
}

#[derive(Debug, Clone)]
pub struct InitState {
    pub loaded_services: Vec<String>,
}

impl InitState {
    pub fn bootstrap() -> Self {
        Self {
            loaded_services: vec!["compiler".to_string(), "jit-loader".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct BootstrapReport {
    pub kernel: KernelState,
    pub init: InitState,
    pub compiler: CompilerState,
    pub phases: Vec<BootPhase>,
    pub init_log: Vec<String>,
    pub compiled_modules: Vec<String>,
    pub hosted_modules: Vec<String>,
}

impl BootstrapReport {
    pub fn new() -> Self {
        let mut kernel = KernelState::bootstrap("x86_64-pc-windows-msvc");
        let compiler = ResidentCompiler::bootstrap();
        kernel.register_resident_module("Kernel");
        kernel.register_resident_module("Init");
        let console_module = console::native_module_artifact();
        let host_menus = HostedModuleArtifact::new(
            "HostMenus",
            vec!["Kernel".to_string()],
            ExportDirectory::new(vec![ExportEntry::command("OpenApp")]),
            "HostMenus.bootstrap",
            "Rust-hosted facade until CP HostMenus is available",
            vec![RustCommandHandlerSpec::new(
                "OpenApp",
                "open the outer application shell stub",
            )],
        );
        let system_artifact = compiler.compile_module(
            "System",
            &["Kernel"],
            vec![ExportEntry::procedure("Start")],
            "System.body",
            "MODULE System; BEGIN END System.",
        );
        let init_shell_artifact = compiler.compile_module(
            "InitShell",
            &["Kernel", "System", "HostMenus"],
            vec![ExportEntry::command("EnterShell")],
            "InitShell.body",
            "MODULE InitShell; IMPORT System, HostMenus; BEGIN END InitShell.",
        );

        let hosted_modules = vec![
            format!("{} [{}]", console_module.hosted.name, console_module.hosted.source_summary),
            format!("{} [{}]", host_menus.name, host_menus.source_summary),
        ];

        let compiled_modules = vec![
            format!("{} [{}]", system_artifact.name, system_artifact.source_summary),
            format!(
                "{} [{}]",
                init_shell_artifact.name, init_shell_artifact.source_summary
            ),
        ];

        kernel.register_native_module(console_module);
        kernel.register_hosted_module(host_menus);
        kernel.register_compiled_module(system_artifact);
        kernel.register_compiled_module(init_shell_artifact);

        let init_log = kernel
            .load_mod("InitShell")
            .unwrap_or_else(|error| vec![error.render()]);

        Self {
            kernel,
            init: InitState::bootstrap(),
            compiler: CompilerState::bootstrap(&compiler),
            phases: vec![
                BootPhase::KernelReady,
                BootPhase::InitReady,
                BootPhase::CompilerReady,
                BootPhase::JitReady,
                BootPhase::BaseModulesReady,
            ],
            init_log,
            compiled_modules,
            hosted_modules,
        }
    }

    pub fn render(&self) -> String {
        let modules = self
            .kernel
            .modules
            .iter()
            .filter(|module| !module.invalidated)
            .map(|module| {
                let residency = match module.kind {
                    ModuleKind::ResidentRust => "resident",
                    ModuleKind::JitCompiled => "jit",
                };
                format!("{}:{}:{}", module.id.0, module.name, residency)
            })
            .collect::<Vec<_>>()
            .join(", ");

        let services = self.init.loaded_services.join(", ");
        let compiler = format!(
            "{}:{}",
            self.compiler.service_name,
            if self.compiler.initialized {
                self.compiler.host_language.as_str()
            } else {
                "not-initialized"
            }
        );
        let phases = self
            .phases
            .iter()
            .map(|phase| phase.as_str())
            .collect::<Vec<_>>()
            .join(" -> ");
        let init_log = self.init_log.join(" | ");
        let compiled_modules = self.compiled_modules.join(" | ");
        let hosted_modules = self.hosted_modules.join(" | ");

        format!(
            concat!(
                "newcp-runtime bootstrap report\n",
                "target: {}\n",
                "pointer-width: {}\n",
                "resident-modules: {}\n",
                "init-services: {}\n",
                "compiler-service: {}\n",
            "hosted-modules: {}\n",
                "compiled-modules: {}\n",
                "boot-phases: {}\n",
                "module-init-log: {}\n",
                "status: ready to compile CP modules into memory"
            ),
            self.kernel.target_triple,
            self.kernel.pointer_width_bits,
            modules,
            services,
            compiler,
            hosted_modules,
            compiled_modules,
            phases,
            init_log
        )
    }
}

pub fn bootstrap_report() -> String {
    BootstrapReport::new().render()
}

pub fn bootstrap_and_invoke(command_path: &str) -> String {
    let mut report = BootstrapReport::new();
    match report.kernel.invoke_command(command_path) {
        Ok(log) => format!(
            "{}\ncommand-log: {}",
            report.render(),
            log.join(" | ")
        ),
        Err(error) => format!("{}\ncommand-error: {}", report.render(), error.render()),
    }
}

pub fn bootstrap_and_describe_interface(module_name: &str) -> String {
    let report = BootstrapReport::new();
    match report.kernel.describe_interface(module_name) {
        Ok(interface) => format!("{}\n{}", report.render(), interface),
        Err(error) => format!("{}\ninterface-error: {}", report.render(), error.render()),
    }
}

fn builtin_native_modules() -> Vec<NativeModuleArtifact> {
    vec![console::native_module_artifact()]
}

pub fn native_export_address(module_name: &str, export_name: &str) -> Option<usize> {
    builtin_native_modules()
        .into_iter()
        .find(|artifact| artifact.hosted.name == module_name)
        .and_then(|artifact| artifact.export_address(export_name))
}

/// Runtime trap handler invoked from JIT code via `@__newcp_trap(i32)`.
///
/// `code` follows the ABI defined in `docs/llvm-codegen-design.md`:
/// 1=Assert, 2=NilDeref, 3=ArrayBounds, 4=TypeGuard, 5=CaseFallthrough, other=Halt.
#[unsafe(no_mangle)]
pub extern "C" fn __newcp_trap(code: i32) -> ! {
    let kind = match code {
        1 => "ASSERT failed",
        2 => "NIL pointer dereference",
        3 => "array index out of bounds",
        4 => "type guard failed",
        5 => "CASE fall-through",
        n => return panic_trap(format!("HALT({n})")),
    };
    panic_trap(kind.to_string())
}

fn panic_trap(message: String) -> ! {
    eprintln!("newcp trap: {message}");
    std::process::abort();
}

pub fn runtime_symbol_address(symbol_name: &str) -> Option<usize> {
    if symbol_name == "__newcp_sys_new" {
        return Some(gc::__newcp_sys_new as *const () as usize);
    }
    if symbol_name == "__newcp_trap" {
        return Some(__newcp_trap as *const () as usize);
    }

    let (module_name, export_name) = symbol_name.split_once('.')?;
    native_export_address(module_name, export_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_report_reflects_resident_kernel_and_init() {
        let report = BootstrapReport::new().render();

        assert!(report.contains("target: x86_64-pc-windows-msvc"));
        assert!(report.contains("pointer-width: 64"));
        assert!(report.contains("resident-modules: 1:Kernel:resident, 2:Init:resident"));
        assert!(report.contains("compiler-service: resident-compiler:Rust"));
        assert!(report.contains("hosted-modules: Console [Rust-hosted console I/O facade for tests and JIT execution] | HostMenus [Rust-hosted facade until CP HostMenus is available]"));
        assert!(report.contains("compiled-modules: System [compiled by resident-compiler"));
        assert!(report.contains("boot-phases: kernel-ready -> init-ready -> compiler-ready -> jit-ready -> base-modules-ready"));
        assert!(report.contains("module-init-log: init Kernel via bootstrap | init System via System.body | init HostMenus via HostMenus.bootstrap | init InitShell via InitShell.body"));
        assert!(report.contains("status: ready to compile CP modules into memory"));
    }

    #[test]
    fn bootstrap_report_exposes_console_interface_metadata() {
        let report = BootstrapReport::new();

        let interface = report
            .kernel
            .describe_interface("Console")
            .expect("Console hosted interface should be registered during bootstrap");

        assert!(interface.contains("interface-module: Console"));
        assert!(interface.contains("interface-imports: <none>"));
        assert!(interface.contains("ReadChar:Procedure:procedure"));
        assert!(interface.contains("ReadInt:Procedure:procedure"));
        assert!(interface.contains("WriteChar:Procedure:procedure"));
        assert!(interface.contains("WriteInt:Procedure:procedure"));
        assert!(interface.contains("WriteLn:Procedure:procedure"));
    }

    #[test]
    fn load_mod_initializes_imports_once_in_dependency_order() {
        let mut kernel = KernelState::bootstrap("x86_64-pc-windows-msvc");
        kernel.register_resident_module("Kernel");
        kernel.register_jit_module("B", vec!["Kernel".to_string()], ExportDirectory::default(), "B.body");
        kernel.register_jit_module(
            "A",
            vec!["Kernel".to_string(), "B".to_string()],
            ExportDirectory::default(),
            "A.body",
        );

        let first = kernel.load_mod("A").expect("A should load");
        let second = kernel.load_mod("A").expect("A should remain loaded");

        assert_eq!(first, vec!["init Kernel via bootstrap", "init B via B.body", "init A via A.body"]);
        assert!(second.is_empty());
    }

    #[test]
    fn export_directory_supports_name_lookup() {
        let directory = ExportDirectory::new(vec![
            ExportEntry::command("CompileModule"),
            ExportEntry::procedure("Boot"),
        ]);

        assert!(directory.find("CompileModule").is_some());
        assert_eq!(directory.names(), vec!["Boot", "CompileModule"]);
    }

    #[test]
    fn resident_compiler_produces_artifact_consumable_by_runtime() {
        let compiler = ResidentCompiler::bootstrap();
        let artifact = compiler.compile_module(
            "Shell",
            &["Kernel", "System"],
            vec![ExportEntry::command("Open")],
            "Shell.body",
            "MODULE Shell; IMPORT System; BEGIN END Shell.",
        );
        let mut kernel = KernelState::bootstrap("x86_64-pc-windows-msvc");
        kernel.register_resident_module("Kernel");
        kernel.register_compiled_module(
            compiler.compile_module(
                "System",
                &["Kernel"],
                vec![ExportEntry::procedure("Start")],
                "System.body",
                "MODULE System; BEGIN END System.",
            ),
        );
        kernel.register_compiled_module(artifact);

        let log = kernel.load_mod("Shell").expect("Shell should load");

        assert_eq!(log, vec!["init Kernel via bootstrap", "init System via System.body", "init Shell via Shell.body"]);
    }

    #[test]
    fn compiled_module_registration_lowers_to_64_bit_descriptor() {
        let compiler = ResidentCompiler::bootstrap();
        let artifact = compiler.compile_module(
            "System",
            &["Kernel"],
            vec![ExportEntry::procedure("Start")],
            "System.body",
            "MODULE System; BEGIN END System.",
        );
        let mut kernel = KernelState::bootstrap("x86_64-pc-windows-msvc");
        kernel.register_resident_module("Kernel");
        kernel.register_compiled_module(artifact);

        let descriptor = kernel.this_descriptor("System").expect("System descriptor should exist");

        assert_eq!(descriptor.import_count, 1);
        assert_eq!(descriptor.export_count, 1);
        assert!(descriptor.code_address > u64::from(u32::MAX));
        assert!(descriptor.data_address > u64::from(u32::MAX));
        assert_eq!(descriptor.exports[0].name, "Start");
        assert_eq!(descriptor.exports[0].kind, ExportKind::Procedure);
        assert!(descriptor.exports[0].entry_address > u64::from(u32::MAX));
    }

    #[test]
    fn interface_descriptor_tracks_exported_symbols_and_imports() {
        let compiler = ResidentCompiler::bootstrap();
        let artifact = compiler.compile_module(
            "InitShell",
            &["Kernel", "System", "HostMenus"],
            vec![ExportEntry::command("EnterShell")],
            "InitShell.body",
            "MODULE InitShell; IMPORT System, HostMenus; BEGIN END InitShell.",
        );
        let mut kernel = KernelState::bootstrap("x86_64-pc-windows-msvc");
        kernel.register_compiled_module(artifact);

        let interface = kernel.this_interface("InitShell").expect("interface should exist");

        assert_eq!(interface.imports, vec!["Kernel", "System", "HostMenus"]);
        assert_eq!(interface.symbols.len(), 1);
        assert_eq!(interface.symbols[0].name, "EnterShell");
        assert_eq!(interface.symbols[0].kind, ExportKind::Command);
        assert_eq!(interface.symbols[0].binding, "command");
    }

    #[test]
    fn invoke_command_loads_module_and_appends_command_call() {
        let mut report = BootstrapReport::new();

        let log = report
            .kernel
            .invoke_command("InitShell.EnterShell")
            .expect("InitShell.EnterShell should resolve");

        assert_eq!(log, vec!["invoke InitShell.EnterShell"]);
    }

    #[test]
    fn rust_hosted_module_executes_stub_handler() {
        let mut report = BootstrapReport::new();

        let log = report
            .kernel
            .invoke_command("HostMenus.OpenApp")
            .expect("HostMenus.OpenApp should resolve to a rust-hosted facade");

        assert_eq!(
            log,
            vec![
                "invoke HostMenus.OpenApp",
                "rust-hosted HostMenus.OpenApp: open the outer application shell stub",
            ]
        );
    }

    #[test]
    fn compiled_module_can_replace_rust_hosted_facade() {
        let compiler = ResidentCompiler::bootstrap();
        let mut kernel = KernelState::bootstrap("x86_64-pc-windows-msvc");
        kernel.register_resident_module("Kernel");
        kernel.register_hosted_module(HostedModuleArtifact::new(
            "HostMenus",
            vec!["Kernel".to_string()],
            ExportDirectory::new(vec![ExportEntry::command("OpenApp")]),
            "HostMenus.bootstrap",
            "Rust-hosted facade",
            vec![RustCommandHandlerSpec::new("OpenApp", "stub")],
        ));
        kernel.register_compiled_module(compiler.compile_module(
            "HostMenus",
            &["Kernel"],
            vec![ExportEntry::command("OpenApp")],
            "HostMenus.body",
            "MODULE HostMenus; BEGIN END HostMenus.",
        ));

        let log = kernel
            .invoke_command("HostMenus.OpenApp")
            .expect("compiled HostMenus should replace the rust-hosted facade");
        let module = kernel.this_mod("HostMenus").expect("active HostMenus module should exist");

        assert_eq!(module.kind, ModuleKind::JitCompiled);
        assert_eq!(log, vec!["init Kernel via bootstrap", "init HostMenus via HostMenus.body", "invoke HostMenus.OpenApp"]);
    }

    #[test]
    fn invoke_command_rejects_non_command_export() {
        let mut report = BootstrapReport::new();

        let error = report
            .kernel
            .invoke_command("System.Start")
            .expect_err("System.Start is not a command export");

        assert_eq!(error, CommandError::CommandNotFound("System.Start".to_string()));
    }

    #[test]
    fn invoke_command_rejects_invalid_path() {
        let mut report = BootstrapReport::new();

        let error = report
            .kernel
            .invoke_command("BrokenPath")
            .expect_err("command path should require Module.Command syntax");

        assert_eq!(error, CommandError::InvalidPath("BrokenPath".to_string()));
    }
}
