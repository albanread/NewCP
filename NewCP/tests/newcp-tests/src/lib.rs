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

    fn dump_llvm(path: &str) -> (String, i32) {
        let out = Command::new(driver_bin())
            .args(["dump-llvm", path])
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

    fn dump_llvm_source(file_name: &str, source: &str) -> (String, i32) {
        let path = std::env::temp_dir().join(file_name);
        std::fs::write(&path, source).expect("failed to write temporary source module");
        let out = Command::new(driver_bin())
            .args(["dump-llvm", path.to_str().expect("temporary source path should be UTF-8")])
            .current_dir(workspace_root())
            .output()
            .expect("failed to spawn driver binary");
        let _ = std::fs::remove_file(&path);

        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let code = out.status.code().unwrap_or(-1);
        (stdout, code)
    }

    #[test]
    fn check_mod_empty_is_clean() {
        let (output, code) = check_mod("Empty");
        assert_eq!(code, 0, "expected exit 0 for Empty.cp\noutput:\n{output}");
        assert!(
            output.trim_end().ends_with("ok"),
            "expected 'ok' in output for Empty.cp\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_vars_uses_module_globals() {
        let (output, code) = dump_llvm("Mod/Vars.cp");
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
        let (output, code) = dump_ir("Mod/Records.cp");
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
        let (output, code) = dump_llvm("Mod/Records.cp");
        assert_eq!(code, 0, "expected exit 0 for Records.cp\noutput:\n{output}");
        assert!(
            output.contains("define void @SetPoint(ptr %0, i64 %1, i64 %2)")
                && output.contains("getelementptr inbounds %Point, ptr %t1, i32 0, i32 0")
                && output.contains("getelementptr inbounds %Point, ptr %t4, i32 0, i32 1"),
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
        let (output, code) = dump_llvm("Mod/Strs.cp");
        assert_eq!(code, 0, "expected exit 0 for Strs.cp\noutput:\n{output}");
        assert!(
            output.contains("@.str.0 = private constant [6 x i8] c\"hello\\00\""),
            "expected private null-terminated string constant\noutput:\n{output}"
        );
        assert!(
            output.contains("call void @StrBase.Print(ptr @.str.0)"),
            "expected ConstStr passed as ptr to call\noutput:\n{output}"
        );
    }

    #[test]
    fn dump_llvm_calls_emits_direct_calls() {
        let (output, code) = dump_llvm("Mod/Calls.cp");
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
        let (output, code) = dump_llvm("Mod/ImportUse.cp");
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
        let (output, code) = dump_llvm("Mod/VarUse.cp");
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
        let (output, code) = dump_llvm("Mod/Pointers.cp");
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
        let (output, code) = dump_llvm("Mod/CaseWith.cp");
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
        let (output, code) = dump_llvm("Mod/TypeExt.cp");
        assert_eq!(code, 0, "expected exit 0 for TypeExt.cp\noutput:\n{output}");
        // struct layout: %Bird = { i64, i1 }
        assert!(
            output.contains("%Bird = type { i64, i1 }"),
            "expected Bird struct with inherited i64 legs and own i1 canFly\noutput:\n{output}"
        );
        // b.legs := 2 → gep %Bird index 0
        assert!(
            output.contains("getelementptr inbounds %Bird, ptr %t0, i32 0, i32 0"),
            "expected b.legs store to use GEP index 0 into %%Bird\noutput:\n{output}"
        );
        // b.canFly := canFly → gep %Bird index 1
        assert!(
            output.contains("getelementptr inbounds %Bird, ptr %t3, i32 0, i32 1"),
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
        let (output, code) = dump_llvm("Mod/Loops.cp");
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
        let (output, code) = dump_llvm("Mod/Methods.cp");
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
}
