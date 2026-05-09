#[cfg(test)]
mod tests {
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
        assert!(
            output.contains("define i1 @Contains(%Rect %0, %Point %1)")
                && output.contains("getelementptr inbounds %Point, ptr %p, i32 0, i32 0")
                && output.contains("getelementptr inbounds %Rect, ptr %r, i32 0, i32 0"),
            "expected mixed Point/Rect accesses to keep the correct parent struct types\noutput:\n{output}"
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
            output.contains("@__newcp_typedesc_Sub"),
            "expected TypeDesc sentinel global for Sub\noutput:\n{output}"
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
        // NEW(d) should call __newcp_sys_new
        assert!(
            output.contains("call ptr @__newcp_sys_new"),
            "expected NEW(d) to emit call to __newcp_sys_new\noutput:\n{output}"
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

    #[test]
    fn strings_real_to_short_str_round_trip() {
        // Format into SHORTCHAR buffer (via Narrow) then parse back (via Widen).
        // RealToShortStr(7.5) -> "7.5..." -> ShortStrToReal -> 7.5 -> ENTIER 7.
        // Exercises both byte<->wide bridges for the real-number procs.
        assert_eq!(run_function("Mod/Tests/MathSmoke.cp", "RealToShortStrRoundTrip"), 7);
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
}
