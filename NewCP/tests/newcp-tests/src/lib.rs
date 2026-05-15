#[cfg(test)]
mod tests {
    // Generated tier-5 probe matrix (see `docs/test_matrix.md` and
    // `src/newcp-test-matrix/`).  Re-run `cargo run -p newcp-test-matrix`
    // to regenerate after editing the manifest.
    mod matrix_generated;

    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn workspace_root() -> PathBuf {
        // CARGO_MANIFEST_DIR = NewCP/tests/newcp-tests
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent() // tests/
            .unwrap()
            .parent() // NewCP/
            .unwrap()
            .to_path_buf()
    }

    /// Load a CP module by `module_ref` (name or path) relative to the workspace root,
    /// call the exported `Module.Proc` procedure (which must have signature `fn() -> i64`
    /// at the C ABI level), and return the integer result.
    ///
    /// Panics if loading fails or the export is not found.
    fn run_function(module_ref: &str, proc_name: &str) -> i64 {
        // Resolve to an absolute path so we don't fight over the process cwd.
        let abs_ref = workspace_root().join(module_ref);
        let abs_ref_str = abs_ref.to_str().expect("workspace path is UTF-8");

        let mut session = newcp_loader::LoaderSession::new();
        session
            .ensure_import_graph_loaded(abs_ref_str)
            .unwrap_or_else(|e| panic!("load {module_ref}: {e}"));

        let module_name = module_ref
            .trim_end_matches(".cp")
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(module_ref)
            .trim_end_matches(".cp");
        let export_path = format!("{module_name}.{proc_name}");
        let address = session
            .active_export_address(module_name, &export_path)
            .unwrap_or_else(|| panic!("export not found: {export_path}"));

        let f: unsafe extern "C" fn() -> i64 = unsafe { std::mem::transmute(address) };
        unsafe { f() }
    }

    /// Like `run_function` but the procedure writes to the console (void return).
    /// Returns the captured console output.
    #[allow(dead_code)]
    fn run_void_function(module_ref: &str, proc_name: &str) -> String {
        let abs_ref = workspace_root().join(module_ref);
        let abs_ref_str = abs_ref.to_str().expect("workspace path is UTF-8");

        newcp_runtime::console::reset();
        newcp_runtime::console::begin_capture();

        let mut session = newcp_loader::LoaderSession::new();
        session
            .ensure_import_graph_loaded(abs_ref_str)
            .unwrap_or_else(|e| panic!("load {module_ref}: {e}"));

        let module_name = module_ref
            .trim_end_matches(".cp")
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(module_ref)
            .trim_end_matches(".cp");
        let export_path = format!("{module_name}.{proc_name}");
        let address = session
            .active_export_address(module_name, &export_path)
            .unwrap_or_else(|| panic!("export not found: {export_path}"));

        let f: unsafe extern "C" fn() = unsafe { std::mem::transmute(address) };
        unsafe { f() };

        let output = newcp_runtime::console::end_capture();
        newcp_runtime::console::reset();
        output
    }

    fn driver_bin() -> PathBuf {
        let bin = workspace_root()
            .join("target")
            .join("debug")
            .join(if cfg!(windows) { "newcp-driver.exe" } else { "newcp-driver" });

        let status = Command::new("cargo")
            .args(["build", "-p", "newcp-driver"])
            .current_dir(workspace_root())
            .status()
            .expect("failed to run cargo build for newcp-driver");
        assert!(status.success(), "cargo build -p newcp-driver failed");
        bin
    }

    /// Run `check-mod <module>` from the workspace root and return (stdout, exit_code).
    fn check_mod(module: &str) -> (String, i32) {
        let out = Command::new(driver_bin())
            .args(["check-mod", module])
            .current_dir(workspace_root())
            .output()
            .expect("failed to spawn driver binary");

        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let code = out.status.code().unwrap_or(-1);
        (stdout, code)
    }

    /// Dump LLVM IR for a CP module at `--opt none`.
    ///
    /// These tests grep the IR for shape-level patterns (named struct GEPs,
    /// per-field stores, vtable globals, direct calls, etc.) that LLVM's
    /// default `-O2` pass pipeline would inline, hoist, or rewrite into byte-
    /// offset GEPs and memset/memcpy intrinsics. The unoptimized form is the
    /// stable surface the dump-llvm tests are written against.
    fn dump_llvm(path: &str) -> (String, i32) {
        let out = Command::new(driver_bin())
            .args(["dump-llvm", "--opt", "none", path])
            .current_dir(workspace_root())
            .output()
            .expect("failed to spawn driver binary");

        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let code = out.status.code().unwrap_or(-1);
        (stdout, code)
    }

    fn dump_ir(path: &str) -> (String, i32) {
        let out = Command::new(driver_bin())
            .args(["dump-ir", path])
            .current_dir(workspace_root())
            .output()
            .expect("failed to spawn driver binary");

        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let code = out.status.code().unwrap_or(-1);
        (stdout, code)
    }

    /// Like [`dump_llvm`], but for an inline source string written to a
    /// temporary file. Also pinned to `--opt none` for the same reason.
    fn dump_llvm_source(file_name: &str, source: &str) -> (String, i32) {
        let path = std::env::temp_dir().join(file_name);
        std::fs::write(&path, source).expect("failed to write temporary source module");
        let out = Command::new(driver_bin())
            .args([
                "dump-llvm",
                "--opt",
                "none",
                path.to_str().expect("temporary source path should be UTF-8"),
            ])
            .current_dir(workspace_root())
            .output()
            .expect("failed to spawn driver binary");
        let _ = std::fs::remove_file(&path);

        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let code = out.status.code().unwrap_or(-1);
        (stdout, code)
    }

    /// Run `dump-heap` with the given mode flags from the workspace root.
    fn dump_heap(extra: &[&str]) -> (String, i32) {
        let mut argv = vec!["dump-heap"];
        argv.extend_from_slice(extra);
        let out = Command::new(driver_bin())
            .args(&argv)
            .current_dir(workspace_root())
            .output()
            .expect("failed to spawn driver binary");
        let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
        combined.push_str(&String::from_utf8_lossy(&out.stderr));
        let code = out.status.code().unwrap_or(-1);
        (combined, code)
    }

    /// Run `invoke-command <cmd>` from the workspace root and return (stdout+stderr, exit_code).
    fn invoke_command(cmd: &str) -> (String, i32) {
        let out = Command::new(driver_bin())
            .args(["invoke-command", cmd])
            .current_dir(workspace_root())
            .output()
            .expect("failed to spawn driver binary");

        let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
        combined.push_str(&String::from_utf8_lossy(&out.stderr));
        let code = out.status.code().unwrap_or(-1);
        (combined, code)
    }

    #[test]
    fn check_mod_empty_is_clean() {
        let (output, code) = check_mod("Mod/Tests/Empty.cp");
        assert_eq!(code, 0, "expected exit 0 for Empty.cp\noutput:\n{output}");
        assert!(
            output.trim_end().ends_with("ok"),
            "expected 'ok' in output for Empty.cp\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_vars_uses_module_globals() {
        let (output, code) = dump_llvm("Mod/Tests/Vars.cp");
        assert_eq!(code, 0, "expected exit 0 for Vars.cp\noutput:\n{output}");
        // All mutable globals are now collected into a single @Module.Data struct.
        assert!(
            output.contains("%Vars.Data = type"),
            "expected @Module.Data struct type declaration\noutput:\n{output}"
        );
        assert!(
            output.contains("@Vars.Data = global %Vars.Data zeroinitializer"),
            "expected @Module.Data zeroinitialiser\noutput:\n{output}"
        );
        // Stores go through GEP into the struct rather than named flat globals.
        assert!(
            output.contains("store i1 false, ptr getelementptr inbounds (%Vars.Data"),
            "expected GEP-based store for boolean field inside @Vars.Data\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_ir_records_uses_gep_for_by_value_record_fields() {
        let (output, code) = dump_ir("Mod/Tests/Records.cp");
        assert_eq!(code, 0, "expected exit 0 for Records.cp\noutput:\n{output}");
        assert!(
            output.contains("proc *Width (r: named:Rect) -> i64")
                && output.contains("t0 : ptr<i64> = gep r, 2")
                && output.contains("t2 : ptr<i64> = gep r, 0"),
            "expected Width to lower record field access through typed GEPs\noutput:\n{output}"
        );
        assert!(
            !output.contains("load r.right") && !output.contains("load r.left"),
            "expected by-value record fields not to lower as unresolved dotted imports\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_records_uses_matching_struct_geps() {
        let (output, code) = dump_llvm("Mod/Tests/Records.cp");
        assert_eq!(code, 0, "expected exit 0 for Records.cp\noutput:\n{output}");
        assert!(
            output.contains("define void @SetPoint(ptr %0, i64 %1, i64 %2)")
                && output.contains("getelementptr inbounds %Point, ptr %p_ref, i32 0, i32 0")
                && output.contains("getelementptr inbounds %Point, ptr %p_ref1, i32 0, i32 1"),
            "expected Point procedures to use %Point GEPs\noutput:\n{output}"
        );
        // Value-mode record params are passed as `ptr` at the C ABI
        // level — the call site emits `designator_addr(arg)` and the
        // callee prologue memmoves the bytes into a stack-local copy
        // (`alloca` + `llvm.memmove` near the entry).  Subsequent
        // field accesses GEP into that local copy.  This is the
        // CP §8.1 private-copy contract; struct-by-value at the LLVM
        // signature level would be an ABI mismatch with the call
        // site.
        assert!(
            output.contains("define i1 @Contains(ptr %0, ptr %1)")
                && output.contains("getelementptr inbounds %Point, ptr %p, i32 0, i32 0")
                && output.contains("getelementptr inbounds %Rect, ptr %r, i32 0, i32 0")
                && output.contains("call void @llvm.memmove"),
            "expected value-mode record params lowered to ptr with prologue memmove\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_const_str_emits_private_string_global() {
        let (output, code) = dump_llvm("Mod/Tests/Strs.cp");
        assert_eq!(code, 0, "expected exit 0 for Strs.cp\noutput:\n{output}");
        assert!(
            output.contains("@.str.0 = private constant [6 x i8] c\"hello\\00\""),
            "expected private null-terminated string constant\noutput:\n{output}"
        );
        // Open-array param ABI: pointer + hidden length (literal "hello" = 5 chars + NUL = 6).
        assert!(
            output.contains("call void @StrBase.Print(ptr @.str.0, i64 6)"),
            "expected ConstStr passed as (ptr, length) pair to open-array param\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_calls_emits_direct_calls() {
        let (output, code) = dump_llvm("Mod/Tests/Calls.cp");
        assert_eq!(code, 0, "expected exit 0 for Calls.cp\noutput:\n{output}");
        assert!(
            output.contains("%t1 = call i64 @AddOne(i64 %t0)"),
            "expected first direct call to AddOne\noutput:\n{output}"
        );
        assert!(
            output.contains("%t2 = call i64 @AddOne(i64 %t1)"),
            "expected nested direct call to AddOne\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_import_use_emits_imported_calls() {
        let (output, code) = dump_llvm("Mod/Tests/ImportUse.cp");
        assert_eq!(code, 0, "expected exit 0 for ImportUse.cp\noutput:\n{output}");
        assert!(
            output.contains("declare i64 @ImportBase.AddOne(i64)"),
            "expected imported function declaration\noutput:\n{output}"
        );
        assert!(
            output.contains("%t1 = call i64 @ImportBase.AddOne(i64 %t0)"),
            "expected first imported call\noutput:\n{output}"
        );
        assert!(
            output.contains("%t2 = call i64 @ImportBase.AddOne(i64 %t1)"),
            "expected nested imported call\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_var_use_passes_pointer_for_var_param() {
        let (output, code) = dump_llvm("Mod/Tests/VarUse.cp");
        assert_eq!(code, 0, "expected exit 0 for VarUse.cp\noutput:\n{output}");
        assert!(
            output.contains("declare void @VarBase.Bump(ptr)"),
            "expected imported VAR callee declaration to take a pointer\noutput:\n{output}"
        );
        assert!(
            output.contains("call void @VarBase.Bump(ptr %x)"),
            "expected VAR argument to be passed by address\noutput:\n{output}"
        );
    }

    #[test]
    fn imported_exported_global_updates_shared_storage() {
        let temp_root = std::env::temp_dir().join(format!(
            "newcp-exported-global-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_root).expect("failed to create temporary module dir");

        let counter_path = temp_root.join("Counter.cp");
        let counter_user_path = temp_root.join("CounterUser.cp");

        std::fs::write(
            &counter_path,
            concat!(
                "MODULE Counter;\n",
                "  VAR n*: INTEGER;\n",
                "  PROCEDURE Bump*;\n",
                "  BEGIN INC(n) END Bump;\n",
                "BEGIN n := 0 END Counter.\n"
            ),
        )
        .expect("failed to write Counter.cp");
        std::fs::write(
            &counter_user_path,
            concat!(
                "MODULE CounterUser;\n",
                "  IMPORT Counter;\n",
                "  PROCEDURE Run*(): INTEGER;\n",
                "  BEGIN\n",
                "    Counter.Bump;\n",
                "    RETURN Counter.n\n",
                "  END Run;\n",
                "END CounterUser.\n"
            ),
        )
        .expect("failed to write CounterUser.cp");

        let mut session = newcp_loader::LoaderSession::new();
        session
            .ensure_import_graph_loaded(counter_user_path.to_str().expect("temp path should be UTF-8"))
            .unwrap_or_else(|e| panic!("load CounterUser: {e}"));

        assert!(
            session.active_export_address("Counter", "Counter.n").is_some(),
            "expected exported variable address for Counter.n"
        );

        let address = session
            .active_export_address("CounterUser", "CounterUser.Run")
            .unwrap_or_else(|| panic!("export not found: CounterUser.Run"));
        let run: unsafe extern "C" fn() -> i64 = unsafe { std::mem::transmute(address) };
        let result = unsafe { run() };
        assert_eq!(result, 1, "expected imported Counter.n to observe Bump update");

        let _ = std::fs::remove_file(counter_path);
        let _ = std::fs::remove_file(counter_user_path);
        let _ = std::fs::remove_dir_all(temp_root);
    }

    #[test]
    fn dump_llvm_console_module_emits_imported_console_calls() {
        let (output, code) = dump_llvm_source(
            "newcp-console-probe.cp",
            concat!(
                "MODULE Demo;\n",
                "IMPORT Console;\n",
                "VAR x: INTEGER; ch: CHAR;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Console.WriteInt(42);\n",
                "  Console.WriteChar(41X);\n",
                "  Console.WriteLn;\n",
                "  Console.ReadInt(x);\n",
                "  Console.ReadChar(ch)\n",
                "END Run;\n",
                "END Demo."
            ),
        );
        assert_eq!(code, 0, "expected exit 0 for Console probe\noutput:\n{output}");
        assert!(
            output.contains("declare void @Console.WriteInt(i64)"),
            "expected imported Console.WriteInt declaration\noutput:\n{output}"
        );
        assert!(
            output.contains("declare void @Console.WriteChar(i32)"),
            "expected imported Console.WriteChar declaration\noutput:\n{output}"
        );
        assert!(
            output.contains("declare void @Console.WriteLn()"),
            "expected imported Console.WriteLn declaration\noutput:\n{output}"
        );
        assert!(
            output.contains("declare void @Console.ReadInt(ptr)"),
            "expected imported Console.ReadInt VAR declaration\noutput:\n{output}"
        );
        assert!(
            output.contains("declare void @Console.ReadChar(ptr)"),
            "expected imported Console.ReadChar VAR declaration\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_system_probe_emits_raw_address_ops() {
        let (output, code) = dump_llvm_source(
            "newcp-system-probe.cp",
            concat!(
                "MODULE Demo;\n",
                "IMPORT SYSTEM;\n",
                "TYPE Raw = RECORD [untagged] value: INTEGER END;\n",
                "TYPE RawPtr = POINTER [untagged] TO Raw;\n",
                "VAR addr: INTEGER; p: RawPtr;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  addr := SYSTEM.ADR(addr);\n",
                "  addr := SYSTEM.LSH(addr, 1);\n",
                "  SYSTEM.PUT(addr, 1);\n",
                "  SYSTEM.NEW(p, 64)\n",
                "END Run;\n",
                "END Demo.\n"
            ),
        );

        assert_eq!(code, 0, "expected exit 0 for SYSTEM probe\noutput:\n{output}");
        assert!(
            output.contains("ptrtoint (ptr") && output.contains("to i64)"),
            "expected SYSTEM.ADR to lower through ptrtoint to i64\noutput:\n{output}"
        );
        assert!(
            output.contains("%lsh.left.value = shl i64 %t1, 1"),
            "expected SYSTEM.LSH to emit integer shift\noutput:\n{output}"
        );
        assert!(
            output.contains("%rawptr = inttoptr i64 %t3 to ptr"),
            "expected SYSTEM.PUT to lower through inttoptr\noutput:\n{output}"
        );
        assert!(
            output.contains("%t4 = call ptr @__newcp_sys_new(i64 64)"),
            "expected SYSTEM.NEW to call the runtime helper\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_system_move_emits_memmove() {
        let (output, code) = dump_llvm_source(
            "newcp-system-move.cp",
            concat!(
                "MODULE Demo;\n",
                "IMPORT SYSTEM;\n",
                "VAR src, dst: INTEGER;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  SYSTEM.MOVE(src, dst, 8)\n",
                "END Run;\n",
                "END Demo.\n"
            ),
        );

        assert_eq!(code, 0, "expected exit 0 for SYSTEM.MOVE probe\noutput:\n{output}");
        assert!(
            output.contains("call void @llvm.memmove.p0.p0.i64"),
            "expected SYSTEM.MOVE to lower to llvm.memmove\noutput:\n{output}"
        );
        assert!(
            output.contains("inttoptr i64 %t0 to ptr"),
            "expected destination address to lower through inttoptr\noutput:\n{output}"
        );
        assert!(
            output.contains("inttoptr i64 %t1 to ptr"),
            "expected source address to lower through inttoptr\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_system_val_emits_bitcast_or_noop_reinterpret() {
        let (output, code) = dump_llvm_source(
            "newcp-system-val.cp",
            concat!(
                "MODULE Demo;\n",
                "IMPORT SYSTEM;\n",
                "VAR x, y: INTEGER;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  x := SYSTEM.VAL(INTEGER, y)\n",
                "END Run;\n",
                "END Demo.\n"
            ),
        );

        assert_eq!(code, 0, "expected exit 0 for SYSTEM.VAL probe\noutput:\n{output}");
        assert!(
            output.contains("%t0 = load i64, ptr") || output.contains("%bitcast"),
            "expected SYSTEM.VAL to materialize its source value\noutput:\n{output}"
        );
        assert!(
            output.contains("store i64 %t0, ptr") || output.contains("store i64 %bitcast, ptr"),
            "expected SYSTEM.VAL result to flow into the destination store\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_system_rot_emits_funnel_shift_intrinsics() {
        let (output, code) = dump_llvm_source(
            "newcp-system-rot.cp",
            concat!(
                "MODULE Demo;\n",
                "IMPORT SYSTEM;\n",
                "VAR x, y: INTEGER;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  x := SYSTEM.ROT(y, 1)\n",
                "END Run;\n",
                "END Demo.\n"
            ),
        );

        assert_eq!(code, 0, "expected exit 0 for SYSTEM.ROT probe\noutput:\n{output}");
        assert!(
            output.contains("call i64 @llvm.fshl.i64"),
            "expected SYSTEM.ROT to use llvm.fshl\noutput:\n{output}"
        );
        assert!(
            output.contains("call i64 @llvm.fshr.i64"),
            "expected SYSTEM.ROT to use llvm.fshr\noutput:\n{output}"
        );
        assert!(
            output.contains("%rot.result = select i1 false"),
            "expected SYSTEM.ROT to select between left and right rotation paths\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_system_typ_fails_explicitly_until_tagged_abi_exists() {
        let (output, code) = dump_llvm_source(
            "newcp-system-typ.cp",
            concat!(
                "MODULE Demo;\n",
                "IMPORT SYSTEM;\n",
                "TYPE Base = RECORD x: INTEGER END;\n",
                "VAR b: Base; tag: INTEGER;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  tag := SYSTEM.TYP(b)\n",
                "END Run;\n",
                "END Demo.\n"
            ),
        );

        assert_eq!(code, 0, "expected driver command to complete for SYSTEM.TYP probe\noutput:\n{output}");
        assert!(
            output.contains("unsupported at emit_instr: TypTag requires tagged-record TypeDesc lowering and heap/header ABI support"),
            "expected SYSTEM.TYP to fail explicitly at the TypTag backend boundary\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_unary_ops_emit_neg_and_not() {
        let (output, code) = dump_llvm_source(
            "newcp-unary-probe.cp",
            concat!(
                "MODULE Demo;\n",
                "VAR x, y: INTEGER; b: BOOLEAN;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  x := -y;\n",
                "  b := ~b\n",
                "END Run;\n",
                "END Demo.\n"
            ),
        );

        assert_eq!(code, 0, "expected exit 0 for unary probe\noutput:\n{output}");
        assert!(
            output.contains("%neg = sub i64 0, %t0"),
            "expected unary minus to emit integer negation\noutput:\n{output}"
        );
        assert!(
            output.contains("%not = xor i1 %t2, true"),
            "expected boolean not to emit a logical inversion\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_set_in_emits_bit_test() {
        let (output, code) = dump_llvm_source(
            "newcp-set-in-probe.cp",
            concat!(
                "MODULE Demo;\n",
                "VAR s: SET; x: INTEGER; result: BOOLEAN;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  result := x IN s\n",
                "END Run;\n",
                "END Demo.\n"
            ),
        );

        assert_eq!(code, 0, "expected exit 0 for SET IN probe\noutput:\n{output}");
        assert!(
            output.contains("in.shr"),
            "expected IN to emit a logical right shift\noutput:\n{output}"
        );
        assert!(
            output.contains("in.and"),
            "expected IN to mask the shifted bit\noutput:\n{output}"
        );
        assert!(
            output.contains("in.ne"),
            "expected IN to compare the masked bit against zero\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_type_check_emits_runtime_type_test() {
        let (output, code) = dump_llvm_source(
            "newcp-typecheck-probe.cp",
            concat!(
                "MODULE Demo;\n",
                "TYPE Base = RECORD x: INTEGER END;\n",
                "TYPE Sub = RECORD (Base) y: INTEGER END;\n",
                "VAR b: POINTER TO Base;\n",
                "PROCEDURE Run*;\n",
                "VAR result: BOOLEAN;\n",
                "BEGIN\n",
                "  result := b IS Sub\n",
                "END Run;\n",
                "END Demo.\n"
            ),
        );

        assert_eq!(code, 0, "expected exit 0 for IS probe\noutput:\n{output}");
        assert!(
            output.contains("declare i1 @__newcp_type_test(ptr, ptr)"),
            "expected __newcp_type_test to be declared\noutput:\n{output}"
        );
        assert!(
            output.contains("@Sub.desc"),
            "expected TypeDesc global for Sub\noutput:\n{output}"
        );
        assert!(
            output.contains("%typetest = call i1 @__newcp_type_test"),
            "expected IS expression to call the type test helper\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_type_test_terminator_emits_conditional_branch() {
        let (output, code) = dump_llvm_source(
            "newcp-typetest-term-probe.cp",
            concat!(
                "MODULE Demo;\n",
                "TYPE Base = RECORD x: INTEGER END;\n",
                "TYPE Sub = RECORD (Base) y: INTEGER END;\n",
                "VAR b: POINTER TO Base;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  WITH b: Sub DO\n",
                "    b.y := 1\n",
                "  END\n",
                "END Run;\n",
                "END Demo.\n"
            ),
        );

        assert_eq!(code, 0, "expected exit 0 for WITH/IS probe\noutput:\n{output}");
        assert!(
            output.contains("declare i1 @__newcp_type_test(ptr, ptr)"),
            "expected __newcp_type_test to be declared\noutput:\n{output}"
        );
        assert!(
            output.contains("br i1 %typetest"),
            "expected TypeTest terminator to produce a conditional branch\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_pointers_field_access_through_pointer_alias() {
        // Verifies that field access through a POINTER TO record type alias:
        //   DataPtr = POINTER TO Data
        //   GetValue(d: DataPtr): INTEGER → RETURN d.value
        // emits a GEP against the correct struct (%Data), not an opaque fallback,
        // and that pointer NIL checks emit `icmp ne ptr _, null`.
        let (output, code) = dump_llvm("Mod/Tests/Pointers.cp");
        assert_eq!(code, 0, "expected exit 0 for Pointers.cp\noutput:\n{output}");
        // GEP into %Data for field 0 (value: INTEGER)
        assert!(
            output.contains("getelementptr inbounds %Data, ptr"),
            "expected GEP into %%Data for field access\noutput:\n{output}"
        );
        // NIL check should be icmp ne ptr
        assert!(
            output.contains("icmp ne ptr"),
            "expected pointer NIL check to use icmp ne ptr\noutput:\n{output}"
        );
        // NEW(d) should now go through __newcp_new_rec with the
        // record's TypeDesc — every record type gets a TypeDesc so
        // Kernel.TypeOf, IS-tests, and GC tracing all work uniformly.
        assert!(
            output.contains("call ptr @__newcp_new_rec(ptr @Data.desc)"),
            "expected NEW(d) to emit call to __newcp_new_rec with @Data.desc\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_case_emits_full_arm_chain() {
        // Verifies that CASE statements with multiple arms emit a proper test chain:
        // each arm's labels are tested in a separate block, with a fall-through to the
        // next arm's tests on miss.  Also verifies that CASE ELSE (without a match) is
        // reachable and that WITH arms resolve imported record fields correctly.
        let (output, code) = dump_llvm("Mod/Tests/CaseWith.cp");
        assert_eq!(code, 0, "expected exit 0 for CaseWith.cp\noutput:\n{output}");

        // Sides: three arms tested in sequence.  Each arm's body stores a literal,
        // so we expect stores for 0, 3, 4, and -1 (the ELSE value).
        assert!(
            output.contains("store i64 0,"),
            "expected Circle arm body (store 0) in Sides\noutput:\n{output}"
        );
        assert!(
            output.contains("store i64 3,"),
            "expected Triangle arm body (store 3) in Sides\noutput:\n{output}"
        );
        assert!(
            output.contains("store i64 4,"),
            "expected Square arm body (store 4) in Sides\noutput:\n{output}"
        );
        assert!(
            output.contains("store i64 -1,"),
            "expected ELSE arm body (store -1) in Sides\noutput:\n{output}"
        );

        // CharClass: three range-test arms with comparisons for 'a'..'z', 'A'..'Z',
        // '0'..'9' should produce at least six icmp instructions.
        let icmp_count = output.matches("icmp").count();
        assert!(
            icmp_count >= 6,
            "expected at least 6 icmp instructions for range-test arms in CharClass, got {icmp_count}\noutput:\n{output}"
        );

        // Describe (WITH statement): TypeExt.Bird and Fish struct types must be
        // declared, and their fields accessed via typed GEP.
        assert!(
            output.contains("%TypeExt.Bird = type"),
            "expected %%TypeExt.Bird struct declaration for WITH arm\noutput:\n{output}"
        );
        assert!(
            output.contains("%TypeExt.Fish = type"),
            "expected %%TypeExt.Fish struct declaration for WITH arm\noutput:\n{output}"
        );
        assert!(
            output.contains("getelementptr inbounds %TypeExt.Bird,"),
            "expected GEP into %%TypeExt.Bird for canFly field\noutput:\n{output}"
        );
        assert!(
            output.contains("getelementptr inbounds %TypeExt.Fish,"),
            "expected GEP into %%TypeExt.Fish for fins field\noutput:\n{output}"
        );
        assert!(
            output.contains("getelementptr inbounds %TypeExt.Animal,"),
            "expected GEP into %%TypeExt.Animal for legs field in ELSE arm\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_typeext_inherited_field_access() {
        // MakeBird(VAR b: Bird; canFly: BOOLEAN)
        //   b.legs   := 2      -- inherited from Animal at Bird field index 0
        //   b.canFly := canFly -- Bird's own field at index 1
        // Without the flatten_sem_type_fields fix, b.legs fell back to the opaque
        // %"field:legs" alloca and canFly landed at index 0 instead of 1.
        let (output, code) = dump_llvm("Mod/Tests/TypeExt.cp");
        assert_eq!(code, 0, "expected exit 0 for TypeExt.cp\noutput:\n{output}");
        // struct layout: %Bird = { i64, i1 }
        assert!(
            output.contains("%Bird = type { i64, i1 }"),
            "expected Bird struct with inherited i64 legs and own i1 canFly\noutput:\n{output}"
        );
        // b.legs := 2 → gep %Bird index 0
        assert!(
            output.contains("getelementptr inbounds %Bird, ptr %b_ref, i32 0, i32 0"),
            "expected b.legs store to use GEP index 0 into %%Bird\noutput:\n{output}"
        );
        // b.canFly := canFly → gep %Bird index 1
        assert!(
            output.contains("getelementptr inbounds %Bird, ptr %b_ref1, i32 0, i32 1"),
            "expected b.canFly store to use GEP index 1 into %%Bird\noutput:\n{output}"
        );
        // No opaque fallback field reference should remain
        assert!(
            !output.contains("%\"field:"),
            "expected no opaque field fallback references in TypeExt output\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_loops_emit_back_edges_odd_ash() {
        // SumDown: REPEAT/UNTIL with a back-edge (bb2 → bb2).
        // PopCount: REPEAT/UNTIL with ODD(x) → and + icmp ne, and ASH(x,-1) → ashr.
        // IndexOf: LOOP/EXIT with two EXIT branches.
        // CollatzLen: LOOP with ODD check, 3n+1 arm, ASH halving.
        let (output, code) = dump_llvm("Mod/Tests/Loops.cp");
        assert_eq!(code, 0, "expected exit 0 for Loops.cp\noutput:\n{output}");

        // SumDown: REPEAT/UNTIL produces a loop with a back-edge.
        // The loop body block should appear as a predecessor of itself.
        assert!(
            output.contains("br label %bb2") && output.contains("preds = %bb2"),
            "expected SumDown to produce a REPEAT back-edge\noutput:\n{output}"
        );

        // ODD(x) expands to (x & 1) != 0 — expect 'and i64 ... 1' + 'icmp ne'.
        assert!(
            output.contains("and i64 %") && output.contains("icmp ne i64 %and"),
            "expected ODD(x) to expand to bitwise and + icmp ne\noutput:\n{output}"
        );

        // ASH(x, -1) expands to arithmetic right shift — expect ashr.
        assert!(
            output.contains("ashr i64 %"),
            "expected ASH(x, -1) to emit arithmetic right shift\noutput:\n{output}"
        );

        // CollatzLen: 3 * n + 1 arm should produce mul + add.
        assert!(
            output.contains("mul i64 3,") && output.contains("add i64 %imul"),
            "expected CollatzLen 3n+1 branch to emit mul then add\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_methods_emits_vtable_and_type_desc() {
        let (output, code) = dump_llvm("Mod/Tests/Methods.cp");
        assert_eq!(code, 0, "expected exit 0 for Methods.cp\noutput:\n{output}");

        // Bound procedures compiled with qualified ReceiverType_MethodName names.
        assert!(
            output.contains("@Shape_GetX"),
            "expected Shape_GetX function\noutput:\n{output}"
        );
        assert!(
            output.contains("@Circle_GetX"),
            "expected Circle_GetX function\noutput:\n{output}"
        );
        assert!(
            output.contains("@Circle_GetR"),
            "expected Circle_GetR function\noutput:\n{output}"
        );
        assert!(
            output.contains("@MakeCircle"),
            "expected MakeCircle function\noutput:\n{output}"
        );

        // Vtable arrays emitted for both types.
        assert!(
            output.contains("@Shape.vtable"),
            "expected Shape.vtable global\noutput:\n{output}"
        );
        assert!(
            output.contains("@Circle.vtable"),
            "expected Circle.vtable global\noutput:\n{output}"
        );

        // Circle vtable has 3 slots; slot 1 is the inherited Shape_GetY.
        assert!(
            output.contains("[3 x ptr]") && output.contains("@Shape_GetY"),
            "expected Circle vtable to have 3 slots and inherit Shape_GetY\noutput:\n{output}"
        );

        // TypeDesc constants emitted.
        assert!(
            output.contains("@Shape.desc"),
            "expected Shape.desc TypeDesc constant\noutput:\n{output}"
        );
        assert!(
            output.contains("@Circle.desc"),
            "expected Circle.desc TypeDesc constant\noutput:\n{output}"
        );

        // Circle.desc links to Shape.desc as its base.
        assert!(
            output.contains("ptr @Shape.desc"),
            "expected Circle.desc base to point to Shape.desc\noutput:\n{output}"
        );

        // vtable_len fields: Shape=2, Circle=3.
        assert!(
            output.contains("i64 2") && output.contains("i64 3"),
            "expected Shape vtable_len=2 and Circle vtable_len=3\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_arrays_emit_index_gep() {
        let (output, code) = dump_llvm("Mod/Tests/Arrays.cp");
        assert_eq!(code, 0, "expected exit 0 for Arrays.cp\noutput:\n{output}");

        // Array global declared.
        assert!(
            output.contains("@Arrays.Data"),
            "expected @data global\noutput:\n{output}"
        );

        // SetElem and GetElem functions compiled.
        assert!(
            output.contains("@SetElem"),
            "expected @SetElem function\noutput:\n{output}"
        );
        assert!(
            output.contains("@GetElem"),
            "expected @GetElem function\noutput:\n{output}"
        );

        // GEP instruction present (array element access).
        assert!(
            output.contains("getelementptr") || output.contains("gep"),
            "expected getelementptr in output\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_arrays_of_records_field_access() {
        let (output, code) = dump_llvm("Mod/Tests/Arrays.cp");
        assert_eq!(code, 0, "expected exit 0 for Arrays.cp\noutput:\n{output}");

        // Point struct type declared.
        assert!(
            output.contains("%Point = type"),
            "expected %Point struct type\noutput:\n{output}"
        );

        // SetPoint, GetX, GetY functions compiled.
        assert!(
            output.contains("@SetPoint"),
            "expected @SetPoint function\noutput:\n{output}"
        );
        assert!(
            output.contains("@GetX"),
            "expected @GetX function\noutput:\n{output}"
        );
        assert!(
            output.contains("@GetY"),
            "expected @GetY function\noutput:\n{output}"
        );

        // Array-index GEP into the Point array, then struct-field GEP for x and y.
        // The index GEP uses the Point type; the field GEP is inbounds with field indices.
        assert!(
            output.contains("getelementptr %Point"),
            "expected GEP into Point array\noutput:\n{output}"
        );
        assert!(
            output.contains("getelementptr inbounds %Point"),
            "expected inbounds GEP for Point field\noutput:\n{output}"
        );

        // Field 0 (x) accessed in GetX.
        assert!(
            output.contains("i32 0, i32 0"),
            "expected field index 0 GEP (x)\noutput:\n{output}"
        );

        // Field 1 (y) accessed in GetY.
        assert!(
            output.contains("i32 0, i32 1"),
            "expected field index 1 GEP (y)\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_array_method_call_dispatches_via_vtable() {
        let (output, code) = dump_llvm("Mod/Tests/ArrayMethods.cp");
        assert_eq!(code, 0, "expected exit 0 for ArrayMethods.cp\noutput:\n{output}");

        // Node struct type and vtable emitted.
        assert!(
            output.contains("%Node = type"),
            "expected %Node struct type\noutput:\n{output}"
        );
        assert!(
            output.contains("@Node.vtable"),
            "expected @Node.vtable\noutput:\n{output}"
        );
        assert!(
            output.contains("@Node_GetVal"),
            "expected @Node_GetVal function\noutput:\n{output}"
        );

        // CallGetVal compiled.
        assert!(
            output.contains("@CallGetVal"),
            "expected @CallGetVal function\noutput:\n{output}"
        );

        // Array-index GEP selects the right slot.
        assert!(
            output.contains("getelementptr ptr"),
            "expected GEP into pointer array\noutput:\n{output}"
        );

        // Vtable dispatch sequence: load tag, mask, load vtable, load fn_ptr, indirect call.
        assert!(
            output.contains("getelementptr i8") && output.contains("i64 -16"),
            "expected BlockHeader tag load (obj-16)\noutput:\n{output}"
        );
        assert!(
            output.contains("and i64") && output.contains("-2"),
            "expected tag masking\noutput:\n{output}"
        );
        assert!(
            output.contains("call i64 %fn_ptr"),
            "expected indirect method call via fn_ptr\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_nested_procs_lambda_lifted() {
        let (output, code) = dump_llvm("Mod/Tests/Nested.cp");
        assert_eq!(code, 0, "expected exit 0 for Nested.cp\noutput:\n{output}");

        // Outer calls Outer_Double — no upvalue args (pure param pass-through).
        assert!(
            output.contains("@Outer_Double"),
            "expected lambda-lifted @Outer_Double\noutput:\n{output}"
        );
        assert!(
            output.contains("call i64 @Outer_Double(i64"),
            "expected Outer to call Outer_Double with one i64 arg\noutput:\n{output}"
        );

        // WithCapture_Add receives `offset` as first ptr (upvalue ref param).
        assert!(
            output.contains("define i64 @WithCapture_Add(ptr %0, i64 %1)"),
            "expected WithCapture_Add with ptr upvalue param\noutput:\n{output}"
        );
        assert!(
            output.contains("call i64 @WithCapture_Add(ptr %offset, i64 10)"),
            "expected WithCapture to pass offset alloca ptr to Add\noutput:\n{output}"
        );

        // WithMutation_Accumulate receives `accum` as first ptr, returns void.
        assert!(
            output.contains("define void @WithMutation_Accumulate(ptr %0, i64 %1)"),
            "expected WithMutation_Accumulate with ptr upvalue and void return\noutput:\n{output}"
        );
        // Accumulate is called twice: once with n, once with n*2.
        assert!(
            output.contains("call void @WithMutation_Accumulate(ptr %accum,"),
            "expected WithMutation to call Accumulate with accum ptr\noutput:\n{output}"
        );
    }

    // -------------------------------------------------------------------------
    // String-array execution tests
    // -------------------------------------------------------------------------

    #[test]
    fn invoke_str_arrays_fixed_size_passed_as_pointer() {
        let (output, code) = invoke_command("Mod/Tests/StrArrays.cp::Run");
        assert_eq!(code, 0, "expected exit 0 for StrArrays.Run\noutput:\n{output}");
        assert!(
            output.contains("hello from literal"),
            "expected string literal passed through open-array param\noutput:\n{output}"
        );
        assert!(
            output.contains("fixed array copy"),
            "expected fixed local array passed through open-array param\noutput:\n{output}"
        );
        assert!(
            output.contains("seven!"),
            "expected small fixed array passed through open-array param\noutput:\n{output}"
        );
        assert!(
            output.contains("global array"),
            "expected module-global fixed array passed through open-array param\noutput:\n{output}"
        );
    }

    #[test]
    fn invoke_str_arrays_ir_passes_arrays_as_pointers() {
        let (output, code) = dump_ir("Mod/Tests/StrArrays.cp");
        assert_eq!(code, 0, "expected exit 0 for StrArrays IR dump\noutput:\n{output}");
        // The fixed-size local arrays must appear as alloca'd slots and be
        // passed by address (ptr), never loaded as [N x i8] values.
        assert!(
            output.contains("call Console.WriteShortString("),
            "expected WriteShortString call in IR\noutput:\n{output}"
        );
        // The call to PrintLn with the local32 array must pass it by address
        // (i.e., the arg is an IrValue::GlobalRef/Ref or a temp from designator_addr,
        // not a loaded array value).
        assert!(
            !output.contains("load local32") && !output.contains("load local8"),
            "arrays must not be loaded as values before passing\noutput:\n{output}"
        );
    }

    // -------------------------------------------------------------------------
    // Result-calculation tests (Calc.cp)
    // These load and JIT Mod/Calc.cp directly via the loader API and call
    // exported functions by address, asserting on the i64 return value.
    // No subprocess, no IR text matching, no console output parsing.
    // -------------------------------------------------------------------------

    #[test]
    fn calc_arithmetic_add() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "Add"), 7);
    }

    #[test]
    fn calc_arithmetic_sub() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "Sub"), 7);
    }

    #[test]
    fn calc_arithmetic_mul() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "Mul"), 42);
    }

    #[test]
    fn calc_arithmetic_div() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "DivPos"), 3);
    }

    #[test]
    fn calc_arithmetic_mod() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ModPos"), 2);
    }

    #[test]
    fn calc_arithmetic_neg() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "NegArith"), -7);
    }

    #[test]
    fn calc_cmp_true() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CmpTrue"), 1);
    }

    #[test]
    fn calc_cmp_false() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CmpFalse"), 0);
    }

    #[test]
    fn calc_char_ord() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CharOrd"), 65);
    }

    #[test]
    fn calc_char_hex_literal() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CharHex"), 65);
    }

    #[test]
    fn calc_char_chr() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CharChr"), 90);
    }

    #[test]
    fn calc_shortchar_ord() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ShortCharOrd"), 97);
    }

    #[test]
    fn calc_shortchar_literal() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ShortCharLit"), 42);
    }

    #[test]
    fn calc_shortchar_array_literal_len() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "LiteralLen"), 5);
    }

    #[test]
    fn calc_shortchar_array_copy_len() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ArrayCopy"), 2);
    }

    #[test]
    fn calc_set_in() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "SetIn"), 1);
    }

    #[test]
    fn calc_set_not_in() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "SetNotIn"), 0);
    }

    #[test]
    fn calc_loop_sum_to_10() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "SumTo10"), 55);
    }

    #[test]
    fn calc_loop_factorial_5() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "Factorial5"), 120);
    }

    #[test]
    fn calc_case_circle() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CaseCircle"), 0);
    }

    #[test]
    fn calc_case_triangle() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CaseTriangle"), 3);
    }

    #[test]
    fn calc_case_else() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CaseElse"), -1);
    }

    // -------------------------------------------------------------------------
    // Floor DIV/MOD — CP spec: x DIV y = ENTIER(x/y), MOD satisfies
    //   0 <= (x MOD y) < y  when y > 0  (floor semantics, not truncation)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_div_neg_dividend() {
        // -5 DIV 3 = -2  (floor), not -1 (truncation)
        assert_eq!(run_function("Mod/Tests/Calc.cp", "DivNeg"), -2);
    }

    #[test]
    fn calc_mod_neg_dividend() {
        // -5 MOD 3 = 1  (always non-negative when divisor > 0)
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ModNeg"), 1);
    }

    #[test]
    fn calc_div_neg_divisor() {
        // 5 DIV -3 = -2  (floor)
        assert_eq!(run_function("Mod/Tests/Calc.cp", "DivNegY"), -2);
    }

    #[test]
    fn calc_mod_neg_divisor() {
        // 5 MOD -3 = -1  (always non-positive when divisor < 0)
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ModNegY"), -1);
    }

    #[test]
    fn calc_div_both_neg() {
        // -5 DIV -3 = 1  (floor)
        assert_eq!(run_function("Mod/Tests/Calc.cp", "DivBothNeg"), 1);
    }

    // -------------------------------------------------------------------------
    // SET binary operators — +, -, *, / on SET type
    // -------------------------------------------------------------------------

    #[test]
    fn calc_set_union() {
        // {1,2} + {3,4}: 3 should be in result
        assert_eq!(run_function("Mod/Tests/Calc.cp", "SetUnion"), 1);
    }

    #[test]
    fn calc_set_intersect() {
        // {1,2,3} * {2,3,4}: 2 in, 1 not in
        assert_eq!(run_function("Mod/Tests/Calc.cp", "SetIntersect"), 1);
    }

    #[test]
    fn calc_set_diff() {
        // {1,2,3} - {2,3,4}: 1 in, 2 not in
        assert_eq!(run_function("Mod/Tests/Calc.cp", "SetDiff"), 1);
    }

    #[test]
    fn calc_set_sym_diff() {
        // {1,2,3} / {2,3,4}: 1 in, 4 in, 2 not in
        assert_eq!(run_function("Mod/Tests/Calc.cp", "SetSymDiff"), 1);
    }

    #[test]
    fn calc_set_range_literal() {
        // {3..7}: 5 in, 2 not in, 8 not in
        assert_eq!(run_function("Mod/Tests/Calc.cp", "SetRange"), 1);
    }

    // -------------------------------------------------------------------------
    // ABS, ODD, ASH
    // -------------------------------------------------------------------------

    #[test]
    fn calc_abs_positive() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "AbsPos"), 7);
    }

    #[test]
    fn calc_abs_negative() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "AbsNeg"), 7);
    }

    #[test]
    fn calc_odd_true() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "OddTrue"), 1);
    }

    #[test]
    fn calc_odd_false() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "OddFalse"), 0);
    }

    #[test]
    fn calc_ash_left_shift() {
        // ASH(1, 4) = 16
        assert_eq!(run_function("Mod/Tests/Calc.cp", "AshLeft"), 16);
    }

    #[test]
    fn calc_ash_right_shift() {
        // ASH(16, -2) = 4
        assert_eq!(run_function("Mod/Tests/Calc.cp", "AshRight"), 4);
    }

    // -------------------------------------------------------------------------
    // FOR loop
    // -------------------------------------------------------------------------

    #[test]
    fn calc_for_sum_1_to_5() {
        // 1+2+3+4+5 = 15
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ForSum"), 15);
    }

    #[test]
    fn calc_for_by_2() {
        // 0+2+4+6+8+10 = 30
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ForBy2"), 30);
    }

    #[test]
    fn calc_for_count_down() {
        // 5+4+3+2+1 = 15
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ForDown"), 15);
    }

    // -------------------------------------------------------------------------
    // LOOP / EXIT
    // -------------------------------------------------------------------------

    #[test]
    fn calc_loop_exit() {
        // increment until i >= 5, return i
        assert_eq!(run_function("Mod/Tests/Calc.cp", "LoopExit"), 5);
    }

    // -------------------------------------------------------------------------
    // Two-argument MAX / MIN
    // -------------------------------------------------------------------------

    #[test]
    fn calc_max_of_two() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "MaxOfTwo"), 7);
    }

    #[test]
    fn calc_min_of_two() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "MinOfTwo"), 3);
    }

    // -------------------------------------------------------------------------
    // INC / DEC variants  (§10.3)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_inc_step() {
        // INC(x, 4) with x=3 → 7
        assert_eq!(run_function("Mod/Tests/Calc.cp", "IncStep"), 7);
    }

    #[test]
    fn calc_dec_one() {
        // DEC(x) with x=8 → 7
        assert_eq!(run_function("Mod/Tests/Calc.cp", "DecOne"), 7);
    }

    #[test]
    fn calc_dec_step() {
        // DEC(x, 3) with x=10 → 7
        assert_eq!(run_function("Mod/Tests/Calc.cp", "DecStep"), 7);
    }

    // -------------------------------------------------------------------------
    // INCL / EXCL  (§10.3)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_incl_excl() {
        // INCL(s,5); EXCL(s,5); INCL(s,3): 3 in, 5 not in
        assert_eq!(run_function("Mod/Tests/Calc.cp", "InclExcl"), 1);
    }

    // -------------------------------------------------------------------------
    // Monadic SET complement  -s = {i | i NOT IN s}  (§8.2.3)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_set_complement() {
        // s={0,1,2}; t:=-s: 0 not in t, 3 in t
        assert_eq!(run_function("Mod/Tests/Calc.cp", "SetComplement"), 1);
    }

    // -------------------------------------------------------------------------
    // ELSIF chain  (§9.4)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_elsif_chain() {
        // x=5: matches ELSIF x<10 → 1
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ElsifChain"), 1);
    }

    // -------------------------------------------------------------------------
    // CASE with range labels  (§9.5)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_case_range() {
        // x=7: matches arm 7..9 → 3
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CaseRange"), 3);
    }

    // -------------------------------------------------------------------------
    // BOOLEAN as an assignable value  (§6.1, §9.1)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_bool_val() {
        // b := 3 > 2 (TRUE); IF b → 1
        assert_eq!(run_function("Mod/Tests/Calc.cp", "BoolVal"), 1);
    }

    // -------------------------------------------------------------------------
    // Nested WHILE loop  (§9.6)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_double_loop() {
        // 3×3 iterations → s = 9
        assert_eq!(run_function("Mod/Tests/Calc.cp", "DoubleLoop"), 9);
    }

    // -------------------------------------------------------------------------
    // Early RETURN  (§9.10)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_early_return() {
        // returns i immediately when i = 5
        assert_eq!(run_function("Mod/Tests/Calc.cp", "EarlyReturn"), 5);
    }

    // -------------------------------------------------------------------------
    // Recursive call  (§10)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_recursive_factorial() {
        // RecFact(5) = 5! = 120
        assert_eq!(run_function("Mod/Tests/Calc.cp", "RecFactorial5"), 120);
    }

    // -------------------------------------------------------------------------
    // REPEAT / UNTIL with DEC  (§9.7, §10.3)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_repeat_down() {
        // i=10; REPEAT DEC(i) UNTIL i<=5 → 5
        assert_eq!(run_function("Mod/Tests/Calc.cp", "RepeatDown"), 5);
    }

    // -------------------------------------------------------------------------
    // Local CONST declaration  (§5)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_local_const() {
        // CONST N=6; N*N = 36
        assert_eq!(run_function("Mod/Tests/Calc.cp", "LocalConst"), 36);
    }

    // -------------------------------------------------------------------------
    // LEN of a fixed-size array  (§10.3)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_len_fixed_array() {
        // VAR a: ARRAY 10 OF INTEGER; LEN(a) = 10
        assert_eq!(run_function("Mod/Tests/Calc.cp", "LenFixed"), 10);
    }

    #[test]
    fn calc_len_open_array() {
        // ARRAY 32 OF SHORTCHAR passed to `IN s: ARRAY OF SHORTCHAR`;
        // LEN(s) reads the hidden length companion (= 32).
        assert_eq!(run_function("Mod/Tests/Calc.cp", "LenOpenArray"), 32);
    }

    #[test]
    fn calc_len_open_array_forwarded() {
        // ARRAY 17 OF SHORTCHAR forwarded through one open-array param to another;
        // hidden length must be threaded (= 17).
        assert_eq!(run_function("Mod/Tests/Calc.cp", "LenOpenArrayForward"), 17);
    }

    // -------------------------------------------------------------------------
    // MIN(T) / MAX(T) — single-argument form returns the type's bounds (§10.3)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_max_longint() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "MaxLong"), i64::MAX);
    }

    #[test]
    fn calc_min_longint_plus_one() {
        // MIN(LONGINT) + 1 == -i64::MAX
        assert_eq!(run_function("Mod/Tests/Calc.cp", "MinLong"), -i64::MAX);
    }

    #[test]
    fn calc_max_intshort() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "MaxIntShort"), i32::MAX as i64);
    }

    #[test]
    fn calc_max_set_index() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "MaxSetIdx"), 31);
    }

    #[test]
    fn calc_min_shortint() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "MinShortInt"), i16::MIN as i64);
    }

    // -------------------------------------------------------------------------
    // arr := "stringliteral" for fixed-size CHAR / SHORTCHAR arrays  (§9.1)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_arr_assign_char_literal() {
        // a := "ABC"; ORD(a[2]) == 'C' == 67. Asserts CHAR (UTF-32) memcpy lands
        // the right code point at index 2.
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ArrAssignCharLit"), 67);
    }

    #[test]
    fn calc_arr_assign_shortchar_literal() {
        // a := "abc" with `a: ARRAY 8 OF SHORTCHAR`. Literal defaults to CHAR;
        // assignment must retype the source to SHORTCHAR for the byte-wise memcpy.
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ArrAssignShortCharLit"), 98);
    }

    #[test]
    fn calc_arr_assign_literal_null_terminator() {
        // a := "hi"; a[2] must be 0X (NUL terminator copied).
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ArrAssignLitNullTerm"), 0);
    }

    // -------------------------------------------------------------------------
    // Math (Rust-resident native module): cross-module REAL calls
    // -------------------------------------------------------------------------

    #[test]
    fn math_sqrt_via_native_module() {
        // ENTIER(Math.Sqrt(9.0)) = 3
        assert_eq!(run_function("Mod/Tests/MathSmoke.cp", "Sqrt9"), 3);
    }

    #[test]
    fn math_pi_via_native_module() {
        // ENTIER(Math.Pi() * 2) = 6
        assert_eq!(run_function("Mod/Tests/MathSmoke.cp", "PiTimes2"), 6);
    }

    #[test]
    fn math_int_power_via_native_module() {
        // Math.IntPower(2.0, 10) = 1024 (exercises the (REAL, INTEGER) -> REAL signature).
        assert_eq!(run_function("Mod/Tests/MathSmoke.cp", "IntPow"), 1024);
    }

    #[test]
    fn math_exponent_decomposition() {
        // Math.Exponent(8.0) = 3 since 8 = 1.0 * 2^3
        // Exercises the (REAL) -> INTEGER bit-decomposition path.
        assert_eq!(run_function("Mod/Tests/MathSmoke.cp", "ExponentOf"), 3);
    }

    #[test]
    fn strings_string_to_real_roundtrip() {
        // Strings.StringToReal("3.14e2", x, res) -> x == 314.0; ENTIER -> 314.
        // End-to-end check that Strings.cp -> Math (Rust libm) actually links.
        assert_eq!(run_function("Mod/Tests/MathSmoke.cp", "StringsRoundTrip"), 314);
    }

    #[test]
    fn strings_real_to_string_round_trip() {
        // RealToString(12.5) -> some scientific-notation form -> StringToReal -> 12.5 -> ENTIER 12
        assert_eq!(run_function("Mod/Tests/MathSmoke.cp", "RealToStringRoundTrip"), 12);
    }

    #[test]
    fn strings_short_str_to_real() {
        // SHORTCHAR parser via Widen + StringToReal: "42.5e1" -> 425.0 -> ENTIER 425
        assert_eq!(run_function("Mod/Tests/MathSmoke.cp", "ShortStrToRealCheck"), 425);
    }

    // -------------------------------------------------------------------------
    // OOP: pointer-aliased records — sema + IR layers verified.
    //
    // Virtual dispatch is implemented by emitting mutable vtable globals and
    // patching them post-JIT with the final method addresses.
    // -------------------------------------------------------------------------

    #[test]
    fn ptr_alloc_no_dispatch() {
        // Smallest tagged-record test: NEW + write field + read field, no
        // method call. Verifies __newcp_new_rec sets up the BlockHeader and
        // returns a usable payload pointer.
        assert_eq!(run_function("Mod/Tests/PtrAlloc.cp", "Run"), 42);
    }

    #[test]
    fn ptr_alloc_block_header_tag_is_typedesc() {
        // After NEW(b), `*(addr - 16)` (the BlockHeader.tag field) must be
        // the TypeDesc address with the GC mark bit cleared. Verifies the
        // allocator threads the type descriptor correctly.
        assert_eq!(run_function("Mod/Tests/PtrSet.cp", "Probe"), 1);
    }

    #[test]
    fn ptr_set_probe_vtable_fn() {
        // After post-JIT vtable patching, vtable[0] should be the address of
        // BoxDesc.Set (non-zero).
        let v = run_function("Mod/Tests/PtrSet.cp", "ProbeFn0");
        eprintln!("vtable[0] = 0x{:x}", v as u64);
        assert!(v != 0, "vtable[0] is zero — post-JIT patching didn't populate it");
    }

    #[test]
    fn ptr_method_box_set_get() {
        // Pointer-aliased OOP: NEW(b) + b.Set(42) + b.Get() -> 42.
        // Exercises auto-deref of pointer aliases for method receivers
        // and end-to-end vtable dispatch through the patched mutable vtable.
        assert_eq!(run_function("Mod/Tests/PtrMethod.cp", "Run"), 42);
    }

    #[test]
    fn abstract_dispatch_square() {
        // Abstract pointer base + concrete subclass + virtual dispatch:
        // Square(side=5).Area() through Shape -> 25.
        assert_eq!(run_function("Mod/Tests/AbstractDispatch.cp", "TestSquare"), 25);
    }

    #[test]
    fn abstract_dispatch_circle() {
        // Different concrete subclass: Circle(r=4).Area() -> 3 * 4 * 4 = 48.
        // Same call site (`AreaOf`) dispatches to the right Area override.
        assert_eq!(run_function("Mod/Tests/AbstractDispatch.cp", "TestCircle"), 48);
    }

    /// Helper that returns Err(diagnostic-string) when the loader's sema
    /// rejects the module — useful for asserting that a specific kind of
    /// cross-module error is or is not present.
    fn loader_error(module_ref: &str) -> Option<String> {
        let abs = workspace_root().join(module_ref);
        let abs_str = abs.to_str().expect("utf-8 path");
        let mut session = newcp_loader::LoaderSession::new();
        match session.ensure_import_graph_loaded(abs_str) {
            Ok(_) => None,
            Err(e) => Some(e),
        }
    }

    #[test]
    fn int_literal_narrows_to_byte() {
        // CP: integer literals are polymorphic and adapt to the static
        // type of the assignment target. `x := 200` for x: BYTE must
        // be accepted (200 fits in u8); `x := 0` for x: BYTE likewise.
        // Used to fail with "expected BYTE, found INTEGER".
        assert_eq!(run_function("Mod/Tests/IntLitNarrowing.cp", "LitToByte"), 200);
    }

    #[test]
    fn int_literal_narrows_to_shortint() {
        // Same shape, narrower target type.
        assert_eq!(
            run_function("Mod/Tests/IntLitNarrowing.cp", "LitToShortInt"),
            100,
        );
    }

    #[test]
    fn xmod_inherited_field_access_through_pointer_alias() {
        // Field declared on the imported abstract base, accessed via a
        // local-subclass pointer. Used to fail with "unsupported cast
        // from i64 to opaque:field:res" when the IR layer's record-
        // field flattening didn't follow the inheritance chain across
        // the source-directory boundary.
        assert_eq!(
            run_function("Mod/Tests/XmodSubtype.cp", "TouchInheritedField"),
            99,
        );
    }

    #[test]
    fn xmod_subtype_assignment() {
        // Blocker 2: a concrete subclass of an imported abstract base must
        // be assignable to the base's pointer alias when returned. Sema
        // currently rejects this with "return type mismatch: expected
        // imported:<Base>, found type:<Sub>" because record-extends
        // doesn't follow inheritance through imported parents.
        let err = loader_error("Mod/Tests/XmodSubtype.cp");
        assert!(
            err.is_none(),
            "expected clean load, got error: {}",
            err.unwrap_or_default(),
        );
    }

    #[test]
    fn xmod_type_alias_passes_array_of_char_through_imported_typedef() {
        // Blocker 5: passing a value of an imported typedef'd fixed array
        // (XmodTypeAliasBase.Name = ARRAY 16 OF CHAR) where ARRAY OF CHAR
        // is expected. Sema should see through the cross-module alias.
        // Currently fails with "expected ARRAY OF CHAR, found
        // imported:XmodTypeAliasBase.Name".
        let err = loader_error("Mod/Tests/XmodTypeAlias.cp");
        assert!(
            err.is_none(),
            "expected clean load, got error: {}",
            err.unwrap_or_default(),
        );
    }

    #[test]
    fn host_files_diag_this() {
        // HostFiles.theDir.This(path) — exercises cross-module method
        // dispatch on a receiver imported from HostFiles, with the
        // path argument being a fixed-size local array.
        assert_eq!(run_function("Mod/Tests/HostFilesRoundTrip.cp", "DiagThis"), 1);
    }

    #[test]
    fn host_files_diag_open_direct() {
        assert_eq!(
            run_function("Mod/Tests/HostFilesRoundTrip.cp", "DiagOpenDirect"),
            1,
        );
    }

    #[test]
    fn host_files_diag_flat_open() {
        // Bypass the OOP layer; verifies the flat HostFileSys API works
        // from CP without any virtual dispatch.
        assert_eq!(run_function("Mod/Tests/HostFilesRoundTrip.cp", "DiagFlatOpen"), 1);
    }

    #[test]
    fn host_files_diag_open() {
        assert_eq!(run_function("Mod/Tests/HostFilesRoundTrip.cp", "DiagOpen"), 1);
    }

    #[test]
    fn host_files_write_then_read_byte() {
        // End-to-end Files / HostFiles / HostFileSys path:
        //   StdDir.This  -> Locator
        //   StdDir.New   -> File (read+write, fresh truncate)
        //   File.NewWriter / Writer.WriteByte
        //   File.NewReader / Reader.ReadByte (OUT BYTE)
        // Exercises virtual dispatch through every Files.* abstract pointer
        // type to the concrete HostFiles.Std* subclasses, and round-trips
        // a byte through std::fs.
        assert_eq!(
            run_function("Mod/Tests/HostFilesRoundTrip.cp", "WriteThenReadByte"),
            0xAA,
        );
    }

    #[test]
    fn host_files_write_then_read_bytes() {
        assert_eq!(
            run_function("Mod/Tests/HostFilesRoundTrip.cp", "WriteThenReadBytes"),
            36,
        );
    }

    #[test]
    fn host_files_length_after_write() {
        // Calls f.Length() on a Files.File pointer — exercises virtual
        // dispatch for an abstract method that returns INTEGER through
        // an imported abstract base.
        assert_eq!(
            run_function("Mod/Tests/HostFilesRoundTrip.cp", "LengthAfterWrite"),
            3,
        );
    }

    #[test]
    fn strings_real_to_short_str_round_trip() {
        // Format into SHORTCHAR buffer (via Narrow) then parse back (via Widen).
        // RealToShortStr(7.5) -> "7.5..." -> ShortStrToReal -> 7.5 -> ENTIER 7.
        // Exercises both byte<->wide bridges for the real-number procs.
        assert_eq!(run_function("Mod/Tests/MathSmoke.cp", "RealToShortStrRoundTrip"), 7);
    }


    #[test]
    fn dyn_array_new_and_index_round_trip() {
        // POINTER TO ARRAY OF SHORTINT, NEW(p, 4), p[i] := v, sum.
        // 7 + 11 + 13 + 17 = 48.
        assert_eq!(run_function("Mod/Tests/DynArray.cp", "NewAndIndex"), 48);
    }

    #[test]
    fn dyn_array_len_reads_back() {
        // LEN(p^) reads the length stored by NewArray's header.
        assert_eq!(run_function("Mod/Tests/DynArray.cp", "Length"), 5);
    }

    #[test]
    fn in_param_write_rejected_by_sema() {
        // Three patterns of write-through-IN that sema should all
        // reject: scalar `n := 7`, field `b.value := 99`, and
        // indexed `a[0] := 1` — for the IN params declared in
        // Mod/Tests/InParamWrite.cp.
        let err = loader_error("Mod/Tests/InParamWrite.cp")
            .expect("expected sema to reject IN-parameter writes");
        for needle in [
            "cannot assign through IN parameter 'n'",
            "cannot assign through IN parameter 'b'",
            "cannot assign through IN parameter 'a'",
        ] {
            assert!(
                err.contains(needle),
                "expected '{needle}' in diagnostic, got: {err}"
            );
        }
    }

    #[test]
    fn value_record_param_is_private_copy() {
        // CP §8.1: a value-mode record param is a private copy. The
        // ABI passes the caller's pointer; the callee prologue
        // memmoves the bytes into a stack-local alloca, so writes
        // through the param don't leak back. Mutate(b: Box) writes
        // 999 into b.value; the caller's record stays at 42.
        assert_eq!(
            run_function("Mod/Tests/ValueRecordParamProbe.cp", "Run"),
            42,
        );
    }

    #[test]
    fn value_fixed_array_param_is_private_copy() {
        // Same private-copy contract for fixed-size array params.
        assert_eq!(
            run_function("Mod/Tests/ValueFixedArrayProbe.cp", "Run"),
            42,
        );
    }

    // -------------------------------------------------------------------------
    // Dates: pure-value arithmetic (no clock dependency)
    // -------------------------------------------------------------------------

    #[test]
    fn dates_day_ordinal_for_2026_05_09() {
        // Sanity check the BlackBox Day formula round-trips. The exact
        // ordinal value isn't pinned (different epoch from Unix); we
        // assert the round-trip in `dates_round_trip` below. Here we
        // just confirm Day returns a reasonably large positive number
        // for a 2020s date (sanity).
        let n = run_function("Mod/Tests/DatesArith.cp", "DayOfMay9_2026");
        assert!(n > 700_000 && n < 800_000, "unexpected ordinal {}", n);
    }

    #[test]
    fn dates_day_round_trip() {
        // Day(2000-02-29) → DayToDate → 2000-02-29.
        assert_eq!(run_function("Mod/Tests/DatesArith.cp", "RoundTrip"), 1);
    }

    #[test]
    fn dates_weekday_may9_2026_is_saturday() {
        // 2026-05-09 is a Saturday. BlackBox convention: Mon=0..Sun=6.
        assert_eq!(
            run_function("Mod/Tests/DatesArith.cp", "WeekdayOfMay9_2026"),
            5
        );
    }

    #[test]
    fn dates_weekday_2024_jan_1_is_monday() {
        assert_eq!(
            run_function("Mod/Tests/DatesArith.cp", "Weekday2024Jan1"),
            0
        );
    }

    #[test]
    fn dates_easter_2024() {
        // Easter Sunday 2024 = March 31, 2024 → 3*100 + 31 = 331.
        assert_eq!(run_function("Mod/Tests/DatesArith.cp", "Easter2024"), 331);
    }

    #[test]
    fn dates_easter_2025() {
        // Easter Sunday 2025 = April 20, 2025 → 4*100 + 20 = 420.
        assert_eq!(run_function("Mod/Tests/DatesArith.cp", "Easter2025"), 420);
    }

    #[test]
    fn dates_feb29_in_leap_year_is_valid() {
        assert_eq!(
            run_function("Mod/Tests/DatesArith.cp", "FebInLeapYearIsValid"),
            1
        );
    }

    #[test]
    fn dates_feb29_in_nonleap_is_invalid() {
        assert_eq!(
            run_function("Mod/Tests/DatesArith.cp", "FebInNonLeapIsInvalid"),
            0
        );
    }

    #[test]
    fn dates_valid_time_midnight() {
        assert_eq!(run_function("Mod/Tests/DatesArith.cp", "ValidTimeMidnight"), 1);
    }

    #[test]
    fn dates_valid_time_24h_rejected() {
        assert_eq!(
            run_function("Mod/Tests/DatesArith.cp", "ValidTimeOutOfRange"),
            0
        );
    }

    // -------------------------------------------------------------------------
    // Dates + HostDates: real clock + formatting via the OOP hook
    // -------------------------------------------------------------------------

    #[test]
    fn dates_get_date_returns_recent_year() {
        // Hook installed by HostDates.body should fetch a sane local date.
        assert_eq!(
            run_function("Mod/Tests/DatesClock.cp", "GetDateReturnsRecentYear"),
            1
        );
    }

    #[test]
    fn dates_get_utc_date_returns_recent_year() {
        assert_eq!(
            run_function("Mod/Tests/DatesClock.cp", "GetUTCDateReturnsRecentYear"),
            1
        );
    }

    #[test]
    fn dates_get_time_fields_in_range() {
        assert_eq!(
            run_function("Mod/Tests/DatesClock.cp", "GetTimeFieldsInRange"),
            1
        );
    }

    #[test]
    fn dates_date_to_string_non_empty() {
        // Some characters were written for "5/9/2026" — at least one byte.
        let n = run_function("Mod/Tests/DatesClock.cp", "DateToStringNonEmpty");
        assert!(n > 0, "expected non-empty formatted date, got {}", n);
    }

    #[test]
    fn dates_time_to_string_zero_pads() {
        // "07:05:03" — first three chars are '0' (0x30), '7' (0x37), ':' (0x3A).
        // packed: 0x30 * 65536 + 0x37 * 256 + 0x3A
        let want = 0x30 * 65536 + 0x37 * 256 + 0x3A;
        assert_eq!(
            run_function("Mod/Tests/DatesClock.cp", "TimeToStringFirstThree"),
            want
        );
    }

    // -------------------------------------------------------------------------
    // ENTIER: floor of real → INTEGER  (§10.3)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_entier_floor() {
        // ENTIER(3.7) = 3
        assert_eq!(run_function("Mod/Tests/Calc.cp", "EntierFloor"), 3);
    }

    #[test]
    fn calc_entier_neg() {
        // ENTIER(-1.2) = -2  (floor, not truncation)
        assert_eq!(run_function("Mod/Tests/Calc.cp", "EntierNeg"), -2);
    }

    #[test]
    fn calc_real_add_entier() {
        // ENTIER(1.5 + 1.5) = 3
        assert_eq!(run_function("Mod/Tests/Calc.cp", "RealAdd"), 3);
    }

    // -------------------------------------------------------------------------
    // CAP: capitalize a Latin-1 letter  (§10.3)
    // -------------------------------------------------------------------------

    #[test]
    fn calc_cap_lower() {
        // ORD(CAP('a')) = 65 = ORD('A')
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CapLower"), 65);
    }

    // =========================================================================
    // OR operator  (§8.2.1)
    // =========================================================================

    #[test]
    fn calc_or_true() {
        // ODD(3) OR ODD(4) = TRUE OR FALSE = TRUE → 1
        assert_eq!(run_function("Mod/Tests/Calc.cp", "OrTrue"), 1);
    }

    #[test]
    fn calc_or_false() {
        // ODD(4) OR ODD(6) = FALSE OR FALSE = FALSE → 0
        assert_eq!(run_function("Mod/Tests/Calc.cp", "OrFalse"), 0);
    }

    // =========================================================================
    // Real division /  (§8.2.2)
    // =========================================================================

    #[test]
    fn calc_real_div() {
        // ENTIER(7.0 / 2.0) = ENTIER(3.5) = 3
        assert_eq!(run_function("Mod/Tests/Calc.cp", "RealDiv"), 3);
    }

    // =========================================================================
    // Hex integer literal  H suffix  (§3)
    // =========================================================================

    #[test]
    fn calc_hex_lit() {
        // 0FFH = 255
        assert_eq!(run_function("Mod/Tests/Calc.cp", "HexLit"), 255);
    }

    // =========================================================================
    // SHORT / LONG  (§10.3)
    // =========================================================================

    #[test]
    fn calc_short_long_roundtrip() {
        // SHORT(1000): INTEGER→INTSHORT; LONG(x): INTSHORT→INTEGER → 1000
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ShortLong"), 1000);
    }

    // =========================================================================
    // ENTIER of SHORTREAL  (§10.3)
    // =========================================================================

    #[test]
    fn calc_entier_shortreal() {
        // ENTIER(SHORT(3.7)) = floor(3.7f32) = 3
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ShortRealFloor"), 3);
    }

    // =========================================================================
    // BITS  (§10.3)
    // =========================================================================

    #[test]
    fn calc_bits_test() {
        // BITS(5) = {0,2} since 5 = 101b; test membership
        assert_eq!(run_function("Mod/Tests/Calc.cp", "BitsTest"), 1);
    }

    // =========================================================================
    // ORD of SET  (§10.3)
    // =========================================================================

    #[test]
    fn calc_ord_set() {
        // ORD({0,2}) = 2^0 + 2^2 = 1 + 4 = 5
        assert_eq!(run_function("Mod/Tests/Calc.cp", "OrdSet"), 5);
    }

    // =========================================================================
    // CASE with CHAR expression and range labels  (§9.5)
    // =========================================================================

    #[test]
    fn calc_case_char() {
        // ch := 'M'; CASE ch OF 'A'..'Z' → 1
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CaseChar"), 1);
    }

    // =========================================================================
    // CASE with comma-separated label list  (§9.5)
    // =========================================================================

    #[test]
    fn calc_case_multi_label() {
        // x := 3; CASE x OF 1,3,5 → 1
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CaseMultiLabel"), 1);
    }

    // =========================================================================
    // Record field access  (§6.3)
    // =========================================================================

    #[test]
    fn calc_record_fields() {
        // p.x := 3; p.y := 4; p.x + p.y = 7
        assert_eq!(run_function("Mod/Tests/Calc.cp", "RecordFields"), 7);
    }

    // =========================================================================
    // 2-dimensional array indexing  (§6.2)
    // =========================================================================

    #[test]
    fn calc_array_2d() {
        // a[1][2] := 7; a[1][2] = 7
        assert_eq!(run_function("Mod/Tests/Calc.cp", "Array2D"), 7);
    }

    // =========================================================================
    // ARRAY m,n abbreviated form + a[i,j] multi-index  (§6.2)
    // =========================================================================

    #[test]
    fn calc_array_2d_comma() {
        assert_eq!(run_function("Mod/Tests/Calc.cp", "Array2DComma"), 42);
    }

    // =========================================================================
    // Comparison operators  # >= <=  (§8.2.5)
    // =========================================================================

    #[test]
    fn calc_cmp_neq() {
        // 3 # 5  → TRUE → 1
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CmpNeq"), 1);
    }

    #[test]
    fn calc_cmp_geq() {
        // 5 >= 5 → TRUE → 1
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CmpGeq"), 1);
    }

    #[test]
    fn calc_cmp_leq() {
        // 3 <= 5 → TRUE → 1
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CmpLeq"), 1);
    }

    // =========================================================================
    // Boolean NOT of a variable  (§8.2.1)
    // =========================================================================

    #[test]
    fn calc_bool_not() {
        // b := ~(3 > 5)  = ~FALSE = TRUE → 1
        assert_eq!(run_function("Mod/Tests/Calc.cp", "BoolNot"), 1);
    }

    // =========================================================================
    // Module-level global variable  (§7, §11)
    // =========================================================================

    #[test]
    fn calc_glob_var() {
        // globalX := 99; RETURN globalX → 99
        assert_eq!(run_function("Mod/Tests/Calc.cp", "GlobVarTest"), 99);
    }

    // =========================================================================
    // L-suffix integer literal → LONGINT  (§3)
    // =========================================================================

    #[test]
    fn calc_l_lit() {
        // 0FFFF0000L = 4294901760
        assert_eq!(run_function("Mod/Tests/Calc.cp", "LLit"), 4294901760_i64);
    }

    // =========================================================================
    // VAR parameter (pass by reference)  (§10.1)
    // =========================================================================

    #[test]
    fn calc_var_param() {
        // n := 14; Increment(n) → INC(n) → n = 15
        assert_eq!(run_function("Mod/Tests/Calc.cp", "VarParamTest"), 15);
    }

    // =========================================================================
    // LOOP with two EXIT points  (§9.9, §9.10)
    // =========================================================================

    #[test]
    fn calc_loop_multi_exit() {
        // exits when i = 3 (the first EXIT fires before i reaches 10)
        assert_eq!(run_function("Mod/Tests/Calc.cp", "LoopMultiExit"), 3);
    }

    // =========================================================================
    // Nested local procedure  (§10 — procedure declarations may be nested)
    // =========================================================================

    #[test]
    fn calc_nested_proc() {
        // NestedProcTest contains local Double(x) = x*2; returns Double(21) = 42
        assert_eq!(run_function("Mod/Tests/Calc.cp", "NestedProcTest"), 42);
    }

    // =========================================================================
    // CHAR comparison  (§8.2.5 — relations on character types)
    // =========================================================================

    #[test]
    fn calc_char_cmp() {
        // 'b' > 'a'  (98 > 97) → TRUE → 1
        assert_eq!(run_function("Mod/Tests/Calc.cp", "CharCmp"), 1);
    }

    // =========================================================================
    // SHORTINT arithmetic via SHORT / LONG  (§6.1, §10.3)
    // =========================================================================

    #[test]
    fn calc_shortint_arith() {
        // x := SHORT(100): INTEGER→SHORTINT; LONG(x)*2 = 200
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ShortIntArith"), 200);
    }

    // =========================================================================
    // IN parameter (read-only open-array formal)  (§10.1)
    // =========================================================================

    #[test]
    fn calc_in_param() {
        // a = [1,2,3,4]; SumArray(a,4) = 10
        assert_eq!(run_function("Mod/Tests/Calc.cp", "InParamTest"), 10);
    }

    #[test]
    fn calc_proc_type_nullary_call() {
        // Store ReturnSeven in a NullaryIntProc variable, call it -> 7
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ProcTypeCall"), 7);
    }

    #[test]
    fn calc_proc_type_param_call() {
        // Store SumTwo in a BinaryIntProc variable, call fn(10, 32) -> 42
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ProcTypeParamCall"), 42);
    }

    #[test]
    fn calc_array_of_record() {
        // ARRAY 4 OF Pair; pairs[2].a=3; pairs[2].b=4; -> 7
        assert_eq!(run_function("Mod/Tests/Calc.cp", "ArrayOfRecord"), 7);
    }

    #[test]
    fn calc_real_param_and_return() {
        // AddReal(1.5, 2.5): REAL -> REAL; ENTIER -> 4
        assert_eq!(run_function("Mod/Tests/Calc.cp", "RealParam"), 4);
    }

    #[test]
    fn kernel_probe_widget_reflection() {
        // TypeOf round-trips a heap allocation through its declared
        // TypeDesc; SizeOf > 0; LevelOf == 0 for a root type; BaseOf
        // is NIL.
        assert_eq!(
            run_function("Mod/Tests/KernelProbe.cp", "WidgetReflection"),
            1,
            "Widget reflection must succeed via Kernel.TypeOf / SizeOf / LevelOf / BaseOf"
        );
    }

    #[test]
    fn kernel_probe_gadget_reflection() {
        // Gadget extends Widget — LevelOf == 1, BaseOf chains to
        // Widget's TypeDesc, SizeOf is strictly larger.
        assert_eq!(
            run_function("Mod/Tests/KernelProbe.cp", "GadgetReflection"),
            1,
            "Gadget reflection must show one level above Widget"
        );
    }

    #[test]
    fn kernel_probe_time_monotonic() {
        assert_eq!(
            run_function("Mod/Tests/KernelProbe.cp", "TimeMonotonic"),
            1,
            "Kernel.Time must be positive and non-decreasing"
        );
    }

    #[test]
    fn kernel_probe_new_obj_round_trip() {
        // Kernel.NewObj allocates a record from a runtime-typed handle;
        // the resulting pointer round-trips through Kernel.TypeOf.
        assert_eq!(
            run_function("Mod/Tests/KernelProbe.cp", "NewObjRoundTrip"),
            1,
            "Kernel.NewObj round-trip via Kernel.TypeOf must succeed"
        );
    }

    #[test]
    fn kernel_probe_get_type_name_returns_bare_name() {
        // GetTypeName returns "WidgetDesc" — the suffix after the
        // last `.` in the codegen-emitted qualified name
        // "KernelProbe.WidgetDesc".
        assert_eq!(
            run_function("Mod/Tests/KernelProbe.cp", "WidgetTypeNameMatches"),
            1,
            "Kernel.GetTypeName must return the bare type name"
        );
    }

    #[test]
    fn kernel_probe_get_qualified_type_name() {
        // GetQualifiedTypeName returns the full "Module.Type" form
        // straight from the codegen-emitted UTF-32 string on TypeDesc.
        assert_eq!(
            run_function("Mod/Tests/KernelProbe.cp", "WidgetQualifiedTypeName"),
            1,
            "Kernel.GetQualifiedTypeName must return the full qualified name"
        );
    }

    #[test]
    fn kernel_probe_this_mod_resolves_registered_module() {
        // Kernel.ThisMod returns a non-NIL handle for a name that was
        // registered at bootstrap (Console, Math, …) and NIL for an
        // unknown name. Verifies the runtime's module-name registry
        // is populated correctly before any user CP code runs.
        assert_eq!(
            run_function("Mod/Tests/KernelProbe.cp", "ThisModResolvesKnownModule"),
            1,
            "ThisMod must succeed for registered modules and fail for unknown"
        );
    }

    /// Locate the BlackBox 1.7 distribution. Tests skip cleanly
    /// when it's absent — set `NEWCP_BB_DIST` to override.
    fn bb_distribution_root() -> Option<std::path::PathBuf> {
        if let Ok(p) = std::env::var("NEWCP_BB_DIST") {
            let path = std::path::PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }
        let default = std::path::PathBuf::from("E:/BlackBox Component Builder 1.7-a1");
        if default.exists() {
            return Some(default);
        }
        None
    }

    /// Copy `Empty.odc` from the BB distribution into the well-
    /// known fixture path the CP probe references. Returns `Ok`
    /// if the fixture is staged and the probe can be expected to
    /// find it; `Err` with a descriptive message otherwise.
    fn stage_empty_odc_fixture() -> Result<(), String> {
        let Some(dist) = bb_distribution_root() else {
            return Err("BlackBox 1.7 distribution not found (set NEWCP_BB_DIST)".to_string());
        };
        let src = dist.join("Empty.odc");
        if !src.exists() {
            return Err(format!("Empty.odc not found at {}", src.display()));
        }
        let workspace = workspace_root();
        let fixture_dir = workspace.join("Mod/Tests/_fixtures");
        std::fs::create_dir_all(&fixture_dir)
            .map_err(|e| format!("create fixture dir: {e}"))?;
        let dst = fixture_dir.join("Empty.odc");
        std::fs::copy(&src, &dst).map_err(|e| format!("copy Empty.odc: {e}"))?;
        Ok(())
    }

    /// Copy a named `.odc` from the BB distribution root into the
    /// fixture directory.  Returns `Ok` if staged, `Err` describing
    /// why otherwise (so the caller can skip cleanly when the BB
    /// distro isn't available on this machine).
    fn stage_bb_odc_fixture(name: &str) -> Result<(), String> {
        let Some(dist) = bb_distribution_root() else {
            return Err("BlackBox 1.7 distribution not found (set NEWCP_BB_DIST)".to_string());
        };
        let src = dist.join(name);
        if !src.exists() {
            return Err(format!("{} not found at {}", name, src.display()));
        }
        let workspace = workspace_root();
        let fixture_dir = workspace.join("Mod/Tests/_fixtures");
        std::fs::create_dir_all(&fixture_dir)
            .map_err(|e| format!("create fixture dir: {e}"))?;
        let dst = fixture_dir.join(name);
        std::fs::copy(&src, &dst).map_err(|e| format!("copy {name}: {e}"))
            .map(|_| ())
    }

    /// Hand-craft a minimal `.odc` containing one root store with the
    /// given qualified type name and body bytes.  Used by the typed-
    /// load test so we don't depend on a particular BlackBox fixture
    /// shape — we control both the type tag and the body contents.
    ///
    /// Wire format follows newcp-odc's reader:
    ///   "CDOo" + 4 zero bytes
    ///   0x82                        (KIND_STORE)
    ///   0xF0                        (NEW_BASE)
    ///   <utf-8 type name>\0
    ///   comment   (i32 LE = 0)
    ///   raw_next  (i32 LE = 0)      // 0 with even comment -> no sibling
    ///   raw_down  (i32 LE = 0)      // no children
    ///   body_len  (i32 LE)
    ///   <body>
    fn write_synthetic_odc(
        path: &std::path::Path,
        qualified_type_name: &str,
        body: &[u8],
    ) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create fixture parent: {e}"))?;
        }
        let mut bytes: Vec<u8> = Vec::with_capacity(64 + qualified_type_name.len() + body.len());
        bytes.extend_from_slice(b"CDOo");
        bytes.extend_from_slice(&[0u8; 4]);
        bytes.push(0x82); // KIND_STORE
        bytes.push(0xF0); // NEW_BASE
        bytes.extend_from_slice(qualified_type_name.as_bytes());
        bytes.push(0); // null-terminated UTF-8 sstring
        bytes.extend_from_slice(&0i32.to_le_bytes()); // comment
        bytes.extend_from_slice(&0i32.to_le_bytes()); // raw_next
        bytes.extend_from_slice(&0i32.to_le_bytes()); // raw_down
        bytes.extend_from_slice(&(body.len() as i32).to_le_bytes()); // body_len
        bytes.extend_from_slice(body);
        std::fs::write(path, &bytes).map_err(|e| format!("write synthetic odc: {e}"))
    }

    /// Stage a synthetic `.odc` fixture into the workspace's
    /// `Mod/Tests/_fixtures` directory under `name`.
    fn stage_synthetic_odc(
        name: &str,
        qualified_type_name: &str,
        body: &[u8],
    ) -> Result<(), String> {
        let workspace = workspace_root();
        let fixture_dir = workspace.join("Mod/Tests/_fixtures");
        let dst = fixture_dir.join(name);
        write_synthetic_odc(&dst, qualified_type_name, body)
    }

    /// Pin the process cwd to the workspace root the first time any
    /// CP probe asks for it.  CP-level `Stores.OpenDocument(...)`
    /// arguments are workspace-relative (`Mod/Tests/...`), so they
    /// need cwd = workspace root.  Cargo launches the test binary
    /// with cwd = the test crate dir, which would misroute the
    /// lookups.
    ///
    /// We previously did this per-call with a save / chdir / chdir-
    /// back dance, but `set_current_dir` is process-global state:
    /// when two parallel test threads each entered the dance they
    /// could end up restoring the cwd back to the wrong directory
    /// (or interleaving such that the inner probe saw the wrong
    /// cwd).  Pinning once is safe because no test in this crate
    /// relies on the original cargo-supplied cwd surviving past the
    /// first probe call.
    fn ensure_workspace_root_cwd() {
        use std::sync::Once;
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
        });
    }

    /// Run a CP probe with the test process cwd pinned to the
    /// workspace root.  Idempotent across calls (see
    /// [`ensure_workspace_root_cwd`]).
    fn run_function_at_workspace_root(module_ref: &str, proc_name: &str) -> i64 {
        ensure_workspace_root_cwd();
        run_function(module_ref, proc_name)
    }

    #[test]
    fn stores_probe_open_walk_empty() {
        // Stage the fixture; skip cleanly if the BB distribution
        // isn't available on this machine.
        match stage_empty_odc_fixture() {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[stores_probe] skipping: {msg}");
                return;
            }
        }

        // OpenAndWalkEmpty: full happy path.
        assert_eq!(
            run_function_at_workspace_root("Mod/Tests/StoresProbe.cp", "OpenAndWalkEmpty"),
            1,
            "Stores S1 should open Empty.odc, walk root, find a child, close cleanly"
        );
    }

    #[test]
    fn stores_probe_negative_paths() {
        // The negative-path probes don't need a fixture file —
        // they exercise OpenDocument-on-missing-file and
        // invalid-handle behaviour. Always run.
        assert_eq!(
            run_function_at_workspace_root("Mod/Tests/StoresProbe.cp", "OpenMissingFails"),
            1,
            "Stores.OpenDocument must return NIL for a missing file"
        );
        assert_eq!(
            run_function_at_workspace_root("Mod/Tests/StoresProbe.cp", "InvalidHandlesReturnZero"),
            1,
            "all Stores.* shims must return 0 / empty for invalid handles"
        );
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/StoresProbe.cp",
                "InvalidReaderHandlesReturnZero",
            ),
            1,
            "Reader shims must return 0 / EOF for invalid reader handles"
        );
    }

    #[test]
    fn stores_probe_reader_basic_cursor() {
        match stage_empty_odc_fixture() {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[stores_probe] skipping: {msg}");
                return;
            }
        }
        assert_eq!(
            run_function_at_workspace_root("Mod/Tests/StoresProbe.cp", "ReaderBasicCursor"),
            1,
            "Stores Reader cursor: Pos starts at 0, ReadByte advances, SetPos(0) seeks back"
        );
    }

    #[test]
    fn stores_probe_reader_eof_at_end() {
        match stage_empty_odc_fixture() {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[stores_probe] skipping: {msg}");
                return;
            }
        }
        assert_eq!(
            run_function_at_workspace_root("Mod/Tests/StoresProbe.cp", "ReaderEofAtEnd"),
            1,
            "ReaderSetPos(body_len) → ReaderEof = 1, over-seek clamps to body_len"
        );
    }

    #[test]
    fn stores_probe_reader_read_bytes_matches_byte_by_byte() {
        match stage_empty_odc_fixture() {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[stores_probe] skipping: {msg}");
                return;
            }
        }
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/StoresProbe.cp",
                "ReaderReadBytesMatchesByteByByte",
            ),
            1,
            "ReaderReadBytes must produce the same byte sequence as N ReadByte calls"
        );
    }

    #[test]
    fn kernel_probe_this_type_nil_when_unseen() {
        // ThisType returns NIL when the (module, type) pair has no
        // matching TypeDesc registered. Validates the "module known
        // but type unregistered" path Stores.ThisType falls through
        // to alien dispatch on.
        assert_eq!(
            run_function("Mod/Tests/KernelProbe.cp", "ThisTypeNilWhenUnseen"),
            1,
            "ThisType must return NIL for unseen (module, type) pairs"
        );
    }

    #[test]
    fn kernel_probe_this_type_finds_registered_type() {
        // KernelProbe's WidgetDesc / GadgetDesc are registered at
        // module-init time via the codegen-emitted __init_types
        // function. The probe verifies the registry mechanics —
        // CP-side compiled-module name registration is still
        // pending so we can't ThisMod("KernelProbe") yet, but the
        // type-init plumbing is exercised end-to-end.
        assert_eq!(
            run_function("Mod/Tests/KernelProbe.cp", "ThisTypeFindsRegisteredType"),
            1,
            "ThisType reflection plumb (registry populated by __init_types)"
        );
    }

    #[test]
    fn kernel_trap_cleaners_balanced_push_pop_runs_clean() {
        // Push two typed cleaners, pop them in matching reverse
        // order, observe that Cleanup did not fire (balanced
        // Pop drains the stack without invoking cleaners).
        assert_eq!(
            run_function("Mod/Tests/TrapCleanerProbe.cp", "BalancedPushPop"),
            1,
            "PushTrapCleaner / PopTrapCleaner must balance without firing Cleanup"
        );
        assert_eq!(
            run_function("Mod/Tests/TrapCleanerProbe.cp", "SingletonPushPop"),
            1,
            "single-cleaner Push then Pop must also balance"
        );
    }

    #[test]
    fn kernel_loop_quits_when_pre_armed() {
        // Both scenarios run in one test so they share a single
        // process-global QUIT_SIGNAL deterministically. cargo's
        // parallel test runner means concurrently calling
        // Kernel.Loop from two tests would race on the signal.
        // No GUI thread runs in the test process; the loop exits
        // cleanly because Quit is pre-armed before each Loop call.
        assert_eq!(
            run_function("Mod/Tests/KernelLoopProbe.cp", "RunOneShot"),
            1,
            "Kernel.Loop should exit cleanly when Quit is pre-armed"
        );
        assert_eq!(
            run_function("Mod/Tests/KernelLoopProbe.cp", "QuitBeforeAnyEvent"),
            1,
            "pre-armed Quit must skip handler invocation entirely"
        );
    }

    #[test]
    fn cross_module_inherited_concrete_method_dispatches() {
        // XMethodBase.BaseDesc has a concrete inherited method `Init(v)`
        // whose body lives in XMethodBase. XMethodChild.ChildDesc extends
        // BaseDesc and inherits Init without override. XMethodChild.Test
        // calls c.Init(21) via virtual dispatch, then c.Doubled() which
        // returns value*2. Expect 42.
        //
        // Until the cross-module vtable patcher lands, the inherited slot
        // points at __newcp_unimpl_method_trap and the call aborts.
        assert_eq!(run_function("Mod/Tests/XMethodChild.cp", "Test"), 42);
    }

    #[test]
    fn dump_heap_after_heaptest_reports_all_buckets() {
        // HeapTest.Run allocates 200 + 100 + 40 + 10 = 350 records of four
        // distinct types. The full snapshot must surface them all.
        let (output, code) = dump_heap(&["--after", "Mod/Tests/HeapTest.cp::Run"]);
        assert_eq!(
            code, 0,
            "dump-heap --after HeapTest.Run should succeed\noutput:\n{output}"
        );
        assert!(
            output.contains("alloc-lifetime:               350 blocks"),
            "expected lifetime alloc count 350\noutput:\n{output}"
        );
        assert!(
            output.contains("cluster-count: 1"),
            "expected one cluster grown\noutput:\n{output}"
        );
        assert!(
            output.contains("live 350 blocks"),
            "expected per-cluster walk to show 350 live blocks\noutput:\n{output}"
        );
        // Type catalog: four buckets, with the expected per-type instance
        // counts. Names render as `Type@0x...` until codegen co-emits names.
        assert!(
            output.contains("instances    200"),
            "expected 200-instance bucket (Tiny)\noutput:\n{output}"
        );
        assert!(
            output.contains("instances    100"),
            "expected 100-instance bucket (Small)\noutput:\n{output}"
        );
        assert!(
            output.contains("instances     40"),
            "expected 40-instance bucket (Mid)\noutput:\n{output}"
        );
        assert!(
            output.contains("instances     10"),
            "expected 10-instance bucket (Big)\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_heap_counters_only_is_lite() {
        // The --counters mode skips the per-block walk; only the counters
        // section appears, no clusters / types / module roots blocks.
        let (output, code) = dump_heap(&["--counters", "--after", "Mod/Tests/HeapTest.cp::Run"]);
        assert_eq!(code, 0, "dump-heap --counters should succeed\noutput:\n{output}");
        assert!(output.contains("counters:"));
        assert!(
            !output.contains("clusters:"),
            "--counters mode must not include the clusters section\noutput:\n{output}"
        );
        assert!(
            !output.contains("types ("),
            "--counters mode must not include the types section\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_heap_json_is_well_formed() {
        // JSON mode must emit a well-formed object containing the expected
        // top-level keys. We don't parse it (no serde dep here) — a substring
        // smoke check is enough to catch shape regressions.
        let (output, code) = dump_heap(&["--json", "--after", "Mod/Tests/HeapTest.cp::Run"]);
        assert_eq!(code, 0, "dump-heap --json should succeed\noutput:\n{output}");
        // Strip the leading "350\n" Console output before the JSON line.
        let json_line = output
            .lines()
            .find(|line| line.starts_with('{'))
            .expect("JSON line missing from dump-heap --json output");
        assert!(json_line.starts_with('{') && json_line.ends_with('}'));
        for key in [
            "\"counters\":",
            "\"clusters\":",
            "\"modules\":",
            "\"types\":",
            "\"alloc_blocks_lifetime\":350",
        ] {
            assert!(
                json_line.contains(key),
                "expected key {key} in JSON output\nline:\n{json_line}"
            );
        }
    }

    #[test]
    fn xmod_missing_field_rejected_by_sema() {
        // Cross-module field lookup must reject names that aren't
        // declared on the imported record. Previously sema's
        // validate_selector suppressed these for imported Named
        // types because lookup_record_member couldn't see fields
        // of cross-module records — that workaround stopped being
        // necessary once imported_modules was populated, and the
        // suppression turned into a silent accept that produced
        // malformed IR (codegen tried to load a nonexistent BOOLEAN
        // field as a pointer, then panicked).
        let err = loader_error("Mod/Tests/XmodMissingField.cp")
            .expect("expected sema to reject the missing cross-module field");
        assert!(
            err.contains("field thisFieldDoesNotExist does not exist"),
            "expected the missing-field diagnostic, got: {err}"
        );
    }

    #[test]
    fn xmod_missing_export_rejected_by_sema() {
        // Cross-module fall-through bug: a CP module referencing a
        // qualified name that the imported CP source doesn't export
        // must be rejected by sema with a clean diagnostic, not
        // silently accepted (which previously let codegen run and
        // emit a malformed cast). See feedback memory:
        // "Fix the compiler bug first".
        let err = loader_error("Mod/Tests/XmodMissingExport.cp")
            .expect("expected sema to reject the missing cross-module export");
        assert!(
            err.contains("module Stores has no exported declaration named DefinitelyNotAnExport"),
            "expected the missing-export diagnostic, got: {err}"
        );
    }

    #[test]
    fn host_stores_probe_basic_cursor() {
        match stage_empty_odc_fixture() {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[host_stores_probe] skipping: {msg}");
                return;
            }
        }
        assert_eq!(
            run_function_at_workspace_root("Mod/Tests/HostStoresProbe.cp", "BasicCursor"),
            1,
            "HostStores.Reader: NewReader, Pos, ReadByte, SetPos, Close"
        );
    }

    #[test]
    fn host_stores_probe_eof_transitions() {
        match stage_empty_odc_fixture() {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[host_stores_probe] skipping: {msg}");
                return;
            }
        }
        assert_eq!(
            run_function_at_workspace_root("Mod/Tests/HostStoresProbe.cp", "EofTransitions"),
            1,
            "HostStores.Reader.SetPos to body_len must transition to eof"
        );
    }

    #[test]
    fn host_stores_probe_split_qualified_name() {
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/HostStoresProbe.cp",
                "SplitNameRoundTrips",
            ),
            1,
            "SplitQualifiedName must split well-formed names and reject bad ones"
        );
    }

    #[test]
    fn host_stores_probe_new_store_by_name_allocates() {
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/HostStoresProbe.cp",
                "NewStoreByNameAllocates",
            ),
            1,
            "NewStoreByName must allocate a real instance whose type tag \
             matches a directly-NEW'd peer"
        );
    }

    #[test]
    fn host_stores_probe_new_store_by_name_rejects_bad_input() {
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/HostStoresProbe.cp",
                "NewStoreByNameRejectsBadInput",
            ),
            1,
            "NewStoreByName must return NIL for malformed / unresolved names"
        );
    }

    #[test]
    fn host_stores_probe_new_like_of_clones_type() {
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/HostStoresProbe.cp",
                "NewLikeOfClonesType",
            ),
            1,
            "NewLikeOf must allocate a fresh peer of the template's runtime type"
        );
    }

    #[test]
    fn host_stores_probe_typed_load_from_synthetic_odc() {
        // Stage a synthetic `.odc` whose root store has type name
        // "HostStoresProbe.BytePeekDesc" and body bytes [17, 42].
        // The CP probe opens it, NewStore allocates a typed BytePeek,
        // its Internalize override populates first/second, and the
        // probe verifies via a type-guarded field read. End-to-end
        // typed load.
        match stage_synthetic_odc(
            "Synthetic.odc",
            "HostStoresProbe.BytePeekDesc",
            &[17u8, 42u8],
        ) {
            Ok(()) => {}
            Err(msg) => {
                panic!("synthetic odc staging failed: {msg}");
            }
        }
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/HostStoresProbe.cp",
                "TypedLoadFromSyntheticOdc",
            ),
            1,
            "End-to-end typed load: synthetic .odc → NewStore → typed \
             field read must round-trip the body bytes"
        );
    }

    #[test]
    fn host_stores_probe_new_store_on_unknown_type_returns_nil() {
        match stage_empty_odc_fixture() {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[host_stores_probe] skipping: {msg}");
                return;
            }
        }
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/HostStoresProbe.cp",
                "NewStoreOnUnknownTypeReturnsNil",
            ),
            1,
            "NewStore on a store whose type isn't yet ported (e.g. \
             Documents.StdDocument) must return NIL cleanly"
        );
    }

    #[test]
    fn host_stores_probe_internalize_dispatches() {
        match stage_empty_odc_fixture() {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[host_stores_probe] skipping: {msg}");
                return;
            }
        }
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/HostStoresProbe.cp",
                "InternalizeDispatches",
            ),
            1,
            "BytePeek.Internalize override must dispatch through the \
             abstract HostStores.StoreDesc.Internalize when called via \
             InternalizeFrom"
        );
    }

    #[test]
    fn host_stores_probe_internalize_from_nil_store_sets_eof() {
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/HostStoresProbe.cp",
                "InternalizeFromNilStoreSetsEof",
            ),
            1,
            "InternalizeFrom on a NIL source store must report eof \
             without dispatching the abstract method"
        );
    }

    #[test]
    fn host_stores_probe_bulk_read_matches_byte_by_byte() {
        match stage_empty_odc_fixture() {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[host_stores_probe] skipping: {msg}");
                return;
            }
        }
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/HostStoresProbe.cp",
                "BulkReadMatchesByteByByte",
            ),
            1,
            "HostStores.Reader.ReadBytes must match the byte-by-byte sequence"
        );
    }

    #[test]
    fn string_array_compare_array_equals_literal() {
        assert_eq!(
            run_function("Mod/Tests/StringArrayCompare.cp", "ArrayEqualsLiteral"),
            1,
            "ARRAY OF CHAR with matching contents must `=` its string literal"
        );
    }

    #[test]
    fn string_array_compare_array_differs_from_literal() {
        assert_eq!(
            run_function("Mod/Tests/StringArrayCompare.cp", "ArrayDiffersFromLiteral"),
            1,
            "ARRAY OF CHAR with non-matching contents must `#` the string literal"
        );
    }

    #[test]
    fn string_array_compare_array_shorter_than_literal() {
        assert_eq!(
            run_function("Mod/Tests/StringArrayCompare.cp", "ArrayShorterThanLiteral"),
            1,
            "Terminator-mismatch (array shorter) must compare unequal"
        );
    }

    #[test]
    fn string_array_compare_array_longer_than_literal() {
        assert_eq!(
            run_function("Mod/Tests/StringArrayCompare.cp", "ArrayLongerThanLiteral"),
            1,
            "Terminator-mismatch (array longer) must compare unequal"
        );
    }

    #[test]
    fn string_array_compare_literal_equals_array() {
        assert_eq!(
            run_function("Mod/Tests/StringArrayCompare.cp", "LiteralEqualsArray"),
            1,
            "Operand order shouldn't matter — literal on the left must work too"
        );
    }

    #[test]
    fn string_array_compare_two_arrays_equal() {
        assert_eq!(
            run_function("Mod/Tests/StringArrayCompare.cp", "TwoArraysEqual"),
            1,
            "Two ARRAY OF CHAR with matching contents must compare equal regardless of address"
        );
    }

    #[test]
    fn string_array_compare_two_arrays_differ() {
        assert_eq!(
            run_function("Mod/Tests/StringArrayCompare.cp", "TwoArraysDiffer"),
            1,
            "Two ARRAY OF CHAR with differing contents must compare unequal"
        );
    }

    #[test]
    fn text_models_probe_type_resolves() {
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/TextModelsProbe.cp",
                "TypeResolves",
            ),
            1,
            "Kernel.ThisMod / ThisType must resolve TextModels.StdModelDesc \
             once the loader has materialized the module"
        );
    }

    #[test]
    fn text_models_probe_load_std_model() {
        // Body layout the probe expects: 6 super-version bytes
        // [0..5], then 4-byte LE run-list length = 7, then a
        // single-byte run-list terminator. Total 11 bytes; the
        // probe verifies the version-chain bytes round-trip and
        // the run-list length read correctly.
        let body = [0u8, 1, 2, 3, 4, 5, 7, 0, 0, 0, 0xFF];
        match stage_synthetic_odc(
            "TextModelsStub.odc",
            "TextModels.StdModelDesc",
            &body,
        ) {
            Ok(()) => {}
            Err(msg) => {
                panic!("synthetic odc staging failed: {msg}");
            }
        }
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/TextModelsProbe.cp",
                "LoadStdModel",
            ),
            1,
            "Synthetic TextModels.StdModelDesc fixture must load through \
             NewStore → Internalize → typed-field read"
        );
    }

    #[test]
    fn text_models_probe_load_std_model_text() {
        // Run-list length = 6 (one piece + terminator):
        //   ano = 1 (existing attribute, non-conforming on first
        //            piece but exercises the text-run branch),
        //   len = 5 (5 1-byte chars),
        //   ano = 0xFF terminator
        // chars buffer: "Hello"
        let mut body: Vec<u8> = Vec::new();
        body.extend_from_slice(&[0u8, 1, 2, 3, 4, 5]); // super-versions
        body.extend_from_slice(&6i32.to_le_bytes());   // run-list length
        body.push(1);                                   // ano = 1
        body.extend_from_slice(&5i32.to_le_bytes());   // len = 5
        body.push(0xFF);                                // terminator
        body.extend_from_slice(b"Hello");
        match stage_synthetic_odc(
            "TextModelsHello.odc",
            "TextModels.StdModelDesc",
            &body,
        ) {
            Ok(()) => {}
            Err(msg) => panic!("synthetic odc staging failed: {msg}"),
        }
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/TextModelsProbe.cp",
                "LoadStdModelText",
            ),
            1,
            "TextModels.StdModel.Internalize must decode the run list \
             and surface 'Hello' through the text buffer"
        );
    }

    #[test]
    fn text_models_probe_load_real_tour_odc() {
        // Tour.odc ships with the BlackBox distribution and is a
        // proper rich-text document — the embedded TextModels.StdModel
        // has at least one NEW attribute (the default char-attrs)
        // and many text pieces.  This exercises:
        //   - HostStores.Reader.SkipInlineStore over real wire bytes
        //   - the run-list decoder's full attribute-aware path
        //   - cross-module NEW + dispatch on a real BB document
        match stage_bb_odc_fixture("Tour.odc") {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[text_models_probe] skipping Tour.odc: {msg}");
                return;
            }
        }
        let result = run_function_at_workspace_root(
            "Mod/Tests/TextModelsProbe.cp",
            "LoadStdModelFromTourOdc",
        );
        assert_eq!(
            result, 1,
            "Tour.odc's first TextModels.StdModelDesc must Internalize \
             cleanly (1 = OkComplete; got {result}). With the inline-\
             store-skip primitive in place, NEW attributes no longer \
             stop the decoder."
        );
    }

    #[test]
    fn text_models_probe_tour_odc_summary() {
        // Asserts on the decoded summary: at least one NEW attribute
        // surfaced (the default char-attribute store) and at least
        // one text-run piece is present.
        match stage_bb_odc_fixture("Tour.odc") {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[text_models_probe] skipping Tour.odc: {msg}");
                return;
            }
        }
        let summary = run_function_at_workspace_root(
            "Mod/Tests/TextModelsProbe.cp",
            "TourOdcModelSummary",
        );
        assert!(summary >= 0, "summary returned a sentinel error: {summary}");
        let attr_growth = summary / 1_000_000;
        let text_pieces = (summary / 1_000) % 1_000;
        let view_pieces = summary % 1_000;
        assert!(
            attr_growth >= 1,
            "expected at least 1 NEW attribute in Tour.odc's text model (saw {attr_growth})"
        );
        assert!(
            text_pieces >= 1,
            "expected at least 1 text-run piece in Tour.odc's model (saw {text_pieces})"
        );
        eprintln!(
            "[text_models_probe] Tour.odc summary: attr_growth={attr_growth}, \
             text_pieces={text_pieces}, view_pieces={view_pieces}"
        );
    }

    #[test]
    fn text_models_probe_tour_odc_text_length() {
        match stage_bb_odc_fixture("Tour.odc") {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[text_models_probe] skipping Tour.odc: {msg}");
                return;
            }
        }
        let len = run_function_at_workspace_root(
            "Mod/Tests/TextModelsProbe.cp",
            "TourOdcTextLength",
        );
        assert!(len > 0, "Tour.odc should produce non-empty text (got len {len})");
        // Tour.odc is a substantial document; expect at least a
        // few hundred chars from its first text model.
        assert!(
            len >= 200,
            "expected at least 200 decoded chars from Tour.odc's first \
             TextModels.StdModel (got {len})"
        );
        eprintln!("[text_models_probe] Tour.odc text length = {len}");
    }

    #[test]
    fn text_models_probe_tour_odc_text_digest() {
        match stage_bb_odc_fixture("Tour.odc") {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[text_models_probe] skipping Tour.odc: {msg}");
                return;
            }
        }
        let digest = run_function_at_workspace_root(
            "Mod/Tests/TextModelsProbe.cp",
            "TourOdcTextDigest",
        );
        // Just assert it's a stable, non-trivial number — a real
        // text-content regression would change the digest. The
        // probe wraps i64 arithmetic, so any positive value
        // confirms the buffer was populated and the decoder
        // produced consistent bytes.
        assert!(digest != 0, "non-empty digest expected, got {digest}");
        eprintln!("[text_models_probe] Tour.odc first-32 digest = {digest}");
    }

    #[test]
    fn text_views_probe_type_resolves() {
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/TextViewsProbe.cp",
                "TypeResolves",
            ),
            1,
        );
    }

    #[test]
    fn text_views_probe_load_std_view_from_tour_odc() {
        // Recursive typed load: TextViews.StdView's Internalize
        // pulls the embedded TextModels.StdModel out of the same
        // wire stream by calling Reader.ReadInlineStore (handle)
        // and then HostStores.NewStore (typed materialization).
        // Both layers must reach OkComplete on a real document.
        match stage_bb_odc_fixture("Tour.odc") {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[text_views_probe] skipping Tour.odc: {msg}");
                return;
            }
        }
        let result = run_function_at_workspace_root(
            "Mod/Tests/TextViewsProbe.cp",
            "LoadStdViewFromTourOdc",
        );
        assert_eq!(
            result, 1,
            "Tour.odc's first TextViews.StdView must Internalize cleanly \
             with a populated TextModels.StdModel (got {result})"
        );
    }

    #[test]
    fn text_views_probe_tour_std_view_summary() {
        match stage_bb_odc_fixture("Tour.odc") {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[text_views_probe] skipping Tour.odc: {msg}");
                return;
            }
        }
        let summary = run_function_at_workspace_root(
            "Mod/Tests/TextViewsProbe.cp",
            "TourStdViewSummary",
        );
        assert!(summary > 0, "expected positive summary, got {summary}");
        let text_len = summary / 1_000_000;
        let org_plus_1 = (summary / 1_000) % 1_000;
        let dy_plus_1 = summary % 1_000;
        assert!(
            text_len >= 200,
            "expected at least 200 decoded chars in the embedded model \
             (got text_len = {text_len})"
        );
        eprintln!(
            "[text_views_probe] Tour.odc StdView summary: text_len={text_len}, \
             org={}, dy={}",
            org_plus_1 - 1,
            dy_plus_1 - 1
        );
    }

    #[test]
    fn text_models_probe_load_real_empty_odc() {
        // Walk a real BB Empty.odc, find its embedded
        // TextModels.StdModelDesc, materialize it via NewStore
        // and confirm Internalize either decoded cleanly or bailed
        // at a known-deferred feature (e.g. NEW attributes).
        // The probe encodes the outcome as an integer; assert on
        // the specific code so a regression points at the exact
        // path that surrendered.
        match stage_empty_odc_fixture() {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[text_models_probe] skipping real Empty.odc: {msg}");
                return;
            }
        }
        let result = run_function_at_workspace_root(
            "Mod/Tests/TextModelsProbe.cp",
            "LoadStdModelFromEmptyOdc",
        );
        // 1 = OkComplete, 25 = OkUnsupportedNewAttr (= 20 + 5).
        // Either is acceptable for this slice — the test just
        // requires that we found a model and dispatched. The
        // OkUnsupportedNewAttr branch unlocks once the inline-
        // child store-skip primitive lands. Reject 10 (no model
        // found) and 11 (NewStore returned NIL) firmly.
        // Empirically Empty.odc's TextModels.StdModel is an empty
        // text buffer (run list = terminator only, zero pieces),
        // so the decoder reaches OkComplete. If a future Empty.odc
        // grows a NEW attribute, this will need the inline child
        // store-skip primitive (deferred — see TextModels.cp).
        assert_eq!(
            result, 1,
            "Empty.odc's TextModels.StdModelDesc must Internalize cleanly \
             (1 = OkComplete; non-1 means a new wire-format feature \
             surfaced — see TextModels.OkXxx codes)"
        );
    }

    #[test]
    fn text_models_probe_load_std_model_empty() {
        // Terminator-only run list: run-list length = 1,
        //   ano = 0xFF
        // No chars.
        let mut body: Vec<u8> = Vec::new();
        body.extend_from_slice(&[0u8; 6]);
        body.extend_from_slice(&1i32.to_le_bytes());
        body.push(0xFF);
        match stage_synthetic_odc(
            "TextModelsEmpty.odc",
            "TextModels.StdModelDesc",
            &body,
        ) {
            Ok(()) => {}
            Err(msg) => panic!("synthetic odc staging failed: {msg}"),
        }
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/TextModelsProbe.cp",
                "LoadStdModelEmpty",
            ),
            1,
            "Empty (terminator-only) run list must decode as zero pieces"
        );
    }

    #[test]
    fn with_statement_dispatches_to_correct_arm_via_runtime_type_test() {
        // Two record types extend a common AnimalDesc abstract base.
        // Identify(a) uses `WITH a: Dog DO ... | a: Cat DO ... ELSE ...`
        // to pick a kind-specific accessor.  The runtime type test
        // (`__newcp_type_test`) walks the heap-block header at -16 to
        // read the dynamic TypeDesc tag and chases the base chain
        // looking for a match.  Run() exercises Dog (88), Cat (-7),
        // and NIL (0) in the same call:
        //   (88 * 1000) + ((-(-7)) * 10) + 0 = 88_070.
        assert_eq!(
            run_function("Mod/Tests/WithProbe.cp", "Run"),
            88_070,
        );
    }

    #[test]
    fn value_open_array_param_is_private_copy() {
        // CP §8.1: value-mode open-array params are private copies.
        // Mutate(a) writes p[0] := 99 in the callee; the caller's a[0]
        // must remain 7. Ports.DrawPath's inner Draw helper relies on
        // exactly this idiom — if it fails, that helper silently
        // mutates the user's IN array.
        assert_eq!(
            run_function("Mod/Tests/ValueOpenArrayProbe.cp", "Run"),
            7,
        );
    }

    #[test]
    fn models_copy_of_delegates_to_stores_copy_of() {
        // Models.CopyOf is no longer an identity stub — it delegates
        // to Stores.CopyOf which round-trips through the Externalize
        // / Internalize hooks. TaggedModel adds an INTEGER tag to
        // ModelDesc; the probe asserts the cloned tag survives a
        // mutation to the source. Returns 9907 on a true clone,
        // 9999 on identity-aliasing.
        assert_eq!(
            run_function("Mod/Tests/ModelsCopyOfProbe.cp", "Run"),
            9907,
        );
    }

    #[test]
    fn stores_copy_of_round_trips_a_concrete_subclass() {
        // BoxDesc extends Stores.StoreDesc with one INTEGER field
        // and overrides Externalize/Internalize to round-trip it.
        // CopyOf must produce a fresh heap object whose `value`
        // matches the source, and subsequent mutation of the source
        // must not leak. Probe returns 999_042 only on a true clone;
        // the old identity stub would have produced 999_999.
        assert_eq!(
            run_function("Mod/Tests/StoresCopyOfProbe.cp", "Run"),
            999_042,
        );
    }

    #[test]
    fn stores_writer_round_trip_through_in_memory_buffer() {
        // The in-memory Writer + buffer-sourced Reader is the
        // foundation Stores.CopyOf will sit on. Probe writes 1 byte
        // (42), 1 INTEGER (1234), 1 LONG (9_999_999_999), 1 BOOLEAN
        // (TRUE) into a writer, hands the buffer over to a reader,
        // drains it, and packs the values into 4_200_124_397 only
        // if every primitive round-tripped intact.
        assert_eq!(
            run_function("Mod/Tests/StoresWriterRoundTripProbe.cp", "Run"),
            4_200_124_397,
        );
    }

    #[test]
    fn pointer_alias_receivers_bind_to_underlying_record() {
        // Methods declared with the BlackBox-style pointer-alias
        // receiver (`(s: Sub) Method` where `Sub = POINTER TO SubDesc`)
        // should bind to the underlying record SubDesc — same as
        // writing `(s: SubDesc) Method`.  Run() allocates a Sub,
        // calls Greet(7) which is dispatched to SubDesc's override.
        // SubDesc.Greet sets tag=7, extra=70.  Returns
        // (tag * 100) + extra = 770.
        assert_eq!(
            run_function("Mod/Tests/PointerReceiverProbe.cp", "Run"),
            770,
        );
    }

    #[test]
    fn super_call_crosses_module_boundary_to_base_method() {
        // Cross-module super: ChildDesc (in SuperXmodProbe) extends
        // SuperBase.BaseDesc and overrides Bump.  The override
        // chains via `c.Bump^(v)` to the base in another module.
        // Same expectation as the same-module case: traceBase=3,
        // traceChild=30, packed as 330.
        assert_eq!(
            run_function("Mod/Tests/SuperXmodProbe.cp", "Run"),
            330,
        );
    }

    #[test]
    fn super_call_dispatches_to_base_method() {
        // ChildDesc.Bump overrides BaseDesc.Bump and chains via
        // `c.Bump^(v)` to the base implementation, then adds its own
        // contribution.  Run() allocates a Child, calls Bump(3):
        //   BaseDesc.Bump:  traceBase += 3        -> 3
        //   ChildDesc.Bump: traceChild += 3*10    -> 30
        // Returns (traceBase * 100) + traceChild = 330.
        assert_eq!(
            run_function("Mod/Tests/SuperProbe.cp", "Run"),
            330,
        );
    }

    #[test]
    fn models_dispatch_procs_forward_to_sequencer_or_fall_back() {
        // The probe wires up a TestSequencer + TestOp and calls each
        // of Models's Sequencer-driven dispatch procs (Do, BeginScript,
        // EndScript, Bunch) — verifying they all forward to the
        // installed sequencer.  Then it detaches the sequencer (passes
        // NIL) and calls Do again — the WITH ELSE branch fires, which
        // dispatches `op.Do()` (the abstract Stores.Operation method)
        // directly.  Returns:
        //   doCount × 1e5 + beginScriptCount × 1e4
        //     + endScriptCount × 1000 + bunchCount × 100 + opDoCount
        // Expect 111_101 — one of each Sequencer call plus one op.Do().
        assert_eq!(
            run_function("Mod/Tests/ModelsDispatchProbe.cp", "Run"),
            111_101,
        );
    }

    #[test]
    fn models_broadcast_dispatches_through_installed_sequencer() {
        // The probe installs a TestSequencer (cross-module-extending
        // Sequencers.SequencerDesc) on a TinyModel and broadcasts a
        // NeutralizeMsg twice.  Each Broadcast dispatches to the
        // sequencer's Handle via WITH on the model's ANYPTR `seq`
        // field; Handle then WITHs `msg` (VAR ANYREC) back to
        // Models.Message to read the era.  This exercises:
        //   * runtime IS test (`__newcp_type_test`) on heap subjects
        //     (the Sequencer ptr) AND stack subjects (the NeutralizeMsg
        //     record, via the shadow-header RTTI)
        //   * cross-module base patching at __init_types
        //   * vtable dispatch through the WITH-narrowed receiver
        // Run() returns:
        //   (Era(m) * 1_000_000) + (handleEra * 1000) + handleCount
        // Expect 2_002_002 = (2 × 1m) + (2 × 1k) + 2.
        assert_eq!(
            run_function("Mod/Tests/ModelsProbe.cp", "Run"),
            2_002_002,
        );
    }

    #[test]
    fn ports_frame_translates_user_to_device_coords() {
        // The probe sets up a Frame on a Port with unit = 100 and
        // offset gx = 50 / gy = 70, then calls Frame.DrawRect with
        // user-space (l=0, t=0, r=200, b=300, s=fill, col=red).
        // The frame divides every coordinate by `unit` after adding
        // the offset, so the rider sees:
        //   l = (50 + 0)   DIV 100 = 0
        //   t = (70 + 0)   DIV 100 = 0
        //   r = (50 + 200) DIV 100 = 2
        //   b = (70 + 300) DIV 100 = 3
        // Run() returns:
        //   (rectCallCount * 1_000_000) + (rectR * 1000) + (rectB * 100)
        //     + (rectColor MOD 1000)
        // Expect 1_002_555 = 1 × 1m + 2 × 1k + 3 × 100 + (red=255 MOD 1000).
        assert_eq!(
            run_function("Mod/Tests/PortsProbe.cp", "Run"),
            1_002_555,
        );
    }

    #[test]
    fn sequencers_notifier_chain_dispatches_in_lifo_order() {
        // The probe installs a TestDirectory, asks for a Sequencer via
        // Sequencers.dir.New(), hooks two notifiers (ids 1 and 2) via
        // InstallNotifier, then broadcasts a PingMsg with tag = 99.
        // InstallNotifier pushes onto the head of the chain, so the
        // firing order is LIFO: n2 runs first, then n1.  Each notifier
        // type-guards `msg` (VAR Sequencers.Message) against PingMsg
        // — exercises the runtime IS test on a stack-allocated record
        // backed by the shadow-header RTTI.  Run() returns:
        //   (notifyCount * 1_000_000) + (lastTag * 1_000)
        //     + (notifyTrace[0] * 10) + notifyTrace[1]
        // Expect 2_099_021 = 2×1m + 99×1k + (2 × 10) + 1.
        assert_eq!(
            run_function("Mod/Tests/SequencersProbe.cp", "Run"),
            2_099_021,
        );
    }

    #[test]
    fn finalizer_runs_when_block_reclaimed() {
        // The probe allocates 64 records of a type with a `Finalize`
        // method, drops every reference, calls Kernel.Collect, and
        // returns the delta (finalizers fired this call).  Runs
        // inside one `run_function` so the JIT module hosting the
        // TypeDesc stays loaded across the alloc → collect → drain
        // sequence.
        const N: i64 = 64;
        let delta = run_function("Mod/Tests/FinalizerProbe.cp", "AllocAndDrop");
        assert!(
            delta >= N,
            "expected at least {N} finalizers to fire, got delta={delta}"
        );
    }

    #[test]
    fn xmod_passthrough_compiles() {
        // CP MODULE → DEFINITION MODULE call with an open-array IN
        // argument forwarded across modules. This is the path the
        // typed Stores.Reader facade needs; if this fails, the
        // facade idea won't fly.
        match stage_empty_odc_fixture() {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[xmod_passthrough] skipping: {msg}");
                return;
            }
        }
        assert_eq!(
            run_function_at_workspace_root(
                "Mod/Tests/XmodPassthroughCaller.cp",
                "Run",
            ),
            1,
        );
    }

    #[test]
    fn local_const_module_dim_works() {
        // Module-level CONST as ARRAY bound — known good baseline.
        assert_eq!(
            run_function("Mod/Tests/LocalConstArrayDim.cp", "ModuleConstDim"),
            34,
        );
    }

    #[test]
    fn local_const_value_works() {
        // Local CONST in an expression position — also works.
        assert_eq!(
            run_function("Mod/Tests/LocalConstArrayDim.cp", "LocalConstValue"),
            16,
        );
    }

    #[test]
    fn local_const_array_dim_works() {
        // Procedure-scoped CONST as ARRAY bound — was the failure
        // case. Expect a 4-element round-trip after the fix.
        assert_eq!(
            run_function("Mod/Tests/LocalConstArrayDim.cp", "LocalConstDim"),
            34,
        );
    }

    #[test]
    fn local_const_array_len_works() {
        assert_eq!(
            run_function("Mod/Tests/LocalConstArrayDim.cp", "LocalConstLen"),
            4,
        );
    }

    /// Cross-module vtable workout via Views.ViewDesc extension.
    ///
    /// `ViewExtBase.CountingView` extends `Views.ViewDesc`, overrides
    /// `Restore` (ABSTRACT in Views) and `ThisModel` (EXTENSIBLE in
    /// Views — with a super-call to the parent module's default).
    /// `Run` widens to the base `Views.View` pointer and dispatches
    /// both methods through the vtable, then packs the recorded
    /// paint rectangle + dispatch counter into a single int.
    ///
    /// 21234 = paintCount(20) * 1000 + lastL(1)*1000 + lastT(2)*100
    ///       + lastR(3)*10 + lastB(4)
    #[test]
    fn views_extension_chain_dispatches_through_vtable() {
        assert_eq!(
            run_function("Mod/Tests/ViewExtBase.cp", "Run"),
            21234,
        );
    }

    /// Three-level super-call chain crossing two modules.
    ///
    /// `ViewExtSuper.TaggedView` extends `Views.View` which extends
    /// `Stores.Store`.  `TaggedView.Internalize` super-calls
    /// `Views.View.Internalize` which super-calls
    /// `Stores.Store.Internalize` (EMPTY at the base).  The driver
    /// invokes the override twice — once through the concrete
    /// pointer (static dispatch), once through the widened
    /// `Views.View` base pointer (virtual dispatch via vtable).
    /// Both should land in the subclass method body and increment
    /// the counter.
    #[test]
    fn views_super_call_chain_resolves_through_two_modules() {
        assert_eq!(
            run_function("Mod/Tests/ViewExtSuper.cp", "Run"),
            2,
        );
    }

    /// 3-level cross-module vtable workout through Containers.
    ///
    /// Type chains:
    ///   Stores.Store -> Models.Model -> Containers.Model -> MyModel
    ///   Stores.Store -> Views.View  -> Containers.View  -> MyView
    ///
    /// Exercises:
    /// - `Containers.View.InitModel` dispatching to overridden
    ///   `AcceptableModel` via the cross-module vtable;
    /// - 3-level super-call chains on Internalize (concrete leaf
    ///   -> Containers layer -> Models/Views layer);
    /// - both static-bound and virtual-bound entry points reach
    ///   the same override.
    ///
    /// Packed expected value 111124 confirms each stage fired.
    #[test]
    fn containers_three_level_vtable_chain_dispatches_through_modules() {
        assert_eq!(
            run_function("Mod/Tests/ContainerExtBase.cp", "Run"),
            111124,
        );
    }

    /// Controllers.Controller extension + Containers.Controller chain.
    ///
    /// Workout for the cross-module controller hierarchy:
    /// - leaf `MyControllerDesc` extending `Controllers.ControllerDesc`
    ///   overrides `Domain` (inherited via Stores.Store) and dispatches
    ///   through the vtable;
    /// - `BoundControllerDesc` extending `Containers.ControllerDesc`
    ///   widens to both `Containers.Controller` and the inherited
    ///   `Controllers.Controller` base, proving the type identity
    ///   stitches Containers -> Controllers -> Stores together;
    /// - a `TaggedEditMsg` reads back fields inherited through
    ///   the RequestMessage chain.
    ///
    /// Expected packed result 118199 — see the procedure body for
    /// the exact decomposition.
    #[test]
    fn controllers_extension_chain_dispatches_through_modules() {
        assert_eq!(
            run_function("Mod/Tests/ControllerExtBase.cp", "Run"),
            118199,
        );
    }

    /// Verify a record with an inline (anonymous) record-typed
    /// field allocates the right struct slot and field access
    /// hits the right offsets.  BlackBox uses this shape for
    /// `Properties.StdProp.style: RECORD val, mask: SET END`.
    #[test]
    fn inline_record_as_record_field_works() {
        assert_eq!(
            run_function("Mod/Tests/InlineFieldProbe.cp", "Run"),
            1119,
        );
    }

    /// Properties.PropertyDesc extension + StdProp inline-record
    /// field round-trip.
    ///
    /// Workout for:
    /// - extending an ABSTRACT `Properties.PropertyDesc` with a leaf
    ///   and overriding its `IntersectWith` ABSTRACT method;
    /// - reading/writing the inline-record `style: RECORD val, mask:
    ///   SET END` field on `StdProp` — exercises the new
    ///   `__anon_inline_` content-hash Named-type path.
    #[test]
    fn properties_extension_and_inline_field_round_trip() {
        assert_eq!(
            run_function("Mod/Tests/PropertiesExtBase.cp", "Run"),
            1001911,
        );
    }

    /// BB-faithful TextViews-style slice round-tripping through
    /// `Stores.CopyOf`.
    ///
    /// Type chain:
    ///   Stores.Store -> Views.View -> Containers.View -> BbView (leaf)
    ///
    /// This is the integration test the Properties/Controllers/
    /// Containers/Views/Stores.Reader extensions were building
    /// toward: a 4-level inheritance chain rooted at Stores.Store,
    /// crossing four modules, surviving a full
    /// `Externalize -> in-memory buffer -> Internalize` round-trip
    /// with every layer's super-call firing in order.
    ///
    /// The leaf's `Externalize2` / `Internalize2` body hooks
    /// run last, mirroring `TextViews.StdView`'s structure.
    /// Packed expected value 1042017 = hideMarks*1e6 +
    /// org*1000 + dy on `(hideMarks=TRUE, org=42, dy=17)`.
    #[test]
    fn textviews_bb_faithful_chain_round_trips_through_copyof() {
        assert_eq!(
            run_function("Mod/Tests/TextViewsBbExt.cp", "Run"),
            1042017,
        );
    }

    /// Services.Action extension + deferred-scheduler round-trip.
    ///
    /// Workout for the new Services slice:
    /// - `CounterAction` extends `Services.ActionDesc` (ABSTRACT)
    ///   and overrides `Do` — the override fires via vtable
    ///   dispatch from inside `Services.Step`;
    /// - `DoLater` with `now` / `immediately` schedules actions;
    /// - `RemoveAction` cancels a pending action so it doesn't
    ///   fire;
    /// - a second `Step` on an empty queue is a no-op (idempotency).
    ///
    /// Expected 1022 = firedCount(1)*1000 + lastFired(22).
    #[test]
    fn services_action_scheduler_dispatches_through_vtable() {
        assert_eq!(
            run_function("Mod/Tests/ServicesExtBase.cp", "Run"),
            1022,
        );
    }

    /// Verifies the sema constant folder resolves a CONST
    /// expression whose operand is a CONST imported from
    /// another module.  Used to silently drop the receiver
    /// from the symbol table, surfacing later as "identifier X
    /// is not declared" at every use site (e.g. Services.scale
    /// referencing Kernel.timeResolution).
    #[test]
    fn imported_const_is_foldable_in_derived_const() {
        assert_eq!(
            run_function("Mod/Tests/ImportedConstProbe.cp", "Run"),
            500,
        );
    }

    /// Parameterless method call on a local of pointer-alias-to-
    /// abstract type — `victim.Fire;` (no parens) on a `Base`
    /// pointing at a `Leaf`.  Used to mis-route through the
    /// `is_bare_proc_call` path and emit a Call to thin air;
    /// now lands on the bound-method vtable dispatch.
    #[test]
    fn parameterless_method_call_on_local_dispatches_through_vtable() {
        assert_eq!(
            run_function("Mod/Tests/AbstractLocalCallProbe.cp", "Run"),
            99,
        );
    }

    /// Mechanisms.Hook trampoline + cross-module hook
    /// installation.
    ///
    /// `MyHookDesc` extends `Mechanisms.HookDesc` and overrides
    /// every ABSTRACT method.  `Mechanisms.SetHook` installs
    /// it.  The driver then calls three trampolines —
    /// `MarkFocusBorder`, `FocusBorderCursor`, `SelBorderCursor`
    /// — and verifies the overrides fire with the right
    /// arguments via cross-module virtual dispatch.
    ///
    /// Expected packed value 112234.
    #[test]
    fn mechanisms_hook_trampoline_dispatches_through_vtable() {
        assert_eq!(
            run_function("Mod/Tests/MechanismsExtBase.cp", "Run"),
            112234,
        );
    }

    /// TextMappers.Scanner / Formatter ConnectTo + cursor
    /// round-trip through a concrete fake Reader / Writer /
    /// Model trio.  Exercises cross-module vtable dispatch
    /// from TextMappers calls (`s.SetPos`, `f.WriteChar`,
    /// etc.) into the concrete leaf overrides defined in the
    /// test fixture.
    ///
    /// Expected 17342 = 17 (posAfterSeek)*1000 + 3 (posAfterWrite)*100
    ///                + 42 (m.simulatedLength).
    #[test]
    fn textmappers_scanner_formatter_round_trip_through_concrete_io() {
        assert_eq!(
            run_function("Mod/Tests/TextMappersExtBase.cp", "Run"),
            17342,
        );
    }

    /// Pointer-alias receiver type-checks as a pointer in
    /// expression contexts.  `(a: Foo) Eq (b: Foo) ... a = b`
    /// — inside the method body, `a` is the pointer (Foo),
    /// not the underlying record (FooDesc).  Used to fail
    /// with "invalid operands for =: RECORD … and POINTER
    /// TO …" because sema canonicalised the receiver to the
    /// descriptor for all uses, including value-comparison.
    #[test]
    fn pointer_alias_receiver_compares_as_pointer() {
        assert_eq!(
            run_function("Mod/Tests/PtrReceiverEqProbe.cp", "Run"),
            1,
        );
    }

    /// TextRulers extension test — four-level cross-module
    /// chain (Stores → Models → TextRulers.Style → MyStyle),
    /// concrete Directory factories dispatching via vtable,
    /// CopyTabs through an inline 32-Tab fixed array, and a
    /// Prop with the inline `opts: RECORD val, mask: SET END`
    /// field.
    ///
    /// Expected 102123 (now that #34 closed: per-tab
    /// `tabsAfter.tab[1].stop` resolves cleanly).
    #[test]
    fn textrulers_directory_factories_and_tabs_round_trip() {
        assert_eq!(
            run_function("Mod/Tests/TextRulersExtBase.cp", "Run"),
            102123,
        );
    }

    /// Repro / regression for deferred_fixes #34 — accessing
    /// a field of an array element where the element type
    /// is a cross-module Named record.  `bag.items[0].stop`
    /// where `Bag.items: ARRAY 8 OF Entry` and Entry lives
    /// in an imported module.
    #[test]
    fn field_of_array_element_of_xmod_record() {
        assert_eq!(
            run_function("Mod/Tests/ArrayOfXModRecordProbe.cp", "Run"),
            1122,
        );
    }

    /// Repro / regression for deferred_fixes #33 — chained
    /// method call where the inner method's return type
    /// lives in another module.  `o.Pick().Total()` where
    /// `Pick` returns a `ChainedXModCallBase.Inner` and
    /// `Total` is a method on that Inner.
    #[test]
    fn chained_xmod_method_call_resolves() {
        assert_eq!(
            run_function("Mod/Tests/ChainedXModCallProbe.cp", "Run"),
            42,
        );
    }

    /// TextSetters extension — concrete Directory / Setter /
    /// Reader leaves implementing every ABSTRACT method,
    /// vtable dispatch through Directory.New + Setter.NewReader +
    /// Setter.NextSequence/PreviousSequence, plus a LineBox
    /// with inline-array-of-INTEGER `tabW` field-access (the
    /// pattern #34 fixed).
    ///
    #[test]
    /// BRK statement: snapshot-style debugger breakpoint.  The dump
    /// goes to stderr (not captured here); the procedure resumes
    /// after BRK and returns `7 * 6 = 42`.  This test verifies that
    /// the BRK call doesn't trap, doesn't corrupt locals, and that
    /// the surrounding state survives the snapshot.
    #[test]
    fn brk_statement_returns_normally() {
        assert_eq!(run_function("Mod/BrkProbe.cp", "Run"), 42);
    }

    /// BRK(pointer) used to inspect MVC framework wiring at three
    /// points: fresh Pane, fresh StdCtrl, after InitView2 binds.
    /// The stderr dump shows TypeDesc + payload bytes at each point;
    /// run with `cargo test -- --nocapture` to read it.  Test passes
    /// if the procedure completes normally (the inspection is the
    /// product, not the return value).
    #[test]
    fn brk_mvc_probe_dumps_pane_and_controller() {
        assert_eq!(run_function("Mod/Tests/BrkMvcProbe.cp", "Run"), 1);
    }

    /// Out module smoke test — BB-faithful textual-output API
    /// routed through Console.  Exercises Open/Char/Ln/String/
    /// Int/Real on a captured Console buffer.  Test passes if
    /// the procedure returns 1 (every path emits without trapping).
    #[test]
    fn out_module_routes_to_console_without_trapping() {
        assert_eq!(run_function("Mod/Tests/OutProbe.cp", "Run"), 1);
    }

    /// In module smoke test — BB-faithful textual-input API.
    /// Controllers.FocusView returns NIL in this slice (no GUI
    /// focus routing yet), so In.Open sets Done := FALSE and
    /// every read proc becomes a no-op.  Verifies that Char /
    /// Int / LongInt / Real / String all respect the Done
    /// flag and leave their OUT slots untouched.
    #[test]
    fn in_module_no_focus_path_no_ops_read_procs() {
        assert_eq!(run_function("Mod/Tests/InProbe.cp", "Run"), 1);
    }

    /// Scanner round-trip — writes a multi-token stream into a
    /// TextModels.Doc through a Formatter, then reads it back
    /// through a Scanner.  Verifies positive int, negative int,
    /// double-quoted and single-quoted strings, lone-sign-as-char,
    /// punctuation fallthrough, and EOT detection.  Expected =
    /// 2 ints * 10000 + 2 strings * 100 + 2 chars = 20202.
    #[test]
    fn textmappers_scan_round_trip_through_doc_buffer() {
        assert_eq!(run_function("Mod/Tests/ScanProbe.cp", "Run"), 20202);
    }

    /// Repro for the bare-method-call dispatch bug.  CP allows
    /// `r.Touch` (no parens) when Touch is a parameterless
    /// method on r.  Without the fix, this lowers to a no-op
    /// designator-as-statement and r.x stays 0.
    #[test]
    fn bare_method_call_dispatches_without_parens() {
        assert_eq!(run_function("Mod/Tests/TestBareCall.cp", "Run"), 42);
    }

    /// Cross-module repro of the bare-method-call bug.  Imports
    /// a record + method from another module; calls
    /// `r.Touch` (no parens).
    #[test]
    fn bare_method_call_xmod_dispatches_without_parens() {
        assert_eq!(run_function("Mod/Tests/BareCallXmodProbe.cp", "Run"), 42);
    }

    /// VAR-receiver method calling a VAR-param top-level proc,
    /// passing the receiver through.  The proc mutates a field;
    /// the mutation must survive at the call site.
    #[test]
    fn var_receiver_method_passes_self_to_var_param_proc() {
        assert_eq!(run_function("Mod/Tests/VarRefChainProbe.cp", "Run"), 42);
    }

    /// POINTER-TO-record local passed as `p^` to a VAR record
    /// argument.  Tests whether the dereferenced pointer
    /// correctly identifies the heap record for VAR mutation.
    /// XXX deferred: `Selector::Dereference` isn't handled in
    /// `designator_addr`, so `p^` passed by VAR loses the deref
    /// hop.  Needs broader audit of pointer-deref paths in IR
    /// lowering — currently triggers STATUS_ACCESS_VIOLATION.
    #[test]
    #[ignore]
    fn ptr_local_dereferenced_as_var_record_arg() {
        assert_eq!(run_function("Mod/Tests/PtrVarArgProbe.cp", "Run"), 42);
    }

    /// Sanity: comparing a CHAR variable to a single-quoted char
    /// literal `"'"`.  Should produce TRUE when the variable
    /// holds the single-quote char (27X).
    #[test]
    fn char_compare_against_single_quote_literal() {
        assert_eq!(run_function("Mod/Tests/SingleQuoteCharProbe.cp", "Run"), 1);
    }

    /// Converters smoke test — BB-faithful file-format dispatch
    /// registry.  Registers three converters (odc / txt / rtf)
    /// and walks the global list.  Expected: 3 * 1000 + 1 = 3001.
    /// Previously hung in LLVM emit because
    /// `flatten_sem_type_fields` in lower.rs recursed unbounded
    /// through named-type cycles — fix landed alongside this port.
    #[test]
    fn converters_register_builds_linked_list() {
        assert_eq!(run_function("Mod/Tests/ConvertersProbe.cp", "Run"), 3001);
    }

    /// Meta MVS smoke — calling Meta.LookupPath returns an Item
    /// with obj = undef in this slice (real reflection deferred).
    #[test]
    fn meta_lookup_path_returns_undef_in_first_slice() {
        assert_eq!(run_function("Mod/Tests/MetaProbe.cp", "Run"), 1);
    }

    /// Documents MVS smoke — dir / stdDir start NIL, the stub
    /// ImportDocument returns s := NIL without trapping.  Full
    /// .odc decoder lands once Stores.Reader grows ReadVersion /
    /// ReadStore.
    #[test]
    fn documents_mvs_surface_loads() {
        assert_eq!(run_function("Mod/Tests/DocumentsProbe.cp", "Run"), 1);
    }

    /// Windows MVS smoke — surface compiles + dir/stdDir start
    /// NIL + the flag constants assemble into a SET cleanly +
    /// SelectByTitle stub returns done = FALSE.
    #[test]
    fn windows_mvs_surface_loads() {
        assert_eq!(run_function("Mod/Tests/WindowsProbe.cp", "Run"), 1);
    }

    /// HostWindows MVS smoke — module init installs the
    /// StdDirectory into Windows.dir, an empty directory's
    /// First / Focus return NIL, and Directory.New allocates a
    /// StdWindow.
    #[test]
    fn host_windows_installs_directory() {
        assert_eq!(run_function("Mod/Tests/HostWindowsProbe.cp", "Run"), 1);
    }

    #[test]
    #[ignore = "manual repro for loader/JIT hang on Windows.dir.First() cross-module dispatch"]
    fn host_windows_directory_first_dispatch_repro() {
        let probe_path = workspace_root()
            .join("Mod")
            .join("Tests")
            .join("HostWindowsDirFirstDispatch.cp");

        std::fs::write(
            &probe_path,
            concat!(
                "MODULE HostWindowsDirFirstDispatch;\n",
                "IMPORT Windows, HostWindows;\n",
                "PROCEDURE Run* (): INTEGER;\n",
                "BEGIN\n",
                "  IF Windows.dir = NIL THEN RETURN -1 END;\n",
                "  IF Windows.stdDir = NIL THEN RETURN -2 END;\n",
                "  IF Windows.dir.First() # NIL THEN RETURN -3 END;\n",
                "  RETURN 1\n",
                "END Run;\n",
                "END HostWindowsDirFirstDispatch.\n"
            ),
        )
        .expect("failed to write HostWindowsDirFirstDispatch.cp");

        let result = run_function("Mod/Tests/HostWindowsDirFirstDispatch.cp", "Run");

        let _ = std::fs::remove_file(&probe_path);

        assert_eq!(result, 1);
    }

    fn inline_fixed_array_field_in_xmod_record() {
        assert_eq!(
            run_function("Mod/Tests/InlineFixedArrayProbe.cp", "Run"),
            -3,
        );
    }

    /// Expected 119425.
    #[test]
    fn textsetters_directory_dispatches_and_linebox_round_trips() {
        assert_eq!(
            run_function("Mod/Tests/TextSettersExtBase.cp", "Run"),
            119425,
        );
    }

    /// Smoke-test the TextControllers first slice: import the abstract
    /// surface, exercise the public message records (SetCaretMsg /
    /// SetSelectionMsg field round-trip), and verify the BB-faithful
    /// `none` constant equals -1.  Encoded result:
    ///   pos=42, beg=1, end=5  →  42*10000 + 1*100 + 5 = 420105
    ///   plus 1 for the `none = -1` constant check          = 420106
    #[test]
    fn textcontrollers_first_slice_abstract_surface_loads() {
        assert_eq!(
            run_function("Mod/Tests/TextControllersSmoke.cp", "Run"),
            420106,
        );
    }

    /// End-to-end exercise of the concrete `StdCtrl` + `StdDirectory`
    /// bodies via the abstract Controller / Directory dispatch chain.
    /// Five checks, each contributing a decimal place:
    ///   1     — fresh controller's CaretPos() = none
    ///   10    — SetCaret(42) → CaretPos() = 42
    ///   100   — SetCaret(none) → CaretPos() = none
    ///   1000  — SetSelection(7,19) → GetSelection(7,19)
    ///   10000 — SetSelection(3,3) → GetSelection(3,3) (empty range)
    #[test]
    fn textcontrollers_stdctrl_caret_and_selection_round_trip() {
        assert_eq!(
            run_function("Mod/Tests/TextControllersStdCtrl.cp", "Run"),
            11111,
        );
    }

    /// First slice where a concrete editor pane (Pane) and a
    /// concrete controller (StdCtrl) bind to each other through
    /// the framework's abstract Views.View / Controller dispatch
    /// chain.  Six checks, bit-encoded:
    ///   1  — dir.New(NIL) returns non-NIL
    ///   2  — fresh Pane has org=0, dy=0, hideMarks=FALSE
    ///   4  — DisplayMarks(TRUE) flips hideMarks via abstract dispatch
    ///   8  — DisplayMarks(FALSE) flips back
    ///   16 — SetOrigin/PollOrigin round-trip via abstract dispatch
    ///   32 — InitView2 binds StdCtrl.view to the Pane
    #[test]
    fn textviews_pane_binds_to_textcontrollers_stdctrl() {
        assert_eq!(
            run_function("Mod/Tests/TextViewsPaneCtrl.cp", "Run"),
            63,
        );
    }

    /// First-pixels: Pane.Restore on an unbound pane emits exactly
    /// one DrawRect (the white background fill).  Verified via a
    /// recording Rider attached to a TestFrame.  Four checks:
    ///   1 — exactly one DrawRect call
    ///   2 — color = Ports.white
    ///   4 — rect = (0, 0, 800, 600) (matches the dirty rect)
    ///   8 — stroke = Ports.fill
    #[test]
    fn textviews_pane_restore_unbound_emits_background_fill() {
        assert_eq!(
            run_function("Mod/Tests/TextViewsPanePixels.cp", "RestoreUnboundEmitsBackground"),
            15,
        );
    }

    /// First-pixels: Pane.Restore on a model-bound pane emits the
    /// background fill PLUS the top-edge indicator bar.  Two
    /// DrawRect calls in order (white, then black).  Four checks:
    ///   1 — exactly two DrawRect calls
    ///   2 — first call color = white
    ///   4 — second call color = black
    ///   8 — second call rect = (0, 0, 800, 50) (top-edge band)
    #[test]
    fn textviews_pane_restore_bound_emits_background_and_bar() {
        assert_eq!(
            run_function("Mod/Tests/TextViewsPanePixels.cp", "RestoreBoundEmitsBackgroundAndBar"),
            15,
        );
    }

    /// Real text rendering: Pane bound to a concrete Doc
    /// populated via DocWriter.WriteString.  Pane.Restore opens a
    /// Reader, walks the chars, and emits one DrawString call
    /// containing "Hello, pixels!".  Four bit-encoded checks:
    ///   1 — scaffold landed (2 DrawRect calls)
    ///   2 — one DrawString call
    ///   4 — DrawString color = black
    ///   8 — captured text matches "Hello, pixels!\0"
    /// Real text rendering: Pane bound to a concrete Doc populated
    /// via DocWriter.WriteString.  Pane.Restore opens a Reader,
    /// walks all chars, and emits one DrawString call containing
    /// "Hello, pixels!".  Four bit-encoded checks:
    ///   1 — scaffold landed (2 DrawRect calls)
    ///   2 — one DrawString call
    ///   4 — DrawString color = black
    ///   8 — captured text matches "Hello, pixels!\0" exactly
    #[test]
    fn textviews_pane_restore_renders_doc_text_via_drawstring() {
        assert_eq!(
            run_function("Mod/Tests/TextViewsPanePixels.cp", "RestoreBoundEmitsText"),
            15,
        );
    }

    /// HostPorts → HostPortsSys → iGui dispatch chain.  No iGui
    /// window opened — the test pushes commands into iGui's batch
    /// queue and verifies the chain reached iGui without trapping.
    /// Seven bit-encoded checks:
    ///   1   — HostPort.Init sets unit + size
    ///   2   — NewRider returned non-NIL
    ///   4   — Rider.Base() round-trips to the port
    ///   8   — UnpackColor produces (1,0,0,1) for Ports.red
    ///   16  — DrawRect → HostPortsSys.FillRect → iGui returned cleanly
    ///   32  — DrawLine returned cleanly
    ///   64  — DrawString → narrow + font params + iGui returned cleanly
    /// Expected: 127.
    #[test]
    fn hostports_rider_forwards_paint_through_igui() {
        assert_eq!(
            run_function("Mod/Tests/HostPortsSmoke.cp", "Run"),
            127,
        );
    }

}
