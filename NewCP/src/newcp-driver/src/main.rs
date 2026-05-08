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
    #[cfg(feature = "gui")]
    "run-gui",
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
        println!("{}", newcp_loader::bootstrap_report());
        return;
    }

    #[cfg(feature = "gui")]
    if command == "run-gui" {
        let command_path = args.next();
        std::process::exit(run_gui(command_path.as_deref()));
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
    eprintln!("  newcp-driver run-gui [Module.Command]");
    eprintln!("  newcp-driver <dump-command> [--opt <none|less|default|aggressive>] <file>");
    eprintln!();
    eprintln!("commands:");
    for command in COMMANDS {
        eprintln!("  {command}");
    }
}

#[cfg(feature = "gui")]
use std::sync::OnceLock;

/// The spec_bind runtime pointer, stored so the CP worker thread can call
/// `request_stop` when it is done.
#[cfg(feature = "gui")]
static GUI_RUNTIME_PTR: OnceLock<usize> = OnceLock::new();

/// Run the wingui spec_bind host on the main (Win32 message-loop) thread.
/// A background thread bootstraps the CP kernel and invokes App.cp:App.Run,
/// which owns the event loop and window layout via WinSpec/HostWindows.
#[cfg(feature = "gui")]
fn run_gui(command_path: Option<&str>) -> i32 {
    use newcp_runtime::wingui_host::{HostConfig, SpecBindRuntime};

    // If a command was given, store it so the worker can pass it to the kernel.
    // When no command is given the kernel boots and App.Run enters the event loop.
    static GUI_STARTUP_COMMAND: OnceLock<String> = OnceLock::new();
    if let Some(path) = command_path {
        let _ = GUI_STARTUP_COMMAND.set(path.to_owned());
    }

    let runtime = match SpecBindRuntime::new() {
        Some(r) => r,
        None => { eprintln!("[run_gui] SpecBindRuntime::new() failed"); return 1; },
    };
    eprintln!("[run_gui] SpecBindRuntime created OK");

    // Store pointer for the worker thread.
    let _ = GUI_RUNTIME_PTR.set(runtime.as_ptr() as usize);

    // Spawn the CP worker.  It bootstraps the kernel and runs App.Run (or a
    // command-path override if one was provided on the CLI).
    let startup_cmd = GUI_STARTUP_COMMAND.get().cloned()
        .unwrap_or_else(|| "App.Run".to_owned());
    eprintln!("[run_gui] spawning cp_worker_thread with cmd={:?}", startup_cmd);
    std::thread::spawn(move || cp_worker_thread(startup_cmd));

    eprintln!("[run_gui] entering runtime.run() on main thread...");
    let config = HostConfig { title: "NewCP".to_string(), ..Default::default() };
    let exit_code = runtime.run(&config);
    eprintln!("[run_gui] runtime.run() returned exit_code={}", exit_code);
    exit_code
}

/// Worker thread: JIT-compiles and runs the given CP command via LoaderSession.
/// When the command exits it requests the GUI to close.
#[cfg(feature = "gui")]
fn cp_worker_thread(command_path: String) {
    use newcp_runtime::wingui_spec_ffi::WinguiSpecBindRuntime;

    eprintln!("[cp-worker] thread started, creating LoaderSession...");
    let mut session = newcp_loader::LoaderSession::new();
    eprintln!("{}", session.report().render());
    eprintln!("[cp-worker] LoaderSession ready, invoking command: {}", command_path);
    eprintln!("[cp-worker] ensure_command_loaded starting...");
    let loaded = match session.ensure_command_loaded(&command_path) {
        Ok(l) => { eprintln!("[cp-worker] ensure_command_loaded OK, load_log: {}", l.load.load_log.join(" | ")); l }
        Err(err) => { eprintln!("[cp-worker] ensure_command_loaded error: {}", err); return; }
    };
    // Optional dev-time probe: synthesize button-click events into the same
    // EVENT_QUEUE that `on_event` uses, so we can verify the surface pane
    // survives a `clear_log`-style spec republish without needing a human at
    // the keyboard. Comma-separated NEWCP_TEST_INJECT_EVENTS=name1,name2;
    // delay before first inject is NEWCP_TEST_INJECT_DELAY_MS (default 4000),
    // gap between subsequent injects is NEWCP_TEST_INJECT_GAP_MS (default 2500).
    if let Ok(names) = std::env::var("NEWCP_TEST_INJECT_EVENTS") {
        let event_names: Vec<String> = names
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect();
        if !event_names.is_empty() {
            let delay_ms: u64 = std::env::var("NEWCP_TEST_INJECT_DELAY_MS")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(4000);
            let gap_ms: u64 = std::env::var("NEWCP_TEST_INJECT_GAP_MS")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(2500);
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                for (i, name) in event_names.iter().enumerate() {
                    if i > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(gap_ms));
                    }
                    eprintln!("[cp-worker-probe] injecting event #{}: {:?}", i + 1, name);
                    let _ = newcp_runtime::wingui_host::inject_test_event(name, "{}");
                }
            });
        }
    }

    // Optional dev-time probe: open N stub MDI child windows under the
    // main frame to exercise the Phase 2 create_child_window path before
    // the CP-level OpenChildWindow API exists. Requires the main window
    // to have been created with NEWCP_MDI_FRAME=1 (otherwise the create
    // call returns an error from the host side).
    if let Ok(count_str) = std::env::var("NEWCP_TEST_MDI_CHILDREN") {
        if let Ok(count) = count_str.parse::<u32>() {
            if count > 0 {
                let delay_ms: u64 = std::env::var("NEWCP_TEST_MDI_DELAY_MS")
                    .ok().and_then(|v| v.parse().ok()).unwrap_or(3000);
                let gap_ms: u64 = std::env::var("NEWCP_TEST_MDI_GAP_MS")
                    .ok().and_then(|v| v.parse().ok()).unwrap_or(800);
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                    // Main window's id is 1 by construction (the runtime's
                    // next_window_id starts at 1 and increments per window).
                    let parent = 1u64;
                    for i in 1..=count {
                        if i > 1 {
                            std::thread::sleep(std::time::Duration::from_millis(gap_ms));
                        }
                        let title = format!("Document {}", i);
                        let spec = format!(
                            r#"{{"type":"window","body":{{"type":"stack","children":[{{"type":"text","text":"MDI child #{}"}}]}}}}"#,
                            i,
                        );
                        eprintln!("[cp-worker-probe] opening MDI child #{}", i);
                        let result = newcp_runtime::wingui_host::open_test_mdi_child(
                            parent, &title, &spec);
                        eprintln!("[cp-worker-probe] MDI child #{} result: {:?}", i, result);
                    }
                });
            }
        }
    }

    eprintln!("[cp-worker] about to call command_fn (JIT execution)...");
    match session.invoke_command(&command_path) {
        Ok(result) => {
            let mut log = result.load_log;
            log.extend(result.execution_log);
            eprintln!("[cp-worker] command-log: {}", log.join(" | "));
        }
        Err(err) => eprintln!("[cp-worker] command-error: {}", err),
    }
    eprintln!("[cp-worker] command finished");
    drop(loaded);

    // Close the window once the command exits.
    if let Some(&ptr) = GUI_RUNTIME_PTR.get() {
        let r = ptr as *mut WinguiSpecBindRuntime;
        if !r.is_null() {
            unsafe { newcp_runtime::wingui_spec_ffi::wingui_spec_bind_runtime_request_stop(r, 0) };
        }
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
