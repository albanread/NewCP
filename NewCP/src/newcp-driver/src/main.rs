use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::io::Write;

const COMMANDS: &[&str] = &[
    "bootstrap",
    "invoke-command",
    "describe-interface",
    "load-module",
    "check-mod",
    "check-dir",
    #[cfg(windows)]
    "run-igui",
    "dump-tokens",
    "dump-ast",
    "dump-sema",
    "dump-module-graph",
    "dump-cfg",
    "dump-ir",
    "dump-llvm",
    "dump-asm",
    "dump-heap",
];

fn main() {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return;
    };

    if command == "bootstrap" {
        println!("{}", newcp_loader::bootstrap_report());
        return;
    }

    #[cfg(windows)]
    if command == "run-igui" {
        let command_path = args.next();
        std::process::exit(run_igui(command_path.as_deref()));
    }

    if command == "invoke-command" {
        let Some(command_path) = args.next() else {
            eprintln!("missing command path\n");
            print_usage();
            std::process::exit(2);
        };

        println!("{}", invoke_command(&command_path));
        return;
    }

    if command == "dump-heap" {
        let remaining: Vec<String> = args.collect();
        std::process::exit(run_dump_heap(&remaining));
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
        let path = newcp_loader::resolve_module_source(Path::new(&module_ref));
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

    let mut codegen_options = newcp_llvm::CodegenOptions::default();
    let mut remaining_args: Vec<String> = args.collect();
    if let Some(flag_pos) = remaining_args.iter().position(|arg| arg == "--opt") {
        let Some(level) = remaining_args.get(flag_pos + 1) else {
            eprintln!("missing value after --opt\n");
            print_usage();
            std::process::exit(2);
        };
        let Some(opt_level) = newcp_llvm::OptLevel::from_str(level) else {
            eprintln!("invalid --opt value: {level}\n");
            print_usage();
            std::process::exit(2);
        };
        codegen_options.opt_level = opt_level;
        remaining_args.drain(flag_pos..=flag_pos + 1);
    }

    let Some(input_path) = remaining_args.first() else {
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
            println!("{}", newcp_llvm::dump_llvm_with_options(path, &codegen_options));
        }
        "dump-asm" => {
            println!("{}", newcp_llvm::dump_asm_with_options(path, &codegen_options));
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
    eprintln!("  newcp-driver run-igui [Module.Command]           (Windows only)");
    eprintln!("  newcp-driver <dump-command> [--opt <none|less|default|aggressive>] <file>");
    eprintln!("  newcp-driver dump-heap [--counters|--clusters|--types|--roots|--json] [--after <Module.Command>]");
    eprintln!();
    eprintln!("commands:");
    for command in COMMANDS {
        eprintln!("  {command}");
    }
}

/// `dump-heap` subcommand. Optionally bootstraps the loader and runs a
/// command (`--after <Module.Command>`) before snapshotting, so the heap
/// has something interesting to report. Output mode is selected by an
/// optional flag; default is the full text report.
fn run_dump_heap(args: &[String]) -> i32 {
    use newcp_runtime::heap_introspect::{
        take_lite_snapshot, take_snapshot, RenderMode,
    };

    let mut mode = RenderMode::Full;
    let mut emit_json = false;
    let mut after_command: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--counters" => mode = RenderMode::Counters,
            "--clusters" => mode = RenderMode::Clusters,
            "--types" => mode = RenderMode::Types,
            "--roots" => mode = RenderMode::Roots,
            "--json" => emit_json = true,
            "--after" => {
                i += 1;
                let Some(cmd) = args.get(i) else {
                    eprintln!("missing argument to --after\n");
                    print_usage();
                    return 2;
                };
                after_command = Some(cmd.clone());
            }
            other => {
                eprintln!("unknown dump-heap flag: {other}\n");
                print_usage();
                return 2;
            }
        }
        i += 1;
    }

    // Hold the session alive across the snapshot. If we drop it earlier, the
    // OwnedJitModule(s) in `active_executable_images` go with it, freeing
    // every TypeDesc emitted by those modules. Heap blocks still tagged with
    // those TypeDescs would then segfault during the per-block walk. This is
    // the exact lifetime hazard the `RetiredImageDropPredicate` integration
    // is meant to address — until that wires up, the only safe pattern is
    // "session outlives heap reads".
    let _session_guard = match after_command.as_deref() {
        None => None,
        Some(cmd) => {
            let mut session = newcp_loader::LoaderSession::new();
            if let Err(error) = session.invoke_command(cmd) {
                eprintln!("dump-heap --after failed: {error}");
                return 2;
            }
            Some(session)
        }
    };

    // Type catalog + cluster occupancy require the per-block walk; counters /
    // roots-only modes can use the cheaper lite path.
    let snapshot = match mode {
        RenderMode::Counters | RenderMode::Roots => take_lite_snapshot(),
        _ => take_snapshot(),
    };

    if emit_json {
        println!("{}", snapshot.to_json());
    } else {
        print!("{}", snapshot.render(mode));
    }

    drop(_session_guard);
    0
}

/// Run iGui on the main (Win32 message-loop) thread. If a command path
/// is supplied, a background language thread bootstraps the kernel and
/// invokes that command; the GUI thread runs the Win32 message pump.
#[cfg(windows)]
fn run_igui(command_path: Option<&str>) -> i32 {
    install_redit_checker();
    let worker = command_path.map(|cmd| {
        let cmd = cmd.to_owned();
        move || cp_worker_thread(cmd)
    });
    match newcp_runtime::igui::run(worker) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

/// Hand redit a closure that runs parser + sema on the buffer text
/// and returns diagnostics. The runtime crate can't depend on
/// parser/sema directly (they sit above it in the dep graph), so the
/// driver injects this at startup. UI-thread only — calls happen on
/// F7 / after save inside the editor.
#[cfg(windows)]
fn install_redit_checker() {
    use newcp_runtime::igui::Diagnostic;
    newcp_runtime::igui::install_checker(|src| {
        match newcp_parser::parse_module_ast(src) {
            Ok(ast) => {
                let module = newcp_sema::analyze_module_ast(&ast);
                let mut out = Vec::new();
                for d in &module.diagnostics {
                    out.push(Diagnostic {
                        line: d.line,
                        column: d.column,
                        message: d.message.clone(),
                    });
                }
                for proc in &module.procedures {
                    for d in &proc.diagnostics {
                        out.push(Diagnostic {
                            line: d.line,
                            column: d.column,
                            message: format!("{}: {}", proc.name, d.message),
                        });
                    }
                }
                out
            }
            Err(parse_error) => vec![Diagnostic {
                line: 1,
                column: 1,
                message: format!("parse: {parse_error}"),
            }],
        }
    });
}

/// Background language thread for `run-igui`. Bootstraps the loader,
/// invokes the supplied command (which is expected to enter an
/// `iGui.NextEvent` loop), then asks the GUI to close once the command
/// returns.
#[cfg(windows)]
fn cp_worker_thread(command_path: String) {
    eprintln!("[igui-worker] starting LoaderSession for {command_path}");
    let mut session = newcp_loader::LoaderSession::new();
    eprintln!("{}", session.report().render());
    // Catch panics from invoke_command (including ones propagated up
    // from JIT traps via resume_unwind) so we can log them and still
    // close the iGui frame instead of dying silently and leaving the
    // user staring at an empty MDI shell.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        session.invoke_command(&command_path)
    }));
    match result {
        Ok(Ok(result)) => {
            let mut log = result.load_log;
            log.extend(result.execution_log);
            if !log.is_empty() {
                eprintln!("[igui-worker] {}", log.join(" | "));
            }
        }
        Ok(Err(err)) => eprintln!("[igui-worker] command-error: {err}"),
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&'static str>() {
                (*s).to_string()
            } else {
                "<non-string panic payload>".to_string()
            };
            eprintln!("[igui-worker] command-panic: {msg}");
        }
    }
    // Command finished — close the frame.
    newcp_runtime::igui::cp_exports::igui_quit();
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
    let mut session = newcp_loader::LoaderSession::new();
    let result = session.ensure_module_loaded(module_ref)?;
    let source_path = result.graph.root_path.clone();
    let spec = result.graph.root_spec().clone();
    let load_log = result.load_log;

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
        session.report().render(),
        source_path.display(),
        imports,
        exports,
        command_exports,
        load_log.join(" | ")
    );

    if let Some(command_path) = command_path {
        let command_result = session.invoke_command(command_path)?;
        let mut command_log = command_result.load_log;
        command_log.extend(command_result.execution_log);
        output.push_str(&format!("\ncommand-log: {}", command_log.join(" | ")));
    }

    Ok(output)
}

fn describe_interface_from_source(module_ref: &str) -> Result<String, String> {
    let mut session = newcp_loader::LoaderSession::new();
    let requested_module_name = newcp_loader::module_name_from_ref(module_ref);
    let source_details = if newcp_loader::can_resolve_module_source(module_ref) {
        let result = session.ensure_module_loaded(module_ref)?;
        let source_path = result.graph.root_path.clone();
        let spec = result.graph.root_spec().clone();
        let load_log = result.load_log;
        Some((source_path, spec, load_log))
    } else {
        None
    };

    let module_name = source_details
        .as_ref()
        .map(|(_, spec, _)| spec.name.as_str())
        .unwrap_or(requested_module_name.as_str());

    match session.report().kernel.describe_interface(module_name) {
        Ok(interface) => {
            let mut output = session.report().render();
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
        Err(error) => Ok(format!("{}\ninterface-error: {}", session.report().render(), error.render())),
    }
}

fn invoke_command(command_path: &str) -> String {
    if let Some(module_ref) = command_module_ref(command_path) {
        if module_ref.contains(std::path::MAIN_SEPARATOR)
            || module_ref.contains('/')
            || newcp_loader::can_resolve_module_source(module_ref)
        {
            let mut session = newcp_loader::LoaderSession::new();
            return match session.invoke_command(command_path) {
                Ok(result) => {
                    let mut log = result.load_log;
                    log.extend(result.execution_log);
                    format!("{}\ncommand-log: {}", session.report().render(), log.join(" | "))
                }
                Err(error) => format!("{}\ncommand-error: {}", session.report().render(), error),
            };
        }
    }

    newcp_runtime::bootstrap_and_invoke(command_path)
}

fn command_module_ref(command_path: &str) -> Option<&str> {
    if let Some((module_ref, _)) = command_path.rsplit_once("::") {
        return Some(module_ref);
    }

    command_path.split_once('.').map(|(module_name, _)| module_name)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_module_name_to_mod_folder() {
        let path = newcp_loader::resolve_module_source(Path::new("HostMenus"));

        assert_eq!(path, PathBuf::from("Mod").join("HostMenus.cp"));
    }

    #[test]
    fn derives_module_name_from_path_or_module_ref() {
        assert_eq!(newcp_loader::module_name_from_ref("HostMenus"), "HostMenus");
        assert_eq!(newcp_loader::module_name_from_ref("Mod/HostMenus.cp"), "HostMenus");
    }
}
