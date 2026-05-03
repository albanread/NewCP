use std::path::Path;

use newcp_parser::read_source_module;
use newcp_runtime::{
    CompilerService, ExportEntry, ExportDirectory, HostedModuleArtifact, KernelState,
    ResidentCompiler, RustCommandHandlerSpec,
};

pub fn dump_module_graph(path: &Path) -> String {
    match read_source_module(path) {
        Ok(spec) => {
            let imports = if spec.imports.is_empty() {
                "<none>".to_string()
            } else {
                spec.imports.join(", ")
            };
            let dependency_edges = if spec.imports.is_empty() {
                "<none>".to_string()
            } else {
                spec.imports
                    .iter()
                    .map(|import| format!("{} -> {}", spec.name, import))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let initialization_order = if spec.imports.is_empty() {
                spec.name.clone()
            } else {
                spec.imports
                    .iter()
                    .cloned()
                    .chain(std::iter::once(spec.name.clone()))
                    .collect::<Vec<_>>()
                    .join(" -> ")
            };

            format!(
                concat!(
                    "newcp-loader module graph\n",
                    "input: {}\n",
                    "root-module: {}\n",
                    "imports: {}\n",
                    "dependency-edges: {}\n",
                    "initialization-order: {}"
                ),
                path.display(),
                spec.name,
                imports,
                dependency_edges,
                initialization_order
            )
        }
        Err(error) => format!("newcp-loader module graph error\ninput: {}\nerror: {}", path.display(), error),
    }
}

pub fn bootstrap_plan() -> String {
    [
        "newcp-loader bootstrap plan",
        "status: bootstrap sequencing not implemented yet",
        "steps:",
        "  1. start resident kernel",
        "  2. start resident init",
        "  3. expose and initialize resident compiler services",
        "  4. open JIT session",
        "  5. compile first base modules into memory",
        "  6. register modules and run module bodies",
    ]
    .join("\n")
}

pub fn blackbox_like_bootstrap_demo() -> String {
    let mut kernel = KernelState::bootstrap("x86_64-pc-windows-msvc");
    let compiler = ResidentCompiler::bootstrap();
    kernel.register_resident_module("Kernel");
    kernel.register_resident_module("Init");
    kernel.register_hosted_module(HostedModuleArtifact::new(
        "HostMenus",
        vec!["Kernel".to_string()],
        ExportDirectory::new(vec![ExportEntry::command("OpenApp")]),
        "HostMenus.bootstrap",
        "Rust-hosted facade until CP HostMenus is available",
        vec![RustCommandHandlerSpec::new(
            "OpenApp",
            "open the outer application shell stub",
        )],
    ));
    kernel.register_compiled_module(compiler.compile_module(
        "System",
        &["Kernel"],
        vec![ExportEntry::procedure("Start")],
        "System.body",
        "MODULE System; BEGIN END System.",
    ));
    kernel.register_compiled_module(compiler.compile_module(
        "InitShell",
        &["Kernel", "System", "HostMenus"],
        vec![ExportEntry::command("EnterShell")],
        "InitShell.body",
        "MODULE InitShell; IMPORT System, HostMenus; BEGIN END InitShell.",
    ));

    match kernel.load_mod("InitShell") {
        Ok(log) => format!(
            "blackbox-like bootstrap demo\nresident-compiler: {}\nthis_mod(InitShell): {}\ninit-log: {}",
            compiler.service_name(),
            kernel.this_mod("InitShell").map(|module| module.name.as_str()).unwrap_or("missing"),
            log.join(" | ")
        ),
        Err(error) => format!("blackbox-like bootstrap demo\nerror: {}", error.render()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_plan_mentions_compiling_base_modules() {
        let plan = bootstrap_plan();

        assert!(plan.contains("start resident kernel"));
        assert!(plan.contains("start resident init"));
        assert!(plan.contains("expose and initialize resident compiler services"));
        assert!(plan.contains("compile first base modules into memory"));
    }

    #[test]
    fn blackbox_like_bootstrap_demo_initializes_import_chain() {
        let demo = blackbox_like_bootstrap_demo();

        assert!(demo.contains("resident-compiler: resident-compiler"));
        assert!(demo.contains("this_mod(InitShell): InitShell"));
        assert!(demo.contains("init-log: init Kernel via bootstrap | init System via System.body | init HostMenus via HostMenus.bootstrap | init InitShell via InitShell.body"));
    }

    #[test]
    fn module_graph_lists_import_edges() {
        let temp = std::env::temp_dir().join("newcp-loader-test.cp");
        std::fs::write(&temp, "MODULE Demo;\nIMPORT Kernel, System;\nEND Demo.")
            .expect("write test module");

        let dump = dump_module_graph(&temp);
        let _ = std::fs::remove_file(&temp);

        assert!(dump.contains("root-module: Demo"));
        assert!(dump.contains("dependency-edges: Demo -> Kernel, Demo -> System"));
        assert!(dump.contains("initialization-order: Kernel -> System -> Demo"));
    }
}
