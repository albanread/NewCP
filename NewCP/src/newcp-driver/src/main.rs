use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::io::Write;

use newcp_parser::{parse_source_module, SourceExportKind};
use newcp_runtime::{BootstrapReport, CompilerService, ExportEntry, ExportKind, ResidentCompiler};

const COMMANDS: &[&str] = &[
    "bootstrap",
    "invoke-command",
    "describe-interface",
    "load-module",
    "check-mod",
    "check-dir",
    "dump-tokens",
    "dump-ast",
    "dump-sema",
    "dump-module-graph",
    "dump-cfg",
    "dump-ir",
    "dump-llvm",
    "dump-asm",
];

fn main() {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return;
    };

    if command == "bootstrap" {
        println!("{}", newcp_runtime::bootstrap_report());
        println!();
        println!("{}", newcp_loader::bootstrap_plan());
        return;
    }

    if command == "invoke-command" {
        let Some(command_path) = args.next() else {
            eprintln!("missing command path\n");
            print_usage();
            std::process::exit(2);
        };

        println!("{}", newcp_runtime::bootstrap_and_invoke(&command_path));
        return;
    }

    if command == "describe-interface" {
        let Some(module_ref) = args.next() else {
            eprintln!("missing module name\n");
            print_usage();
            std::process::exit(2);
        };

        match describe_interface_from_source(&module_ref) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        return;
    }

    if command == "load-module" {
        let Some(module_ref) = args.next() else {
            eprintln!("missing module name or path\n");
            print_usage();
            std::process::exit(2);
        };
        let command_path = args.next();

        match load_module_from_source(&module_ref, command_path.as_deref()) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        return;
    }

    // check-mod and check-dir resolve via the Mod/ folder convention
    if command == "check-mod" {
        let Some(module_ref) = args.next() else {
            eprintln!("missing module name or path\n");
            print_usage();
            std::process::exit(2);
        };
        let path = resolve_module_source(&module_ref);
        let report = newcp_sema::check_module(&path);
        println!("{report}");
        let ok = report.lines().last().map(|l| l == "ok").unwrap_or(false);
        if !ok {
            std::process::exit(1);
        }
        return;
    }

    if command == "check-dir" {
        let Some(dir_ref) = args.next() else {
            eprintln!("missing directory path\n");
            print_usage();
            std::process::exit(2);
        };
        let exit_code = run_check_dir(Path::new(&dir_ref));
        std::process::exit(exit_code);
    }

    let Some(input_path) = args.next() else {
        eprintln!("missing input path\n");
        print_usage();
        std::process::exit(2);
    };

    if !COMMANDS.contains(&command.as_str()) {
        eprintln!("unknown command: {command}\n");
        print_usage();
        std::process::exit(2);
    }

    let path = Path::new(&input_path);
    match command.as_str() {
        "dump-tokens" => {
            println!("{}", newcp_lexer::dump_tokens(path));
        }
        "dump-ast" => {
            println!("{}", newcp_parser::dump_ast(path));
        }
        "dump-sema" => {
            println!("{}", newcp_sema::dump_sema(path));
        }
        "dump-module-graph" => {
            println!("{}", newcp_loader::dump_module_graph(path));
        }
        "dump-cfg" => {
            println!("{}", newcp_ir::dump_cfg(path));
        }
        "dump-ir" => {
            println!("{}", newcp_ir::dump_ir(path));
        }
        "dump-llvm" => {
            println!("{}", newcp_llvm::dump_llvm(path));
        }
        "dump-asm" => {
            println!("{}", newcp_llvm::dump_asm(path));
        }
        _ => unreachable!(),
    }
}

fn print_usage() {
    eprintln!("NewCP driver");
    eprintln!();
    eprintln!("usage:");
    eprintln!("  newcp-driver bootstrap");
    eprintln!("  newcp-driver invoke-command <Module.Command>");
    eprintln!("  newcp-driver describe-interface <Module>");
    eprintln!("  newcp-driver load-module <Module|Path> [Module.Command]");
    eprintln!("  newcp-driver check-mod <Module|Path>");
    eprintln!("  newcp-driver check-dir <dir>");
    eprintln!("  newcp-driver <dump-command> <file>");
    eprintln!();
    eprintln!("commands:");
    for command in COMMANDS {
        eprintln!("  {command}");
    }
}

/// Check every `.cp` file in `dir`, print a report per file, exit 0 if all clean.
fn run_check_dir(dir: &Path) -> i32 {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("check-dir: cannot read {}: {err}", dir.display());
            return 2;
        }
    };

    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|ext| ext.to_str()) == Some("cp"))
        .collect();
    files.sort();

    if files.is_empty() {
        eprintln!("check-dir: no .cp files found in {}", dir.display());
        return 2;
    }

    let mut any_errors = false;
    for path in &files {
        let report = newcp_sema::check_module(path);
        let _ = writeln!(out, "{report}");
        let clean = report.lines().last().map(|l| l == "ok").unwrap_or(false);
        if !clean {
            any_errors = true;
        }
    }

    if any_errors { 1 } else { 0 }
}

fn load_module_from_source(module_ref: &str, command_path: Option<&str>) -> Result<String, String> {
    let mut report = BootstrapReport::new();
    let compiler = ResidentCompiler::bootstrap();
    let (source_path, spec, load_log) = register_source_module(&mut report, &compiler, module_ref)?;
    for entry in &load_log {
        if !report.init_log.iter().any(|existing| existing == entry) {
            report.init_log.push(entry.clone());
        }
    }

    let imports = if spec.imports.is_empty() {
        "<none>".to_string()
    } else {
        spec.imports.join(", ")
    };
    let exports = if spec.exports.is_empty() {
        "<none>".to_string()
    } else {
        spec.exports
            .iter()
            .map(|export| format!("{}:{:?}", export.name, export.kind))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let command_exports = {
        let command_exports = spec.command_exports();
        if command_exports.is_empty() {
            "<none>".to_string()
        } else {
            command_exports.join(", ")
        }
    };

    let mut output = format!(
        "{}\nsource-module: {}\nmodule-imports: {}\nmodule-exports: {}\nmodule-command-exports: {}\nload-log: {}",
        report.render(),
        source_path.display(),
        imports,
        exports,
        command_exports,
        load_log.join(" | ")
    );

    if let Some(command_path) = command_path {
        let command_log = report
            .kernel
            .invoke_command(command_path)
            .map_err(|error| error.render())?;
        output.push_str(&format!("\ncommand-log: {}", command_log.join(" | ")));
    }

    Ok(output)
}

fn describe_interface_from_source(module_ref: &str) -> Result<String, String> {
    let mut report = BootstrapReport::new();
    let compiler = ResidentCompiler::bootstrap();
    let requested_module_name = module_name_from_ref(module_ref);
    let source_details = if can_resolve_module_source(module_ref) {
        let (source_path, spec, load_log) = register_source_module(&mut report, &compiler, module_ref)?;
        Some((source_path, spec, load_log))
    } else {
        None
    };

    let module_name = source_details
        .as_ref()
        .map(|(_, spec, _)| spec.name.as_str())
        .unwrap_or(requested_module_name.as_str());

    match report.kernel.describe_interface(module_name) {
        Ok(interface) => {
            let mut output = report.render();
            if let Some((source_path, spec, load_log)) = source_details {
                let imports = if spec.imports.is_empty() {
                    "<none>".to_string()
                } else {
                    spec.imports.join(", ")
                };
                let exports = if spec.exports.is_empty() {
                    "<none>".to_string()
                } else {
                    spec.exports
                        .iter()
                        .map(|export| format!("{}:{:?}", export.name, export.kind))
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                output.push_str(&format!(
                    "\nsource-module: {}\nmodule-imports: {}\nmodule-exports: {}\nload-log: {}",
                    source_path.display(),
                    imports,
                    exports,
                    load_log.join(" | ")
                ));
            }
            output.push_str(&format!("\n{}", interface));
            Ok(output)
        }
        Err(error) => Ok(format!("{}\ninterface-error: {}", report.render(), error.render())),
    }
}

fn register_source_module(
    report: &mut BootstrapReport,
    compiler: &ResidentCompiler,
    module_ref: &str,
) -> Result<(PathBuf, newcp_parser::SourceModuleSpec, Vec<String>), String> {
    let source_path = resolve_module_source(module_ref);
    let source_text = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read {}: {error}", source_path.display()))?;
    let spec = parse_source_module(&source_text)?;

    let import_refs = spec.imports.iter().map(String::as_str).collect::<Vec<_>>();
    let exports = spec
        .exports
        .iter()
        .map(|export| ExportEntry {
            name: export.name.clone(),
            kind: match export.kind {
                SourceExportKind::Constant => ExportKind::Constant,
                SourceExportKind::Type => ExportKind::Type,
                SourceExportKind::Variable => ExportKind::Variable,
                SourceExportKind::Procedure => ExportKind::Procedure,
                SourceExportKind::Command => ExportKind::Command,
            },
        })
        .collect::<Vec<_>>();
    let artifact = compiler.compile_module(
        &spec.name,
        &import_refs,
        exports,
        &format!("{}.body", spec.name),
        &source_path.display().to_string(),
    );

    report.kernel.register_compiled_module(artifact);
    report
        .hosted_modules
        .retain(|entry| !entry.starts_with(&format!("{} [", spec.name)));
    report
        .compiled_modules
        .retain(|entry| !entry.starts_with(&format!("{} [", spec.name)));
    report.compiled_modules.push(format!(
        "{} [compiled by {} from {}]",
        spec.name,
        compiler.service_name(),
        source_path.display()
    ));

    let load_log = report
        .kernel
        .load_mod(&spec.name)
        .map_err(|error| error.render())?;
    for entry in &load_log {
        if !report.init_log.iter().any(|existing| existing == entry) {
            report.init_log.push(entry.clone());
        }
    }

    Ok((source_path, spec, load_log))
}

fn can_resolve_module_source(module_ref: &str) -> bool {
    resolve_module_source(module_ref).exists()
}

fn module_name_from_ref(module_ref: &str) -> String {
    Path::new(module_ref)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(module_ref)
        .to_string()
}

fn resolve_module_source(module_ref: &str) -> PathBuf {
    let path = PathBuf::from(module_ref);
    if path.exists() {
        return path;
    }

    if path.extension().is_some() {
        Path::new("Mod").join(path)
    } else {
        Path::new("Mod").join(format!("{module_ref}.cp"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_module_name_to_mod_folder() {
        let path = resolve_module_source("HostMenus");

        assert_eq!(path, PathBuf::from("Mod").join("HostMenus.cp"));
    }

    #[test]
    fn derives_module_name_from_path_or_module_ref() {
        assert_eq!(module_name_from_ref("HostMenus"), "HostMenus");
        assert_eq!(module_name_from_ref("Mod\\HostMenus.cp"), "HostMenus");
    }
}
