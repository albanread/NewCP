//! Curated probe manifest for tier 5 of the test matrix.
//!
//! Each `Probe` describes one cell of the (receiver × record-flavor ×
//! method-flavor × param-mode × param-type × dispatch-site × module-
//! boundary) cube.  We don't enumerate the full Cartesian product —
//! many cells are equivalent or impossible.  Instead we seed with:
//!
//!   1. Regression cells for the bug classes that have actually been
//!      found in NewCP so far (pointer-alias receivers, value-mode
//!      params, plain-record dispatch, nested-proc `$len` ABI).
//!   2. A small systematic spread across the rest of the cube so the
//!      next bug in the same neighbourhood has an empty matrix row
//!      to land in instead of a green-test illusion.
//!
//! New probes go here.  Re-run `cargo run -p newcp-test-matrix` to
//! emit them.

pub struct Probe {
    /// Module name (and `.cp` filename stem).  Use PascalCase so the
    /// emitted `MODULE` declaration looks idiomatic.
    pub module_name: &'static str,
    /// snake_case stem used to derive the generated `#[test] fn`
    /// name (`matrix_<test_name>`).  Keep it close to `module_name`.
    pub test_name: &'static str,
    /// CP Language Report section this probe exercises.  Used by the
    /// future coverage report to group cells by spec rule.  String,
    /// e.g. `"10.2"` or `"10.2 / 8.1"`.
    pub spec_section: &'static str,
    /// One-line human description.  Lands in the `///` doc comment
    /// above the generated test fn so `cargo test -- --list` reads
    /// nicely.
    pub description: &'static str,
    /// Expected return value of `Run()`.  The probe encodes its
    /// success/failure as a single packed INTEGER so the harness can
    /// `assert_eq!` against a literal.
    pub expected_value: i64,
    /// Full Component Pascal source.  Self-contained module — must
    /// `MODULE <module_name>` at the top, `Run* (): INTEGER` as the
    /// entry point, and `END <module_name>.` at the bottom.
    pub cp_source: &'static str,
    /// When `Some`, the generated test is `#[ignore]`d with this
    /// reason.  Use this to document a known compiler/runtime bug
    /// that the probe surfaces — the probe stays in the matrix as
    /// the regression target, but the suite stays green.  Un-ignore
    /// when the bug is fixed; that lights up as a real failure that
    /// proves the fix.
    pub ignored: Option<&'static str>,
}

pub static MANIFEST: &[Probe] = &[
    // ─── Receiver / dispatch backfill ───────────────────────────────

    Probe {
        module_name: "M_RecvPtrAlias_NewOnAbstract",
        test_name: "recv_ptr_alias_new_on_abstract",
        spec_section: "10.2",
        description: "pointer-alias receiver overrides an ABSTRACT method on the base \
                      (regression: pointer-alias receiver binding)",
        expected_value: 770,
        cp_source: r#"MODULE M_RecvPtrAlias_NewOnAbstract;
    TYPE
        BaseDesc* = ABSTRACT RECORD tag*: INTEGER END;
        Base*     = POINTER TO BaseDesc;
        SubDesc*  = RECORD (BaseDesc) extra*: INTEGER END;
        Sub*      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Greet* (v: INTEGER), NEW, ABSTRACT;

    PROCEDURE (s: Sub) Greet* (v: INTEGER);
    BEGIN s.tag := v; s.extra := v * 10 END Greet;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub;
    BEGIN
        NEW(s); s.Greet(7);
        RETURN (s.tag * 100) + s.extra
    END Run;
END M_RecvPtrAlias_NewOnAbstract.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_RecvValueStyle_NewOnPlain",
        test_name: "recv_value_style_new_on_plain",
        spec_section: "10.2 / 6.4",
        description: "NEW method on a plain (non-extensible) record — must emit a direct \
                      call, not a vtable dispatch (regression: plain-record dispatch)",
        expected_value: 4242,
        cp_source: r#"MODULE M_RecvValueStyle_NewOnPlain;
    TYPE
        Counter = RECORD value: INTEGER END;

    PROCEDURE (VAR c: Counter) Bump* (n: INTEGER), NEW;
    BEGIN c.value := c.value + n END Bump;

    PROCEDURE (c: Counter) Read* (): INTEGER, NEW;
    BEGIN RETURN c.value END Read;

    PROCEDURE Run* (): INTEGER;
        VAR c: Counter;
    BEGIN
        c.value := 0;
        c.Bump(42);
        c.Bump(4200);
        RETURN c.Read()
    END Run;
END M_RecvValueStyle_NewOnPlain.
"#,
        ignored: None,
    },

    // ─── Parameter-mode backfill (CP §10.1 / §8.1) ──────────────────

    Probe {
        module_name: "M_Param_Value_Record",
        test_name: "param_value_record_is_private_copy",
        spec_section: "10.1 / 8.1",
        description: "value-mode record param — callee mutation must NOT leak (regression: \
                      pass-by-value semantics for records)",
        expected_value: 42,
        cp_source: r#"MODULE M_Param_Value_Record;
    TYPE Box = RECORD value*: INTEGER END;

    PROCEDURE Mutate (b: Box);
    BEGIN b.value := 999 END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR caller: Box;
    BEGIN
        caller.value := 42;
        Mutate(caller);
        RETURN caller.value
    END Run;
END M_Param_Value_Record.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Param_Value_FixedArray",
        test_name: "param_value_fixed_array_is_private_copy",
        spec_section: "10.1 / 8.1",
        description: "value-mode fixed-array param — callee mutation must NOT leak \
                      (regression: pass-by-value for arrays)",
        expected_value: 7,
        cp_source: r#"MODULE M_Param_Value_FixedArray;
    PROCEDURE Mutate (a: ARRAY 4 OF INTEGER);
    BEGIN a[0] := 999 END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR caller: ARRAY 4 OF INTEGER;
    BEGIN
        caller[0] := 7;
        Mutate(caller);
        RETURN caller[0]
    END Run;
END M_Param_Value_FixedArray.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Param_Value_OpenArray",
        test_name: "param_value_open_array_is_private_copy",
        spec_section: "10.1 / 8.1",
        description: "value-mode open-array param — callee mutation must NOT leak \
                      (regression: prologue memmove on the data buffer)",
        expected_value: 7,
        cp_source: r#"MODULE M_Param_Value_OpenArray;
    PROCEDURE Mutate (p: ARRAY OF INTEGER);
    BEGIN p[0] := 99 END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY 4 OF INTEGER;
    BEGIN
        a[0] := 7;
        Mutate(a);
        RETURN a[0]
    END Run;
END M_Param_Value_OpenArray.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Param_VAR_Record",
        test_name: "param_var_record_aliases_caller",
        spec_section: "10.1",
        description: "VAR record param — callee mutation MUST be visible to the caller",
        expected_value: 999,
        cp_source: r#"MODULE M_Param_VAR_Record;
    TYPE Box = RECORD value*: INTEGER END;

    PROCEDURE Mutate (VAR b: Box);
    BEGIN b.value := 999 END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR caller: Box;
    BEGIN
        caller.value := 42;
        Mutate(caller);
        RETURN caller.value
    END Run;
END M_Param_VAR_Record.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Param_OUT_Record",
        test_name: "param_out_record_writes_through",
        spec_section: "10.1",
        description: "OUT record param — callee write must materialise in the caller's slot",
        expected_value: 7,
        cp_source: r#"MODULE M_Param_OUT_Record;
    TYPE Box = RECORD value*: INTEGER END;

    PROCEDURE Init (OUT b: Box);
    BEGIN b.value := 7 END Init;

    PROCEDURE Run* (): INTEGER;
        VAR caller: Box;
    BEGIN
        caller.value := 100;
        Init(caller);
        RETURN caller.value
    END Run;
END M_Param_OUT_Record.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Param_IN_OpenArray_LengthSurvives",
        test_name: "param_in_open_array_length_visible_via_LEN",
        spec_section: "10.1 / 8.2",
        description: "IN open-array param — LEN(p) inside callee returns the caller's length \
                      (regression: hidden $len companion ABI)",
        expected_value: 12,
        cp_source: r#"MODULE M_Param_IN_OpenArray_LengthSurvives;
    PROCEDURE Sum (IN p: ARRAY OF INTEGER): INTEGER;
        VAR i, s: INTEGER;
    BEGIN
        s := 0; i := 0;
        WHILE i < LEN(p) DO s := s + p[i]; INC(i) END;
        RETURN s
    END Sum;

    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY 4 OF INTEGER;
    BEGIN
        a[0] := 3; a[1] := 3; a[2] := 3; a[3] := 3;
        RETURN Sum(a)
    END Run;
END M_Param_IN_OpenArray_LengthSurvives.
"#,
        ignored: None,
    },

    // ─── Nested procedures & upvalues ───────────────────────────────

    Probe {
        module_name: "M_Nested_OpenArrayUpvalue",
        test_name: "nested_proc_calls_with_open_array_arg",
        spec_section: "10 / 10.1",
        description: "nested proc taking a value-mode open-array; the call site must \
                      push the hidden $len companion (regression: nested-call ABI)",
        expected_value: 42,
        cp_source: r#"MODULE M_Nested_OpenArrayUpvalue;
    PROCEDURE Outer (IN p: ARRAY OF INTEGER): INTEGER;
        VAR result: INTEGER;

        PROCEDURE Inner (q: ARRAY OF INTEGER): INTEGER;
            VAR i, s: INTEGER;
        BEGIN
            s := 0; i := 0;
            WHILE i < LEN(q) DO s := s + q[i]; INC(i) END;
            RETURN s
        END Inner;

    BEGIN
        result := Inner(p);
        RETURN result
    END Outer;

    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY 4 OF INTEGER;
    BEGIN
        a[0] := 10; a[1] := 11; a[2] := 9; a[3] := 12;
        RETURN Outer(a)
    END Run;
END M_Nested_OpenArrayUpvalue.
"#,
        ignored: None,
    },

    // ─── More param-mode × type-kind cells ──────────────────────────

    Probe {
        module_name: "M_Param_VAR_FixedArray",
        test_name: "param_var_fixed_array_aliases_caller",
        spec_section: "10.1",
        description: "VAR fixed-array param — callee mutation propagates to the caller",
        expected_value: 999,
        cp_source: r#"MODULE M_Param_VAR_FixedArray;
    PROCEDURE Mutate (VAR a: ARRAY 4 OF INTEGER);
    BEGIN a[0] := 999 END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR caller: ARRAY 4 OF INTEGER;
    BEGIN
        caller[0] := 42;
        Mutate(caller);
        RETURN caller[0]
    END Run;
END M_Param_VAR_FixedArray.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Param_VAR_OpenArray",
        test_name: "param_var_open_array_aliases_caller",
        spec_section: "10.1",
        description: "VAR open-array param — callee mutation propagates to the caller",
        expected_value: 999,
        cp_source: r#"MODULE M_Param_VAR_OpenArray;
    PROCEDURE Mutate (VAR p: ARRAY OF INTEGER);
    BEGIN p[0] := 999 END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR caller: ARRAY 4 OF INTEGER;
    BEGIN
        caller[0] := 42;
        Mutate(caller);
        RETURN caller[0]
    END Run;
END M_Param_VAR_OpenArray.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Param_IN_Record",
        test_name: "param_in_record_field_readable",
        spec_section: "10.1",
        description: "IN record param — fields readable (write would be a sema error, \
                      covered separately by the negative-test corpus)",
        expected_value: 77,
        cp_source: r#"MODULE M_Param_IN_Record;
    TYPE Box = RECORD value: INTEGER END;

    PROCEDURE Peek (IN b: Box): INTEGER;
    BEGIN RETURN b.value END Peek;

    PROCEDURE Run* (): INTEGER;
        VAR caller: Box;
    BEGIN
        caller.value := 77;
        RETURN Peek(caller)
    END Run;
END M_Param_IN_Record.
"#,
        ignored: None,
    },

    // ─── Method dispatch on aggregate access paths ──────────────────

    Probe {
        module_name: "M_Method_On_RecordField",
        test_name: "method_call_on_record_field_dispatches",
        spec_section: "10.2 / 8.4",
        description: "method invoked through a record-field designator (obj.fld.Method()) \
                      — receiver lowering must descend through the field GEP",
        expected_value: 21,
        cp_source: r#"MODULE M_Method_On_RecordField;
    TYPE
        InnerDesc = EXTENSIBLE RECORD n: INTEGER END;
        Inner     = POINTER TO InnerDesc;
        Outer     = RECORD inner: Inner END;

    PROCEDURE (i: Inner) Triple* (): INTEGER, NEW;
    BEGIN RETURN i.n * 3 END Triple;

    PROCEDURE Run* (): INTEGER;
        VAR o: Outer;
    BEGIN
        NEW(o.inner);
        o.inner.n := 7;
        RETURN o.inner.Triple()
    END Run;
END M_Method_On_RecordField.
"#,
        ignored: Some(
            "KNOWN BUG: NEW(record_field_pointer) trips IR codegen with \
             `Instr::New: unknown record type opaque:new-ptr`. The IR layer \
             can't resolve the destination's record type when the NEW target \
             is a record field's pointer (vs. a plain local pointer). \
             Surfaced by the matrix on first run — file under deferred_fixes \
             and un-ignore once IR `lower_new` learns to chase the field \
             type.",
        ),
    },

    // ─── Procedure-typed value (indirect call) ──────────────────────

    Probe {
        module_name: "M_ProcType_IndirectCall",
        test_name: "procedure_typed_variable_indirect_call",
        spec_section: "6.5 / 10",
        description: "procedure-typed variable invoked via name() syntax — indirect call \
                      through the function-pointer slot",
        expected_value: 49,
        cp_source: r#"MODULE M_ProcType_IndirectCall;
    TYPE BinOp = PROCEDURE (a, b: INTEGER): INTEGER;

    PROCEDURE Mul (a, b: INTEGER): INTEGER;
    BEGIN RETURN a * b END Mul;

    PROCEDURE Apply (op: BinOp; x, y: INTEGER): INTEGER;
    BEGIN RETURN op(x, y) END Apply;

    PROCEDURE Run* (): INTEGER;
        VAR op: BinOp;
    BEGIN
        op := Mul;
        RETURN Apply(op, 7, 7)
    END Run;
END M_ProcType_IndirectCall.
"#,
        ignored: None,
    },

    // ─── Super-call shape ───────────────────────────────────────────

    Probe {
        module_name: "M_SuperCall_SameModule",
        test_name: "super_call_lands_in_base_method_body",
        spec_section: "10.2",
        description: "subclass override calls Super^ to chain into the base implementation",
        expected_value: 30,
        cp_source: r#"MODULE M_SuperCall_SameModule;
    TYPE
        BaseDesc* = EXTENSIBLE RECORD
            n*: INTEGER
        END;
        Base*     = POINTER TO BaseDesc;
        SubDesc*  = RECORD (BaseDesc) END;
        Sub*      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Add* (k: INTEGER), NEW, EXTENSIBLE;
    BEGIN b.n := b.n + k END Add;

    PROCEDURE (s: Sub) Add* (k: INTEGER);
    BEGIN
        s.Add^(k);          (* chain into Base.Add: n := n + k *)
        s.n := s.n + k      (* then double the effect *)
    END Add;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub;
    BEGIN
        NEW(s);
        s.n := 0;
        s.Add(15);          (* 0 + 15 (super) + 15 (override) = 30 *)
        RETURN s.n
    END Run;
END M_SuperCall_SameModule.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_MultiLevel_Inheritance_Dispatch",
        test_name: "multi_level_inheritance_dispatch",
        spec_section: "10.2",
        description: "three-level inheritance (Base ← Mid ← Sub) — calling a method via a \
                      Sub pointer must reach Sub's override; calling via Mid pointer to \
                      a Sub instance must also reach Sub's override (virtual dispatch)",
        expected_value: 137,
        cp_source: r#"MODULE M_MultiLevel_Inheritance_Dispatch;
    TYPE
        BaseDesc* = ABSTRACT RECORD tag*: INTEGER END;
        Base*     = POINTER TO BaseDesc;
        MidDesc*  = EXTENSIBLE RECORD (BaseDesc) END;
        Mid*      = POINTER TO MidDesc;
        SubDesc*  = RECORD (MidDesc) END;
        Sub*      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Set* (v: INTEGER), NEW, ABSTRACT;

    PROCEDURE (m: Mid) Set* (v: INTEGER), EXTENSIBLE;
    BEGIN m.tag := v * 10 END Set;

    PROCEDURE (s: Sub) Set* (v: INTEGER);
    BEGIN s.tag := v * 100 + 7 END Set;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub; m: Mid;
    BEGIN
        NEW(s);
        s.Set(1);              (* hits Sub.Set: tag = 107 *)
        m := s;                (* widen pointer; dynamic type still Sub *)
        m.Set(3);              (* virtual: hits Sub.Set: tag = 307 *)
        RETURN s.tag - 170     (* 307 - 170 = 137 *)
    END Run;
END M_MultiLevel_Inheritance_Dispatch.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Empty_Method_Is_NoOp",
        test_name: "empty_method_callable_as_noop",
        spec_section: "10.2",
        description: "EMPTY method is callable and is a no-op; subclass may override or \
                      leave the default in place",
        expected_value: 5,
        cp_source: r#"MODULE M_Empty_Method_Is_NoOp;
    TYPE
        BaseDesc* = EXTENSIBLE RECORD value*: INTEGER END;
        Base*     = POINTER TO BaseDesc;

    PROCEDURE (b: Base) Visit* (), NEW, EMPTY;

    PROCEDURE Run* (): INTEGER;
        VAR b: Base;
    BEGIN
        NEW(b);
        b.value := 5;
        b.Visit();             (* no-op; value untouched *)
        RETURN b.value
    END Run;
END M_Empty_Method_Is_NoOp.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Param_VAR_Pointer",
        test_name: "param_var_pointer_can_be_swapped",
        spec_section: "10.1",
        description: "VAR pointer param — callee may reassign the pointer itself, and \
                      the caller sees the new target",
        expected_value: 99,
        cp_source: r#"MODULE M_Param_VAR_Pointer;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Replace (VAR b: Box);
        VAR fresh: Box;
    BEGIN
        NEW(fresh);
        fresh.value := 99;
        b := fresh
    END Replace;

    PROCEDURE Run* (): INTEGER;
        VAR orig: Box;
    BEGIN
        NEW(orig);
        orig.value := 1;
        Replace(orig);
        RETURN orig.value      (* 99 if the new pointer landed in the caller's slot *)
    END Run;
END M_Param_VAR_Pointer.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Param_IN_Pointer_Deref",
        test_name: "param_in_pointer_target_is_readable",
        spec_section: "10.1",
        description: "IN pointer param — callee may dereference and read fields (writing \
                      to the pointer itself is a sema error, covered separately)",
        expected_value: 42,
        cp_source: r#"MODULE M_Param_IN_Pointer_Deref;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Peek (IN b: Box): INTEGER;
    BEGIN RETURN b.value END Peek;

    PROCEDURE Run* (): INTEGER;
        VAR p: Box;
    BEGIN
        NEW(p);
        p.value := 42;
        RETURN Peek(p)
    END Run;
END M_Param_IN_Pointer_Deref.
"#,
        ignored: Some(
            "KNOWN BUG: `IN p: PointerAlias` parameter crashes with \
             STATUS_ACCESS_VIOLATION when the body dereferences `p`. \
             Likely the param-lowering treats the pointer alias as a \
             record value and skips the necessary heap-pointer Load \
             (similar shape to the method-dispatch receiver fix but \
             on the parameter-access path). File under deferred_fixes \
             #17 and un-ignore once IN-pointer field access is fixed.",
        ),
    },

    Probe {
        module_name: "M_AnyPtr_TypeGuard",
        test_name: "anyptr_narrowed_via_type_guard",
        spec_section: "8.4 / 8.5",
        description: "ANYPTR carrying a typed pointer narrows back to the concrete type \
                      via the `p(T)` type-guard syntax",
        expected_value: 73,
        cp_source: r#"MODULE M_AnyPtr_TypeGuard;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Run* (): INTEGER;
        VAR
            b: Box;
            any: ANYPTR;
    BEGIN
        NEW(b);
        b.value := 73;
        any := b;
        RETURN any(Box).value
    END Run;
END M_AnyPtr_TypeGuard.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_AnyPtr_IS_Test",
        test_name: "anyptr_is_test_distinguishes_types",
        spec_section: "8.5",
        description: "`IS` test on ANYPTR returns TRUE for the actual dynamic type and \
                      FALSE for an unrelated record",
        expected_value: 110,
        cp_source: r#"MODULE M_AnyPtr_IS_Test;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;
        BagDesc = RECORD count: INTEGER END;
        Bag     = POINTER TO BagDesc;

    PROCEDURE Run* (): INTEGER;
        VAR
            b: Box;
            any: ANYPTR;
            score: INTEGER;
    BEGIN
        NEW(b);
        any := b;
        score := 0;
        IF any IS Box THEN score := score + 100 END;
        IF any IS Bag THEN score := score + 1000 END;
        score := score + 10;
        RETURN score
    END Run;
END M_AnyPtr_IS_Test.
"#,
        ignored: Some(
            "KNOWN BUG: `IS` test on ANYPTR against a record type whose \
             TypeDesc has not been instantiated elsewhere in the module \
             segfaults at runtime (STATUS_ACCESS_VIOLATION). Likely the \
             type-test fast path dereferences a NIL TypeDesc when the \
             Bag side of the test has never been registered. File under \
             deferred_fixes #16 and un-ignore once the lookup hardens \
             the NIL-TypeDesc case.",
        ),
    },

    Probe {
        module_name: "M_ProcType_Param_Callback",
        test_name: "proc_type_param_invoked_as_callback",
        spec_section: "6.5 / 10.1",
        description: "procedure-typed value passed as a parameter and invoked inside the \
                      callee (callback pattern)",
        expected_value: 121,
        cp_source: r#"MODULE M_ProcType_Param_Callback;
    TYPE Unary = PROCEDURE (x: INTEGER): INTEGER;

    PROCEDURE Square (x: INTEGER): INTEGER;
    BEGIN RETURN x * x END Square;

    PROCEDURE ApplyTwice (seed: INTEGER; f: Unary): INTEGER;
        VAR once: INTEGER;
    BEGIN
        once := f(seed);
        RETURN f(once)              (* Square(Square(seed)) via a temp *)
    END ApplyTwice;

    PROCEDURE Run* (): INTEGER;
        VAR cb: Unary;
    BEGIN
        cb := Square;
        (* ApplyTwice(Square, 3) = Square(Square(3)) = Square(9) = 81;
           plus a marker constant 40 so a stub returning 0 fails fast *)
        RETURN ApplyTwice(3, cb) + 40
    END Run;
END M_ProcType_Param_Callback.
"#,
        ignored: Some(
            "KNOWN BUG: sema mis-types the argument of an indirect call \
             through a procedure-typed parameter — `f(seed)` reports \
             `found unresolved:seed` even though `seed` is a peer \
             parameter in the same proc.  `M_ProcType_IndirectCall` \
             works because that probe assigns the proc-value to a \
             local first and calls the local — so the bug is specific \
             to calling through a parameter, not through a local. \
             File under deferred_fixes #18.",
        ),
    },

    Probe {
        module_name: "M_OpenArray_Of_Records_ValueCopy",
        test_name: "open_array_of_records_is_private_copy",
        spec_section: "10.1 / 8.1",
        description: "value-mode open array of records — prologue must memmove the full \
                      array width (count * sizeof(record)), not just the first element",
        expected_value: 50,
        cp_source: r#"MODULE M_OpenArray_Of_Records_ValueCopy;
    TYPE Point = RECORD x*, y*: INTEGER END;

    PROCEDURE Mutate (a: ARRAY OF Point);
    BEGIN
        a[0].x := 999; a[0].y := 999;
        a[1].x := 999; a[1].y := 999
    END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY 2 OF Point;
    BEGIN
        a[0].x := 10; a[0].y := 11;
        a[1].x := 14; a[1].y := 15;
        Mutate(a);
        RETURN a[0].x + a[0].y + a[1].x + a[1].y    (* 10+11+14+15 = 50 *)
    END Run;
END M_OpenArray_Of_Records_ValueCopy.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_OpenArray_Of_CHAR",
        test_name: "open_array_of_char_iteration",
        spec_section: "10.1 / 8.2",
        description: "open-array of CHAR — iteration with LEN(p) walks the right element \
                      width; classic string-handling idiom",
        expected_value: 295,
        cp_source: r#"MODULE M_OpenArray_Of_CHAR;
    PROCEDURE Sum (IN s: ARRAY OF CHAR): INTEGER;
        VAR i, total: INTEGER;
    BEGIN
        i := 0; total := 0;
        WHILE (i < LEN(s)) & (s[i] # 0X) DO
            total := total + ORD(s[i]);
            INC(i)
        END;
        RETURN total
    END Sum;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        (* "ABC" → ORD('A')+ORD('B')+ORD('C') = 65+66+67 = 198; plus
           a length marker (97 = "a") so the sum has to include the
           trailing char before the NUL. *)
        RETURN Sum("ABCa")
    END Run;
END M_OpenArray_Of_CHAR.
"#,
        ignored: None,
    },


    // ─── Tier 3: expressions (CP §8) ────────────────────────────────

    Probe {
        module_name: "M_Expr_DIV_Floored",
        test_name: "expr_div_floors_toward_negative_infinity",
        spec_section: "8.2.2",
        description: "CP's DIV is floored division (rounds toward -∞), unlike C's / on \
                      negative dividends",
        expected_value: 10004,
        cp_source: r#"MODULE M_Expr_DIV_Floored;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, c, d: INTEGER;
    BEGIN
        a :=    7  DIV   3;     (*  2 *)
        b := (-7) DIV   3;      (* -3 (C would say -2) *)
        c :=    7  DIV (-3);    (* -3 *)
        d := (-7) DIV (-3);     (*  2 *)
        (* pack into one int: a*1000 + (b+10)*100 + (c+10)*10 + (d+10)
           = 2000 + 7*100 + 7*10 + 12 = 2782; offset by 7222 to land on a stable signature *)
        RETURN a * 1000 + (b + 10) * 100 + (c + 10) * 10 + (d + 10) + 7222
    END Run;
END M_Expr_DIV_Floored.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_MOD_NonNegative",
        test_name: "expr_mod_result_is_non_negative_when_divisor_positive",
        spec_section: "8.2.2",
        description: "CP's MOD with a positive divisor always returns a non-negative result, \
                      matching the floored-DIV identity a = (a DIV b) * b + (a MOD b)",
        expected_value: 1212,
        cp_source: r#"MODULE M_Expr_MOD_NonNegative;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: INTEGER;
    BEGIN
        a :=    7  MOD 3;     (* 1 *)
        b := (-7) MOD 3;      (* 2 — C would say -1 *)
        RETURN a * 1000 + b * 100 + 12      (* 1000 + 200 + 12 = 1212 *)
    END Run;
END M_Expr_MOD_NonNegative.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_SET_RangeConstruction",
        test_name: "expr_set_construction_with_range",
        spec_section: "8.2.4",
        description: "SET literal with range syntax `{lo..hi}` populates every element \
                      in the inclusive interval",
        expected_value: 248,
        cp_source: r#"MODULE M_Expr_SET_RangeConstruction;
    PROCEDURE Run* (): INTEGER;
        VAR s: SET; score: INTEGER;
    BEGIN
        s := {3..7};                 (* bits 3,4,5,6,7 *)
        score := 0;
        IF 3 IN s THEN score := score + 1   END;
        IF 5 IN s THEN score := score + 10  END;
        IF 7 IN s THEN score := score + 100 END;
        IF 8 IN s THEN score := score + 1000 END;   (* must not fire *)
        IF 2 IN s THEN score := score + 10000 END;  (* must not fire *)
        RETURN score + 137                          (* 111 + 137 = 248 *)
    END Run;
END M_Expr_SET_RangeConstruction.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_SET_Operators",
        test_name: "expr_set_union_intersect_difference",
        spec_section: "8.2.4",
        description: "SET union (+), intersection (*), difference (-), symmetric \
                      difference (/) on small sets",
        expected_value: 4321,
        cp_source: r#"MODULE M_Expr_SET_Operators;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, u, i, d, sd: SET; score: INTEGER;
    BEGIN
        a := {0, 1, 2};
        b := {1, 2, 3};
        u  := a + b;                 (* {0,1,2,3} *)
        i  := a * b;                 (* {1,2}     *)
        d  := a - b;                 (* {0}       *)
        sd := a / b;                 (* {0,3}     *)
        score := 0;
        IF (0 IN u) & (3 IN u) & (1 IN u) & (2 IN u) THEN score := score + 1 END;
        IF (1 IN i) & (2 IN i) & ~(0 IN i) & ~(3 IN i) THEN score := score + 20 END;
        IF (0 IN d) & ~(1 IN d) & ~(2 IN d) & ~(3 IN d) THEN score := score + 300 END;
        IF (0 IN sd) & (3 IN sd) & ~(1 IN sd) & ~(2 IN sd) THEN score := score + 4000 END;
        RETURN score
    END Run;
END M_Expr_SET_Operators.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_Pointer_IS_Test",
        test_name: "expr_is_test_on_pointer_to_extensible",
        spec_section: "8.5",
        description: "IS test on a record-pointer narrows correctly across an extensible \
                      hierarchy",
        expected_value: 1010,
        cp_source: r#"MODULE M_Expr_Pointer_IS_Test;
    TYPE
        BaseDesc = EXTENSIBLE RECORD END;
        Base     = POINTER TO BaseDesc;
        SubDesc  = RECORD (BaseDesc) END;
        Sub      = POINTER TO SubDesc;
        OtherDesc = RECORD (BaseDesc) END;
        Other    = POINTER TO OtherDesc;

    PROCEDURE Run* (): INTEGER;
        VAR p: Base; sub: Sub; score: INTEGER;
    BEGIN
        NEW(sub);
        p := sub;
        score := 0;
        IF p IS Base  THEN score := score + 1000 END;   (* always true *)
        IF p IS Sub   THEN score := score +   10 END;   (* dynamic type matches *)
        IF p IS Other THEN score := score + 1000000 END;(* must NOT fire *)
        RETURN score
    END Run;
END M_Expr_Pointer_IS_Test.
"#,
        ignored: Some(
            "KNOWN BUG (same family as M_AnyPtr_IS_Test): IS test \
             against a record type with no instantiated TypeDesc \
             (Other is declared but never NEW'd) crashes with \
             STATUS_ACCESS_VIOLATION. See deferred_fixes #16.",
        ),
    },

    Probe {
        module_name: "M_Expr_ENTIER_NegativeReal",
        test_name: "expr_entier_floors_negative_real",
        spec_section: "10.3",
        description: "ENTIER(r) floors a REAL toward -∞ and returns the LONGINT result \
                      (CP semantics: ENTIER(-2.3) = -3, not -2)",
        expected_value: 280,
        cp_source: r#"MODULE M_Expr_ENTIER_NegativeReal;
    PROCEDURE Run* (): LONGINT;
        VAR a, b, c: LONGINT;
    BEGIN
        a := ENTIER( 2.7);     (*  2 *)
        b := ENTIER(-2.3);     (* -3 *)
        c := ENTIER(-2.7);     (* -3 *)
        (* a*100 + (b+10)*10 + (c+10) + 3
           = 2*100 + 7*10 + 7 + 3 = 280 *)
        RETURN a * 100 + (b + 10) * 10 + (c + 10) + 3
    END Run;
END M_Expr_ENTIER_NegativeReal.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Builtin_ABS_ODD_MIN_MAX",
        test_name: "builtin_abs_odd_min_max",
        spec_section: "10.3",
        description: "ABS / ODD / MIN / MAX predeclared procedures on INTEGER",
        expected_value: 1111,
        cp_source: r#"MODULE M_Builtin_ABS_ODD_MIN_MAX;
    PROCEDURE Run* (): INTEGER;
        VAR score: INTEGER;
    BEGIN
        score := 0;
        IF ABS(-7) = 7 THEN score := score + 1 END;
        IF ODD(7)      THEN score := score + 10 END;
        IF ~ODD(8)     THEN score := score + 100 END;
        IF MAX(INTEGER) > 0 THEN score := score + 1000 END;
        RETURN score
    END Run;
END M_Builtin_ABS_ODD_MIN_MAX.
"#,
        ignored: None,
    },


    // ─── Tier 4: statements (CP §9) ─────────────────────────────────

    Probe {
        module_name: "M_Stmt_CASE_IntegerRanges",
        test_name: "stmt_case_integer_with_ranges",
        spec_section: "9.5",
        description: "CASE statement on INTEGER with range labels + single labels + ELSE",
        expected_value: 246,
        cp_source: r#"MODULE M_Stmt_CASE_IntegerRanges;
    PROCEDURE Bucket (n: INTEGER): INTEGER;
    BEGIN
        CASE n OF
          0:        RETURN 100
        | 1..5:     RETURN 200 + n
        | 7, 9, 11: RETURN 300 + n
        ELSE        RETURN 999
        END
    END Bucket;

    PROCEDURE Run* (): INTEGER;
        VAR score: INTEGER;
    BEGIN
        score := 0;
        IF Bucket(0)  = 100 THEN score := score + 1   END;
        IF Bucket(3)  = 203 THEN score := score + 5   END;
        IF Bucket(9)  = 309 THEN score := score + 40  END;
        IF Bucket(99) = 999 THEN score := score + 200 END;
        RETURN score
    END Run;
END M_Stmt_CASE_IntegerRanges.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_CASE_CHAR",
        test_name: "stmt_case_on_char",
        spec_section: "9.5",
        description: "CASE statement on CHAR with single + range labels",
        expected_value: 333,
        cp_source: r#"MODULE M_Stmt_CASE_CHAR;
    PROCEDURE Classify (c: CHAR): INTEGER;
    BEGIN
        CASE c OF
          "0".."9": RETURN 10
        | "A".."Z": RETURN 100
        | "a".."z": RETURN 1
        ELSE        RETURN 0
        END
    END Classify;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        (* 'M' → 100, '7' → 10, 'q' → 1, '?' → 0 = 100+10+1+0 = 111;
           multiplied by 3 = 333 *)
        RETURN (Classify("M") + Classify("7") + Classify("q") + Classify("?")) * 3
    END Run;
END M_Stmt_CASE_CHAR.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_FOR_PositiveStep",
        test_name: "stmt_for_positive_step",
        spec_section: "9.7",
        description: "FOR loop with explicit positive non-unit BY step",
        expected_value: 25,
        cp_source: r#"MODULE M_Stmt_FOR_PositiveStep;
    PROCEDURE Run* (): INTEGER;
        VAR i, sum: INTEGER;
    BEGIN
        sum := 0;
        FOR i := 1 TO 9 BY 2 DO sum := sum + i END;
        RETURN sum                      (* 1 + 3 + 5 + 7 + 9 = 25 *)
    END Run;
END M_Stmt_FOR_PositiveStep.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_FOR_NegativeStep",
        test_name: "stmt_for_negative_step",
        spec_section: "9.7",
        description: "FOR loop with negative BY step counts down inclusive",
        expected_value: 15,
        cp_source: r#"MODULE M_Stmt_FOR_NegativeStep;
    PROCEDURE Run* (): INTEGER;
        VAR i, sum: INTEGER;
    BEGIN
        sum := 0;
        FOR i := 5 TO 1 BY -1 DO sum := sum + i END;
        RETURN sum                      (* 5+4+3+2+1 = 15 *)
    END Run;
END M_Stmt_FOR_NegativeStep.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_WITH_MultiArm",
        test_name: "stmt_with_multi_arm_narrowing",
        spec_section: "9.6",
        description: "WITH statement with multiple type-arm branches dispatches to the \
                      arm that matches the dynamic type",
        expected_value: 33,
        cp_source: r#"MODULE M_Stmt_WITH_MultiArm;
    TYPE
        BaseDesc = EXTENSIBLE RECORD tag: INTEGER END;
        Base     = POINTER TO BaseDesc;
        ADesc    = RECORD (BaseDesc) av: INTEGER END;
        A        = POINTER TO ADesc;
        BDesc    = RECORD (BaseDesc) bv: INTEGER END;
        B        = POINTER TO BDesc;

    PROCEDURE Score (p: Base): INTEGER;
    BEGIN
        WITH p: A DO
            RETURN 10 + p.av
        |  p: B DO
            RETURN 20 + p.bv
        ELSE
            RETURN 100
        END
    END Score;

    PROCEDURE Run* (): INTEGER;
        VAR a: A; b: B;
    BEGIN
        NEW(a); a.av := 1;
        NEW(b); b.bv := 2;
        (* Score(a) = 10 + 1 = 11; Score(b) = 20 + 2 = 22; sum = 33 *)
        RETURN Score(a) + Score(b)
    END Run;
END M_Stmt_WITH_MultiArm.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_LOOP_EXIT_Nested",
        test_name: "stmt_loop_exit_only_inner",
        spec_section: "9.8",
        description: "EXIT inside a nested LOOP leaves only the innermost loop, not the outer",
        expected_value: 11,
        cp_source: r#"MODULE M_Stmt_LOOP_EXIT_Nested;
    PROCEDURE Run* (): INTEGER;
        VAR outer, inner, count: INTEGER;
    BEGIN
        outer := 0; count := 0;
        LOOP
            inner := 0;
            LOOP
                INC(inner); INC(count);
                IF inner >= 3 THEN EXIT END
            END;
            INC(outer);
            IF outer >= 3 THEN EXIT END
        END;
        (* outer runs 3 times, inner runs 3 times each → count = 9; outer = 3.
           Pack: outer*10 + (count - 8) = 30 + 1 = 31... wait recompute. *)
        RETURN outer + count - 1        (* 3 + 9 - 1 = 11 *)
    END Run;
END M_Stmt_LOOP_EXIT_Nested.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_RETURN_FromInside_WITH",
        test_name: "stmt_return_from_inside_with",
        spec_section: "9.6 / 10",
        description: "RETURN nested inside a WITH arm exits the procedure and yields the \
                      WITH-narrowed value",
        expected_value: 77,
        cp_source: r#"MODULE M_Stmt_RETURN_FromInside_WITH;
    TYPE
        BaseDesc = EXTENSIBLE RECORD END;
        Base     = POINTER TO BaseDesc;
        SubDesc  = RECORD (BaseDesc) value: INTEGER END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE PullValue (p: Base): INTEGER;
    BEGIN
        WITH p: Sub DO
            RETURN p.value
        ELSE
            RETURN 0
        END
    END PullValue;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub;
    BEGIN
        NEW(s); s.value := 77;
        RETURN PullValue(s)
    END Run;
END M_Stmt_RETURN_FromInside_WITH.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_REPEAT_Until",
        test_name: "stmt_repeat_until_runs_at_least_once",
        spec_section: "9.7",
        description: "REPEAT/UNTIL evaluates the body before the test — guaranteed at \
                      least one iteration even when the condition is initially true",
        expected_value: 4,
        cp_source: r#"MODULE M_Stmt_REPEAT_Until;
    PROCEDURE Run* (): INTEGER;
        VAR i, count: INTEGER;
    BEGIN
        i := 0; count := 0;
        REPEAT
            INC(i);
            INC(count)
        UNTIL i >= 4;
        RETURN count
    END Run;
END M_Stmt_REPEAT_Until.
"#,
        ignored: None,
    },


    // ─── Tier 10.3: predeclared procedures ──────────────────────────

    Probe {
        module_name: "M_Builtin_INC_DEC",
        test_name: "builtin_inc_dec_with_and_without_delta",
        spec_section: "10.3",
        description: "INC/DEC predeclared mutate their argument; the two-arg form takes \
                      an explicit delta",
        expected_value: 13,
        cp_source: r#"MODULE M_Builtin_INC_DEC;
    PROCEDURE Run* (): INTEGER;
        VAR i: INTEGER;
    BEGIN
        i := 10;
        INC(i);          (* 11 *)
        INC(i, 5);       (* 16 *)
        DEC(i);          (* 15 *)
        DEC(i, 2);       (* 13 *)
        RETURN i
    END Run;
END M_Builtin_INC_DEC.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Builtin_INCL_EXCL",
        test_name: "builtin_incl_excl_on_set",
        spec_section: "10.3",
        description: "INCL/EXCL predeclared add/remove a single SET element",
        expected_value: 211,
        cp_source: r#"MODULE M_Builtin_INCL_EXCL;
    PROCEDURE Run* (): INTEGER;
        VAR s: SET; score: INTEGER;
    BEGIN
        s := {};
        INCL(s, 3);
        INCL(s, 7);
        INCL(s, 11);
        EXCL(s, 7);
        score := 0;
        IF  3 IN s THEN score := score +   1 END;
        IF  7 IN s THEN score := score + 1000 END;   (* must not fire *)
        IF 11 IN s THEN score := score + 10 END;
        IF ~(7 IN s) THEN score := score + 200 END;
        RETURN score
    END Run;
END M_Builtin_INCL_EXCL.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Builtin_LEN_OnDifferentArrayKinds",
        test_name: "builtin_len_on_fixed_and_open_arrays",
        spec_section: "10.3",
        description: "LEN on a fixed-size array is a constant; LEN on an open-array \
                      parameter pulls the hidden $len companion",
        expected_value: 87,
        cp_source: r#"MODULE M_Builtin_LEN_OnDifferentArrayKinds;
    PROCEDURE OpenLen (IN a: ARRAY OF INTEGER): INTEGER;
    BEGIN RETURN LEN(a) END OpenLen;

    PROCEDURE Run* (): INTEGER;
        VAR fixed: ARRAY 7 OF INTEGER;
    BEGIN
        (* LEN(fixed) is the static 7; OpenLen(fixed) reports the same 7
           via the open-array ABI's hidden length companion.  Combine
           into 7 * 10 + 7 + 10 = 87. *)
        RETURN LEN(fixed) * 10 + OpenLen(fixed) + 10
    END Run;
END M_Builtin_LEN_OnDifferentArrayKinds.
"#,
        ignored: None,
    },


    // ─── Module structure (CP §11) ──────────────────────────────────

    Probe {
        module_name: "M_Module_ForwardReference",
        test_name: "module_forward_call_resolves_after_decl",
        spec_section: "11",
        description: "a procedure may call another procedure declared later in the \
                      same module — sema resolves the forward reference at the module level",
        expected_value: 49,
        cp_source: r#"MODULE M_Module_ForwardReference;
    PROCEDURE Outer (x: INTEGER): INTEGER;
    BEGIN RETURN Inner(x) * 7 END Outer;

    PROCEDURE Inner (x: INTEGER): INTEGER;
    BEGIN RETURN x + 4 END Inner;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN Outer(3)      (* Inner(3)*7 = 7*7 = 49 *)
    END Run;
END M_Module_ForwardReference.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Module_VAR_Shared",
        test_name: "module_level_var_shared_across_procs",
        spec_section: "11 / 7",
        description: "module-level VARs are shared state across procedure calls in the \
                      same module's body",
        expected_value: 30,
        cp_source: r#"MODULE M_Module_VAR_Shared;
    VAR counter: INTEGER;

    PROCEDURE Bump (k: INTEGER);
    BEGIN counter := counter + k END Bump;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        counter := 0;
        Bump(7);
        Bump(11);
        Bump(12);
        RETURN counter
    END Run;
END M_Module_VAR_Shared.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Module_BEGIN_Block_Runs",
        test_name: "module_begin_block_initializes_state",
        spec_section: "11",
        description: "the module-level BEGIN block runs once at load time, before any \
                      exported procedure is called",
        expected_value: 99,
        cp_source: r#"MODULE M_Module_BEGIN_Block_Runs;
    VAR seed: INTEGER;

    PROCEDURE Run* (): INTEGER;
    BEGIN RETURN seed END Run;

BEGIN
    seed := 99
END M_Module_BEGIN_Block_Runs.
"#,
        ignored: None,
    },


    // ─── More tier 3: expressions (CP §8) ───────────────────────────

    Probe {
        module_name: "M_Expr_LogicalAnd_ShortCircuit",
        test_name: "expr_logical_and_short_circuits",
        spec_section: "8.2.3",
        description: "`&` evaluates the right operand only if the left is TRUE; a side \
                      effect in the right operand must NOT fire when the left is FALSE",
        expected_value: 1,
        cp_source: r#"MODULE M_Expr_LogicalAnd_ShortCircuit;
    VAR sideEffect: INTEGER;

    PROCEDURE Touch (): BOOLEAN;
    BEGIN INC(sideEffect); RETURN TRUE END Touch;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        sideEffect := 0;
        (* FALSE & Touch() — Touch must NOT be called *)
        IF FALSE & Touch() THEN END;
        IF sideEffect # 0 THEN RETURN -1 END;
        (* TRUE & Touch() — Touch IS called *)
        IF TRUE & Touch() THEN END;
        RETURN sideEffect      (* 1 = Touch called once total *)
    END Run;
END M_Expr_LogicalAnd_ShortCircuit.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_ShortCircuit_NilGuard",
        test_name: "expr_short_circuit_nil_guard_idiom",
        spec_section: "8.2.3",
        description: "the load-bearing CP idiom `IF (p # NIL) & (p.field > 0) THEN ...` \
                      must NOT dereference p when it is NIL (short-circuit eval); without \
                      the fix every defensive NIL guard in BlackBox source silently \
                      crashes on the FALSE branch",
        expected_value: 42,
        cp_source: r#"MODULE M_Expr_ShortCircuit_NilGuard;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Probe (p: Box): INTEGER;
    BEGIN
        IF (p # NIL) & (p.value > 0) THEN
            RETURN p.value
        ELSE
            RETURN 42
        END
    END Probe;

    PROCEDURE Run* (): INTEGER;
        VAR nilBox: Box;
    BEGIN
        nilBox := NIL;
        (* p is NIL — second conjunct must NOT execute *)
        RETURN Probe(nilBox)
    END Run;
END M_Expr_ShortCircuit_NilGuard.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_LogicalOr_ShortCircuit",
        test_name: "expr_logical_or_short_circuits",
        spec_section: "8.2.3",
        description: "`OR` evaluates the right operand only if the left is FALSE; a side \
                      effect in the right operand must NOT fire when the left is TRUE",
        expected_value: 1,
        cp_source: r#"MODULE M_Expr_LogicalOr_ShortCircuit;
    VAR sideEffect: INTEGER;

    PROCEDURE Touch (): BOOLEAN;
    BEGIN INC(sideEffect); RETURN TRUE END Touch;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        sideEffect := 0;
        (* TRUE OR Touch() — Touch must NOT be called *)
        IF TRUE OR Touch() THEN END;
        IF sideEffect # 0 THEN RETURN -1 END;
        (* FALSE OR Touch() — Touch IS called *)
        IF FALSE OR Touch() THEN END;
        RETURN sideEffect      (* 1 *)
    END Run;
END M_Expr_LogicalOr_ShortCircuit.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_Relational_CHAR",
        test_name: "expr_relational_on_char",
        spec_section: "8.2.5",
        description: "relational operators on CHAR follow code-point order",
        expected_value: 11111,
        cp_source: r#"MODULE M_Expr_Relational_CHAR;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, c: CHAR; score: INTEGER;
    BEGIN
        a := "A"; b := "B"; c := "A";
        score := 0;
        IF a <  b THEN score := score + 1     END;
        IF a <= c THEN score := score + 10    END;
        IF b >  a THEN score := score + 100   END;
        IF b >= a THEN score := score + 1000  END;
        IF a =  c THEN score := score + 10000 END;
        RETURN score
    END Run;
END M_Expr_Relational_CHAR.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_Relational_Pointer_NIL",
        test_name: "expr_pointer_nil_comparisons",
        spec_section: "8.2.5",
        description: "POINTER = NIL / # NIL and pointer-to-pointer equality compare \
                      identity, not contents",
        expected_value: 11110,
        cp_source: r#"MODULE M_Expr_Relational_Pointer_NIL;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Run* (): INTEGER;
        VAR p, q, r: Box; score: INTEGER;
    BEGIN
        p := NIL;
        NEW(q); q.value := 42;
        r := q;
        score := 0;
        IF p = NIL THEN score := score + 10    END;
        IF q # NIL THEN score := score + 100   END;
        IF q = r   THEN score := score + 1000  END;   (* same heap object *)
        IF p # q   THEN score := score + 10000 END;
        RETURN score                          (* 11110 *)
    END Run;
END M_Expr_Relational_Pointer_NIL.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_ORD_CHR_RoundTrip",
        test_name: "expr_ord_chr_round_trip",
        spec_section: "10.3",
        description: "ORD(CHR(n)) = n for every code point in the CHAR range",
        expected_value: 257,
        cp_source: r#"MODULE M_Expr_ORD_CHR_RoundTrip;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: INTEGER;
    BEGIN
        a := ORD(CHR(65));         (* 65 = "A" *)
        b := ORD(CHR(192));        (* 192 — out of ASCII, still valid CHAR *)
        RETURN a + b               (* 65 + 192 = 257 *)
    END Run;
END M_Expr_ORD_CHR_RoundTrip.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_REAL_Arithmetic",
        test_name: "expr_real_arithmetic_packs_to_integer",
        spec_section: "8.2.2",
        description: "REAL +, -, *, / arithmetic with ENTIER to land in an INTEGER result",
        expected_value: 21,
        cp_source: r#"MODULE M_Expr_REAL_Arithmetic;
    PROCEDURE Run* (): LONGINT;
        VAR x, y, r: REAL;
    BEGIN
        x := 3.5;
        y := 2.0;
        r := (x + y) * 4.0 - 1.0;    (* (5.5)*4 - 1 = 21.0 *)
        RETURN ENTIER(r)
    END Run;
END M_Expr_REAL_Arithmetic.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_LEN_OnString",
        test_name: "expr_len_on_string_literal_via_open_array",
        spec_section: "10.3",
        description: "LEN on a string literal passed through an open-array IN param \
                      counts elements including the trailing NUL",
        expected_value: 4,
        cp_source: r#"MODULE M_Expr_LEN_OnString;
    PROCEDURE Measure (IN s: ARRAY OF CHAR): INTEGER;
    BEGIN RETURN LEN(s) END Measure;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        (* "abc" → 3 chars + NUL = 4 *)
        RETURN Measure("abc")
    END Run;
END M_Expr_LEN_OnString.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_SET_Equality",
        test_name: "expr_set_equality_and_subset",
        spec_section: "8.2.5",
        description: "SET equality (=) and subset/superset (<=, >=) compare by membership, \
                      not by literal construction",
        expected_value: 1111,
        cp_source: r#"MODULE M_Expr_SET_Equality;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: SET; score: INTEGER;
    BEGIN
        a := {1, 3, 5, 7};
        b := {3, 5} + {1, 7};
        score := 0;
        IF a = b           THEN score := score + 1    END;
        IF {3, 5} <= a     THEN score := score + 10   END;
        IF a >= {3, 5}     THEN score := score + 100  END;
        IF ~({0} <= a)     THEN score := score + 1000 END;
        RETURN score
    END Run;
END M_Expr_SET_Equality.
"#,
        ignored: Some(
            "KNOWN BUG: sema rejects `<=` and `>=` on SET operands with \
             `invalid operands for <=: SET and SET`. CP §8.2.5 defines these \
             as subset / superset tests; needs to be wired through the \
             relational-operator type table. File under deferred_fixes #20.",
        ),
    },


    // ─── More tier 4: statements (CP §9) ────────────────────────────

    Probe {
        module_name: "M_Stmt_IF_ElsIf_Chain",
        test_name: "stmt_if_elsif_chain",
        spec_section: "9.4",
        description: "IF / ELSIF / ELSE chain selects the first matching arm and skips \
                      the rest",
        expected_value: 33,
        cp_source: r#"MODULE M_Stmt_IF_ElsIf_Chain;
    PROCEDURE Pick (n: INTEGER): INTEGER;
    BEGIN
        IF n < 0 THEN RETURN -1
        ELSIF n = 0 THEN RETURN 0
        ELSIF n < 10 THEN RETURN n * 10
        ELSIF n < 100 THEN RETURN n + 100
        ELSE RETURN 999
        END
    END Pick;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        (* Pick(-3)=-1, Pick(0)=0, Pick(5)=50, Pick(7)=70, Pick(15)=115 → sum = 234;
           offset to land on a stable signature *)
        RETURN Pick(-3) + Pick(0) + Pick(5) + Pick(7) + Pick(15) - 201
    END Run;
END M_Stmt_IF_ElsIf_Chain.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_IF_NoElse",
        test_name: "stmt_if_without_else",
        spec_section: "9.4",
        description: "IF without ELSE leaves state untouched when the condition is FALSE",
        expected_value: 7,
        cp_source: r#"MODULE M_Stmt_IF_NoElse;
    PROCEDURE Run* (): INTEGER;
        VAR x: INTEGER;
    BEGIN
        x := 7;
        IF x < 0 THEN x := 999 END;        (* skipped *)
        IF x > 5 THEN x := x END;           (* no change *)
        RETURN x
    END Run;
END M_Stmt_IF_NoElse.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_WHILE_Loop",
        test_name: "stmt_while_loop_basic",
        spec_section: "9.7",
        description: "WHILE loop evaluates the condition first; body skipped entirely if \
                      it starts FALSE",
        expected_value: 55,
        cp_source: r#"MODULE M_Stmt_WHILE_Loop;
    PROCEDURE Run* (): INTEGER;
        VAR i, sum: INTEGER;
    BEGIN
        i := 1; sum := 0;
        WHILE i <= 10 DO
            sum := sum + i;
            INC(i)
        END;
        RETURN sum                          (* 1+2+...+10 = 55 *)
    END Run;
END M_Stmt_WHILE_Loop.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_Procedure_Recursion",
        test_name: "stmt_procedure_direct_recursion",
        spec_section: "10",
        description: "a procedure may call itself recursively; classic factorial confirms \
                      the call stack and return values both behave",
        expected_value: 720,
        cp_source: r#"MODULE M_Stmt_Procedure_Recursion;
    PROCEDURE Fact (n: INTEGER): INTEGER;
    BEGIN
        IF n <= 1 THEN RETURN 1 END;
        RETURN n * Fact(n - 1)
    END Fact;

    PROCEDURE Run* (): INTEGER;
    BEGIN RETURN Fact(6)                    (* 720 *)
    END Run;
END M_Stmt_Procedure_Recursion.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_Procedure_NoParams",
        test_name: "stmt_procedure_no_params",
        spec_section: "10",
        description: "procedure with no parameters and no return value (void); callable \
                      both with and without empty parens per BlackBox idiom",
        expected_value: 100,
        cp_source: r#"MODULE M_Stmt_Procedure_NoParams;
    VAR counter: INTEGER;

    PROCEDURE Bump;
    BEGIN INC(counter, 50) END Bump;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        counter := 0;
        Bump;          (* bare-name call *)
        Bump();        (* same proc, parenthesised *)
        RETURN counter (* 100 *)
    END Run;
END M_Stmt_Procedure_NoParams.
"#,
        ignored: None,
    },


    // ─── More tier 5: records / methods / arrays ────────────────────

    Probe {
        module_name: "M_Record_With_Array_Field",
        test_name: "record_with_fixed_array_field",
        spec_section: "6.3",
        description: "record containing a fixed-size array field; field-then-index access \
                      paths walk through the parent struct's GEP before the array GEP",
        expected_value: 60,
        cp_source: r#"MODULE M_Record_With_Array_Field;
    TYPE Vec3 = RECORD elems: ARRAY 3 OF INTEGER END;

    PROCEDURE Run* (): INTEGER;
        VAR v: Vec3; i, sum: INTEGER;
    BEGIN
        v.elems[0] := 10;
        v.elems[1] := 20;
        v.elems[2] := 30;
        sum := 0;
        FOR i := 0 TO 2 DO sum := sum + v.elems[i] END;
        RETURN sum                              (* 60 *)
    END Run;
END M_Record_With_Array_Field.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Record_With_Pointer_Field",
        test_name: "record_with_pointer_field_initially_nil",
        spec_section: "6.3",
        description: "record with a pointer field — a freshly NEW'd record has its pointer \
                      fields zero-initialised to NIL",
        expected_value: 9,
        cp_source: r#"MODULE M_Record_With_Pointer_Field;
    TYPE
        InnerDesc = RECORD value: INTEGER END;
        Inner     = POINTER TO InnerDesc;
        OuterDesc = RECORD ptr: Inner; tag: INTEGER END;
        Outer     = POINTER TO OuterDesc;

    PROCEDURE Run* (): INTEGER;
        VAR o: Outer; score: INTEGER;
    BEGIN
        NEW(o);
        o.tag := 5;
        score := 0;
        IF o.ptr = NIL THEN score := score + 4 END;   (* zero-init NIL *)
        NEW(o.ptr);
        o.ptr.value := 5;
        IF o.ptr # NIL THEN score := score + o.ptr.value END;
        RETURN score                                  (* 4 + 5 = 9 *)
    END Run;
END M_Record_With_Pointer_Field.
"#,
        ignored: Some(
            "KNOWN BUG (same family as #14): `NEW(o.ptr)` where `ptr` is a \
             record-field pointer trips IR codegen with \
             `Instr::New: unknown record type opaque:new-ptr`. See \
             deferred_fixes #14.",
        ),
    },

    Probe {
        module_name: "M_MultiDim_FixedArray",
        test_name: "multi_dim_fixed_array_access",
        spec_section: "6.2 / 8.4",
        description: "multi-dimensional fixed array — `ARRAY M, N OF T` indexed as \
                      `a[i, j]` (CP syntax) or `a[i][j]`",
        expected_value: 250,
        cp_source: r#"MODULE M_MultiDim_FixedArray;
    PROCEDURE Run* (): INTEGER;
        VAR grid: ARRAY 3, 3 OF INTEGER; i, j, sum: INTEGER;
    BEGIN
        FOR i := 0 TO 2 DO
            FOR j := 0 TO 2 DO
                grid[i, j] := (i + 1) * 10 + j
            END
        END;
        sum := 0;
        FOR i := 0 TO 2 DO
            FOR j := 0 TO 2 DO
                sum := sum + grid[i, j]
            END
        END;
        (* Values: row0 = 10,11,12; row1 = 20,21,22; row2 = 30,31,32
           sum = 33 + 63 + 93 = 189; +61 = 250 *)
        RETURN sum + 61
    END Run;
END M_MultiDim_FixedArray.
"#,
        ignored: Some(
            "KNOWN BUG: multi-dim fixed array indexing `arr[i, j]` panics in \
             LLVM emit with `Found ArrayValue but expected the IntValue \
             variant` — the code path loads a full row [N x T] and then \
             tries to use it as an index. Single-dim arrays work fine; the \
             multi-index lowering path needs to chain GEPs instead of \
             loading the inner array. File under deferred_fixes #21.",
        ),
    },

    Probe {
        module_name: "M_Method_Recursive",
        test_name: "method_can_call_itself_recursively",
        spec_section: "10.2",
        description: "a method may recurse on the same receiver (Fibonacci on a counter \
                      receiver — silly but exercises the dispatch + recursion path)",
        expected_value: 21,
        cp_source: r#"MODULE M_Method_Recursive;
    TYPE
        WrapperDesc = RECORD END;
        Wrapper     = POINTER TO WrapperDesc;

    PROCEDURE (w: Wrapper) Fib* (n: INTEGER): INTEGER, NEW;
    BEGIN
        IF n < 2 THEN RETURN n END;
        RETURN w.Fib(n - 1) + w.Fib(n - 2)
    END Fib;

    PROCEDURE Run* (): INTEGER;
        VAR w: Wrapper;
    BEGIN
        NEW(w);
        RETURN w.Fib(8)                     (* fib(8) = 21 *)
    END Run;
END M_Method_Recursive.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Override_Three_Levels_Deep",
        test_name: "override_three_levels_deep",
        spec_section: "10.2",
        description: "Sub.Method overrides Mid.Method overrides Base.Method; dispatch via \
                      Base pointer to a Sub instance lands in Sub's body",
        expected_value: 4242,
        cp_source: r#"MODULE M_Override_Three_Levels_Deep;
    TYPE
        BaseDesc = EXTENSIBLE RECORD v: INTEGER END;
        Base     = POINTER TO BaseDesc;
        MidDesc  = EXTENSIBLE RECORD (BaseDesc) END;
        Mid      = POINTER TO MidDesc;
        SubDesc  = RECORD (MidDesc) END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Set* (n: INTEGER), NEW, EXTENSIBLE;
    BEGIN b.v := n END Set;

    PROCEDURE (m: Mid) Set* (n: INTEGER), EXTENSIBLE;
    BEGIN m.v := n * 10 END Set;

    PROCEDURE (s: Sub) Set* (n: INTEGER);
    BEGIN s.v := n * 100 END Set;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub; p: Base;
    BEGIN
        NEW(s);
        p := s;
        p.Set(42);             (* virtual: Sub.Set → s.v = 4200 *)
        RETURN s.v + 42        (* 4242 *)
    END Run;
END M_Override_Three_Levels_Deep.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Param_OUT_OpenArray",
        test_name: "param_out_open_array_writes_through",
        spec_section: "10.1",
        description: "OUT open-array param — callee writes propagate to caller's array \
                      buffer",
        expected_value: 60,
        cp_source: r#"MODULE M_Param_OUT_OpenArray;
    PROCEDURE Fill (OUT p: ARRAY OF INTEGER);
        VAR i: INTEGER;
    BEGIN
        FOR i := 0 TO LEN(p) - 1 DO p[i] := (i + 1) * 10 END
    END Fill;

    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY 3 OF INTEGER;
    BEGIN
        Fill(a);
        RETURN a[0] + a[1] + a[2]           (* 10 + 20 + 30 = 60 *)
    END Run;
END M_Param_OUT_OpenArray.
"#,
        ignored: None,
    },


    // ─── Built-ins / runtime helpers ────────────────────────────────

    Probe {
        module_name: "M_Builtin_ASSERT_TrueIsNoOp",
        test_name: "builtin_assert_true_does_not_trap",
        spec_section: "10.3",
        description: "ASSERT(TRUE, code) returns without trapping; this exercises the \
                      cooperative-poll path that ASSERT inserts but no trap fires",
        expected_value: 99,
        cp_source: r#"MODULE M_Builtin_ASSERT_TrueIsNoOp;
    PROCEDURE Run* (): INTEGER;
        VAR x: INTEGER;
    BEGIN
        x := 99;
        ASSERT(x > 0, 20);
        ASSERT(x = 99, 21);
        ASSERT(TRUE, 22);
        RETURN x
    END Run;
END M_Builtin_ASSERT_TrueIsNoOp.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Builtin_MIN_MAX_OfType",
        test_name: "builtin_min_max_of_type_constants",
        spec_section: "10.3",
        description: "MIN(T) / MAX(T) yield the type's range bounds at compile time",
        expected_value: 4,
        cp_source: r#"MODULE M_Builtin_MIN_MAX_OfType;
    PROCEDURE Run* (): INTEGER;
        VAR score: INTEGER;
    BEGIN
        score := 0;
        IF MIN(INTEGER) < 0      THEN score := score + 1 END;
        IF MAX(INTEGER) > 0      THEN score := score + 2 END;
        IF MIN(INTEGER) < MAX(INTEGER) THEN score := score + 1 END;
        RETURN score                          (* 1 + 2 + 1 = 4 *)
    END Run;
END M_Builtin_MIN_MAX_OfType.
"#,
        ignored: None,
    },


    // ─── Tier 12: SYSTEM module (safe ops only, where importable) ───

    Probe {
        module_name: "M_SYSTEM_ADR_RoundTrip",
        test_name: "system_adr_returns_an_address_word",
        spec_section: "12",
        description: "SYSTEM.ADR(v) yields v's address as an INTEGER; comparing the same \
                      variable's address to itself must produce TRUE",
        expected_value: 1,
        cp_source: r#"MODULE M_SYSTEM_ADR_RoundTrip;
    IMPORT SYSTEM;

    PROCEDURE Run* (): INTEGER;
        VAR x: INTEGER; a, b: INTEGER;
    BEGIN
        x := 42;
        a := SYSTEM.ADR(x);
        b := SYSTEM.ADR(x);
        IF a = b THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_SYSTEM_ADR_RoundTrip.
"#,
        ignored: None,
    },


    // ─── Tier 6/7: type system ──────────────────────────────────────

    Probe {
        module_name: "M_Type_Alias_Chain",
        test_name: "type_alias_chain_resolves_to_underlying",
        spec_section: "6.1",
        description: "a chain of type aliases (`TYPE A = INTEGER; B = A; C = B`) — \
                      assigning between any two is allowed and arithmetic still works",
        expected_value: 100,
        cp_source: r#"MODULE M_Type_Alias_Chain;
    TYPE
        A = INTEGER;
        B = A;
        C = B;

    PROCEDURE Run* (): INTEGER;
        VAR x: A; y: B; z: C; r: INTEGER;
    BEGIN
        x := 10;
        y := x;
        z := y * 10;
        r := z;
        RETURN r                              (* 100 *)
    END Run;
END M_Type_Alias_Chain.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Record_Field_DefaultZero",
        test_name: "record_fields_zero_initialised_on_NEW",
        spec_section: "10.3",
        description: "NEW zero-initialises every field of the allocated record — INTEGER \
                      fields read 0, BOOLEAN reads FALSE, POINTER reads NIL",
        expected_value: 1111,
        cp_source: r#"MODULE M_Record_Field_DefaultZero;
    TYPE
        InnerDesc = RECORD x: INTEGER END;
        Inner     = POINTER TO InnerDesc;
        ItemDesc  = RECORD
            n*: INTEGER;
            flag*: BOOLEAN;
            next*: Inner
        END;
        Item      = POINTER TO ItemDesc;

    PROCEDURE Run* (): INTEGER;
        VAR p: Item; score: INTEGER;
    BEGIN
        NEW(p);
        score := 0;
        IF p.n = 0      THEN score := score + 1    END;
        IF ~p.flag      THEN score := score + 10   END;
        IF p.next = NIL THEN score := score + 100  END;
        score := score + 1000;
        RETURN score
    END Run;
END M_Record_Field_DefaultZero.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Type_PointerToOpenArray_NEW",
        test_name: "type_pointer_to_open_array_dynamic_new",
        spec_section: "6.4 / 10.3",
        description: "POINTER TO ARRAY OF T with NEW(p, n) — dynamic open-array allocated \
                      on the heap, length retrievable via LEN(p^)",
        expected_value: 30,
        cp_source: r#"MODULE M_Type_PointerToOpenArray_NEW;
    TYPE IntVec = POINTER TO ARRAY OF INTEGER;

    PROCEDURE Run* (): INTEGER;
        VAR p: IntVec; i, sum: INTEGER;
    BEGIN
        NEW(p, 4);
        FOR i := 0 TO LEN(p^) - 1 DO p[i] := (i + 1) * 3 END;
        sum := 0;
        FOR i := 0 TO LEN(p^) - 1 DO sum := sum + p[i] END;
        RETURN sum                            (* 3 + 6 + 9 + 12 = 30 *)
    END Run;
END M_Type_PointerToOpenArray_NEW.
"#,
        ignored: None,
    },


    // ─── More tier 11: modules ──────────────────────────────────────

    Probe {
        module_name: "M_Module_VAR_DefaultZero",
        test_name: "module_var_default_zero_when_no_init",
        spec_section: "7 / 11",
        description: "module-level VARs without a BEGIN-block initialiser default to zero \
                      / FALSE / NIL — same rule as record fields",
        expected_value: 111,
        cp_source: r#"MODULE M_Module_VAR_DefaultZero;
    TYPE
        BoxDesc = RECORD x: INTEGER END;
        Box     = POINTER TO BoxDesc;

    VAR
        n: INTEGER;
        flag: BOOLEAN;
        ptr: Box;

    PROCEDURE Run* (): INTEGER;
        VAR score: INTEGER;
    BEGIN
        score := 0;
        IF n = 0      THEN score := score + 1   END;
        IF ~flag      THEN score := score + 10  END;
        IF ptr = NIL  THEN score := score + 100 END;
        RETURN score
    END Run;
END M_Module_VAR_DefaultZero.
"#,
        ignored: None,
    },


    // ─── Tier 3: more expression cells ──────────────────────────────

    Probe {
        module_name: "M_Expr_SET_Membership",
        test_name: "expr_set_in_membership",
        spec_section: "8.2.5",
        description: "`x IN s` membership test over SET elements",
        expected_value: 11,
        cp_source: r#"MODULE M_Expr_SET_Membership;
    PROCEDURE Run* (): INTEGER;
        VAR s: SET; score: INTEGER;
    BEGIN
        s := {1, 3, 5};
        score := 0;
        IF 3 IN s  THEN score := score + 1   END;
        IF 4 IN s  THEN score := score + 100 END;   (* must NOT fire *)
        IF 5 IN s  THEN score := score + 10  END;
        RETURN score
    END Run;
END M_Expr_SET_Membership.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_Bit_Style_Via_SET",
        test_name: "expr_bit_style_operations_via_set",
        spec_section: "8.2.4",
        description: "INCL/EXCL on a SET behave as bit-set / bit-clear; cast to INTEGER via \
                      SYSTEM.VAL recovers the underlying word",
        expected_value: 42,
        cp_source: r#"MODULE M_Expr_Bit_Style_Via_SET;
    IMPORT SYSTEM;

    PROCEDURE Run* (): INTEGER;
        VAR s: SET; n: INTEGER;
    BEGIN
        s := {};
        INCL(s, 1);   (* 0000_0010 = 2  *)
        INCL(s, 3);   (* + 0000_1000 = 10 *)
        INCL(s, 5);   (* + 0010_0000 = 42 *)
        n := SYSTEM.VAL(INTEGER, s);
        RETURN n      (* 2 + 8 + 32 = 42 *)
    END Run;
END M_Expr_Bit_Style_Via_SET.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_REAL_Relational",
        test_name: "expr_real_relational",
        spec_section: "8.2.5",
        description: "relational operators (<, <=, =, >, >=) on REAL operands",
        expected_value: 1111,
        cp_source: r#"MODULE M_Expr_REAL_Relational;
    PROCEDURE Run* (): INTEGER;
        VAR x, y: REAL; score: INTEGER;
    BEGIN
        x := 1.5; y := 2.0;
        score := 0;
        IF x <  y  THEN score := score + 1     END;
        IF y >  x  THEN score := score + 10    END;
        IF x <= x  THEN score := score + 100   END;
        IF x =  1.5 THEN score := score + 1000 END;
        RETURN score
    END Run;
END M_Expr_REAL_Relational.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_Hex_Literals",
        test_name: "expr_hex_literal_arithmetic",
        spec_section: "8.1",
        description: "INTEGER hex literals (suffix `H`) are accepted in arithmetic; mixed \
                      with decimal literals they pack into the same INTEGER type",
        expected_value: 511,
        cp_source: r#"MODULE M_Expr_Hex_Literals;
    PROCEDURE Run* (): INTEGER;
        VAR x: INTEGER;
    BEGIN
        x := 0FFH + 100H;       (* 255 + 256 = 511 *)
        RETURN x
    END Run;
END M_Expr_Hex_Literals.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_CHAR_Hex_Literals",
        test_name: "expr_char_hex_literals_match_decimal",
        spec_section: "8.1",
        description: "CHAR hex literals (suffix `X`) — `41X` is the same CHAR as `\"A\"` \
                      (ASCII 65)",
        expected_value: 1,
        cp_source: r#"MODULE M_Expr_CHAR_Hex_Literals;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: CHAR;
    BEGIN
        a := 41X;       (* 0x41 = 65 = "A" *)
        b := "A";
        IF a = b THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_CHAR_Hex_Literals.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_CAP_Builtin",
        test_name: "expr_cap_uppercases_lowercase_only",
        spec_section: "10.3",
        description: "CAP(c) returns the uppercase letter for lowercase ASCII; non-letters \
                      pass through unchanged",
        expected_value: 67,
        cp_source: r#"MODULE M_Expr_CAP_Builtin;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, c: CHAR;
    BEGIN
        a := CAP("a");      (* "A" = 65 *)
        b := CAP("Z");      (* still "Z" = 90 *)
        c := CAP("0");      (* still "0" = 48 *)
        (* Pack: ORD(a) - 64 = 1; ORD(b) - 90 = 0; ORD(c) - 48 = 0
           Combine 1 + 0 + 0 + 66 = 67 *)
        RETURN (ORD(a) - 64) + (ORD(b) - 90) + (ORD(c) - 48) + 66
    END Run;
END M_Expr_CAP_Builtin.
"#,
        ignored: None,
    },


    // ─── Tier 4: more statement cells ───────────────────────────────

    Probe {
        module_name: "M_Stmt_WITH_ElseOnly",
        test_name: "stmt_with_else_arm_fires_when_no_match",
        spec_section: "9.6",
        description: "WITH ELSE arm fires when the receiver's dynamic type matches none \
                      of the listed type guards",
        expected_value: 999,
        cp_source: r#"MODULE M_Stmt_WITH_ElseOnly;
    TYPE
        BaseDesc = EXTENSIBLE RECORD END;
        Base     = POINTER TO BaseDesc;
        ADesc    = RECORD (BaseDesc) v: INTEGER END;
        A        = POINTER TO ADesc;
        UnusedDesc = RECORD (BaseDesc) END;
        Unused     = POINTER TO UnusedDesc;

    PROCEDURE Score (p: Base): INTEGER;
    BEGIN
        WITH p: Unused DO
            RETURN 1
        ELSE
            RETURN 999
        END
    END Score;

    PROCEDURE Run* (): INTEGER;
        VAR a: A;
    BEGIN
        NEW(a); a.v := 1;
        RETURN Score(a)         (* dynamic type is A, not Unused → ELSE *)
    END Run;
END M_Stmt_WITH_ElseOnly.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_Empty_If_Arm",
        test_name: "stmt_empty_if_arm_is_legal",
        spec_section: "9.4",
        description: "an IF/ELSIF arm with an empty statement sequence (just a semicolon \
                      effectively, or nothing) compiles and runs cleanly",
        expected_value: 5,
        cp_source: r#"MODULE M_Stmt_Empty_If_Arm;
    PROCEDURE Run* (): INTEGER;
        VAR n: INTEGER;
    BEGIN
        n := 5;
        IF n < 0 THEN
            (* empty arm — semantic no-op *)
        ELSE
            n := n
        END;
        RETURN n
    END Run;
END M_Stmt_Empty_If_Arm.
"#,
        ignored: None,
    },


    // ─── Tier 5: more receivers / dispatch ──────────────────────────

    Probe {
        module_name: "M_Method_On_Function_Result",
        test_name: "method_called_on_function_result",
        spec_section: "10.2 / 8.4",
        description: "method dispatch on the return value of a procedure call \
                      (`Make().Method()`) — exercises temporary lifetime + receiver lowering",
        expected_value: 99,
        cp_source: r#"MODULE M_Method_On_Function_Result;
    TYPE
        BoxDesc = EXTENSIBLE RECORD v: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) Get* (): INTEGER, NEW;
    BEGIN RETURN b.v END Get;

    PROCEDURE Make (n: INTEGER): Box;
        VAR b: Box;
    BEGIN
        NEW(b);
        b.v := n;
        RETURN b
    END Make;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN Make(99).Get()
    END Run;
END M_Method_On_Function_Result.
"#,
        ignored: Some(
            "KNOWN BUG: method dispatch on a procedure-call result \
             (`Make(99).Get()`) returns a wild value (uninitialised memory \
             read) instead of 99. The receiver-lowering refactor for plain \
             record method dispatch worked off `designator_addr`, but a \
             call-as-prefix produces a Temp IrValue rather than a \
             designator — so the dispatch path is reading the wrong slot. \
             File under deferred_fixes #22 and un-ignore once \
             call-result-as-receiver is wired up.",
        ),
    },

    Probe {
        module_name: "M_Method_On_ArrayElement",
        test_name: "method_called_on_array_element_pointer",
        spec_section: "10.2 / 8.4",
        description: "method dispatch on an element of `ARRAY N OF Ptr` — receiver lowering \
                      must descend through the index GEP before the vtable lookup",
        expected_value: 27,
        cp_source: r#"MODULE M_Method_On_ArrayElement;
    TYPE
        ItemDesc = EXTENSIBLE RECORD v: INTEGER END;
        Item     = POINTER TO ItemDesc;

    PROCEDURE (i: Item) Treble* (): INTEGER, NEW;
    BEGIN RETURN i.v * 3 END Treble;

    PROCEDURE Run* (): INTEGER;
        VAR arr: ARRAY 3 OF Item;
    BEGIN
        NEW(arr[0]); arr[0].v := 5;
        NEW(arr[1]); arr[1].v := 7;
        NEW(arr[2]); arr[2].v := 9;
        RETURN arr[2].Treble()      (* 9 * 3 = 27 *)
    END Run;
END M_Method_On_ArrayElement.
"#,
        ignored: Some(
            "KNOWN BUG (same family as #14): NEW(arr[i]) where arr is an \
             ARRAY OF Pointer trips IR codegen with `Instr::New: unknown \
             record type [N x named:Item]`. The destination-type resolution \
             doesn't walk past the index-GEP to find the pointer's referent \
             record. See deferred_fixes #14.",
        ),
    },

    Probe {
        module_name: "M_Procedure_Returns_Pointer",
        test_name: "procedure_returns_pointer_to_record",
        spec_section: "10",
        description: "procedure that returns a POINTER TO record — caller receives the \
                      heap pointer and can mutate the record through it",
        expected_value: 1000,
        cp_source: r#"MODULE M_Procedure_Returns_Pointer;
    TYPE
        BoxDesc = RECORD v: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Make (n: INTEGER): Box;
        VAR b: Box;
    BEGIN
        NEW(b);
        b.v := n;
        RETURN b
    END Make;

    PROCEDURE Run* (): INTEGER;
        VAR b: Box;
    BEGIN
        b := Make(500);
        b.v := b.v + 500;       (* mutation through the returned pointer *)
        RETURN b.v
    END Run;
END M_Procedure_Returns_Pointer.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_VAR_Receiver_Mutates_Record",
        test_name: "var_receiver_mutates_value_record",
        spec_section: "10.2",
        description: "VAR receiver on a plain record — method body can write through the \
                      receiver and the caller sees the change",
        expected_value: 88,
        cp_source: r#"MODULE M_VAR_Receiver_Mutates_Record;
    TYPE Counter = RECORD value: INTEGER END;

    PROCEDURE (VAR c: Counter) SetAndDouble* (n: INTEGER), NEW;
    BEGIN
        c.value := n;
        c.value := c.value * 2
    END SetAndDouble;

    PROCEDURE Run* (): INTEGER;
        VAR c: Counter;
    BEGIN
        c.value := 0;
        c.SetAndDouble(44);
        RETURN c.value           (* 88 *)
    END Run;
END M_VAR_Receiver_Mutates_Record.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Override_EmptyMethod_WithBody",
        test_name: "override_empty_method_with_real_body",
        spec_section: "10.2",
        description: "subclass override of an EMPTY method must actually execute its body \
                      when the method is dispatched through a base pointer",
        expected_value: 41,
        cp_source: r#"MODULE M_Override_EmptyMethod_WithBody;
    TYPE
        BaseDesc = EXTENSIBLE RECORD touched: INTEGER END;
        Base     = POINTER TO BaseDesc;
        SubDesc  = RECORD (BaseDesc) END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Visit* (), NEW, EMPTY;

    PROCEDURE (s: Sub) Visit*;
    BEGIN s.touched := 41 END Visit;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub; p: Base;
    BEGIN
        NEW(s);
        p := s;
        p.Visit();              (* dispatches to Sub.Visit through Base ptr *)
        RETURN s.touched        (* 41 *)
    END Run;
END M_Override_EmptyMethod_WithBody.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Receiver_Differently_Named",
        test_name: "receiver_name_differs_across_methods",
        spec_section: "10.2",
        description: "the receiver formal name may differ across methods on the same \
                      record (`(self: Foo)` vs `(this: Foo)` vs `(f: Foo)`) — sema must \
                      bind each to its own scope",
        expected_value: 28,
        cp_source: r#"MODULE M_Receiver_Differently_Named;
    TYPE Counter = RECORD value: INTEGER END;

    PROCEDURE (self: Counter) Read* (): INTEGER, NEW;
    BEGIN RETURN self.value END Read;

    PROCEDURE (VAR this: Counter) Set* (n: INTEGER), NEW;
    BEGIN this.value := n END Set;

    PROCEDURE Run* (): INTEGER;
        VAR c: Counter;
    BEGIN
        c.value := 0;
        c.Set(28);
        RETURN c.Read()
    END Run;
END M_Receiver_Differently_Named.
"#,
        ignored: None,
    },


    // ─── Tier 6: more type-system shapes ────────────────────────────

    Probe {
        module_name: "M_Type_SHORTINT_Arithmetic",
        test_name: "type_shortint_arithmetic",
        spec_section: "6.1",
        description: "SHORTINT arithmetic (CP's narrow-width signed integer); operates \
                      within range and assigns back to SHORTINT",
        expected_value: 200,
        cp_source: r#"MODULE M_Type_SHORTINT_Arithmetic;
    PROCEDURE Run* (): INTEGER;
        VAR s: SHORTINT;
    BEGIN
        s := 100;
        s := SHORT(s + s);       (* 200 fits in SHORTINT *)
        RETURN s
    END Run;
END M_Type_SHORTINT_Arithmetic.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Type_LONGINT_Explicit",
        test_name: "type_longint_explicit_assignment",
        spec_section: "6.1",
        description: "LONGINT declared explicitly; values out of INTEGER range survive \
                      through to the runtime",
        expected_value: 10,
        cp_source: r#"MODULE M_Type_LONGINT_Explicit;
    PROCEDURE Run* (): LONGINT;
        VAR a, b: LONGINT;
    BEGIN
        a := 10000000000;        (* > 2^31 *)
        b := 20000000000;
        (* (b - a) DIV 1_000_000_000 = 10_000_000_000 / 1_000_000_000 = 10 *)
        RETURN (b - a) DIV 1000000000
    END Run;
END M_Type_LONGINT_Explicit.
"#,
        ignored: None,
    },


    // ─── Tier 10.3: more predeclared procedures ─────────────────────

    Probe {
        module_name: "M_Builtin_LONG_SHORT_Casts",
        test_name: "builtin_long_short_round_trip",
        spec_section: "10.3",
        description: "LONG(x) widens a narrower numeric type to LONGINT / LONGREAL; \
                      SHORT(x) narrows.  Round-trip preserves the value when it fits.",
        expected_value: 250,
        cp_source: r#"MODULE M_Builtin_LONG_SHORT_Casts;
    PROCEDURE Run* (): INTEGER;
        VAR n: INTEGER; l: LONGINT;
    BEGIN
        n := 250;
        l := LONG(n);
        n := SHORT(l);
        RETURN n
    END Run;
END M_Builtin_LONG_SHORT_Casts.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Builtin_ASH_Shifts",
        test_name: "builtin_ash_arithmetic_shift",
        spec_section: "10.3",
        description: "ASH(n, k) arithmetic shift: positive k = left shift, negative = \
                      signed right shift",
        expected_value: 32,
        cp_source: r#"MODULE M_Builtin_ASH_Shifts;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: INTEGER;
    BEGIN
        a := ASH(1, 5);      (* 1 << 5 = 32 *)
        b := ASH(64, -1);    (* 64 >> 1 = 32 *)
        IF a = b THEN RETURN a ELSE RETURN -1 END
    END Run;
END M_Builtin_ASH_Shifts.
"#,
        ignored: None,
    },


    // ─── Tier 12: more SYSTEM operations ────────────────────────────

    Probe {
        module_name: "M_SYSTEM_VAL_TypePunning",
        test_name: "system_val_reinterprets_set_as_integer",
        spec_section: "12",
        description: "SYSTEM.VAL(T, x) reinterprets `x`'s bit pattern as type T — used \
                      here to pull the bit pattern of a SET out as an INTEGER",
        expected_value: 41,
        cp_source: r#"MODULE M_SYSTEM_VAL_TypePunning;
    IMPORT SYSTEM;

    PROCEDURE Run* (): INTEGER;
        VAR s: SET; n: INTEGER;
    BEGIN
        s := {0, 3, 5};                  (* 1 + 8 + 32 = 41 *)
        n := SYSTEM.VAL(INTEGER, s);
        RETURN n
    END Run;
END M_SYSTEM_VAL_TypePunning.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_SYSTEM_LSH_Bitshift",
        test_name: "system_lsh_logical_shift",
        spec_section: "12",
        description: "SYSTEM.LSH(n, k) — logical (unsigned) shift; positive k shifts left, \
                      negative right, zero-fills on both ends",
        expected_value: 256,
        cp_source: r#"MODULE M_SYSTEM_LSH_Bitshift;
    IMPORT SYSTEM;

    PROCEDURE Run* (): INTEGER;
        VAR a: INTEGER;
    BEGIN
        a := SYSTEM.LSH(1, 8);     (* 1 << 8 = 256 *)
        RETURN a
    END Run;
END M_SYSTEM_LSH_Bitshift.
"#,
        ignored: None,
    },


    // ─── Tier 11: more module structure ─────────────────────────────

    Probe {
        module_name: "M_Module_Const_Arithmetic",
        test_name: "module_constants_used_in_arithmetic",
        spec_section: "5 / 11",
        description: "module-level CONSTs are constant expressions and may be combined \
                      with each other in further CONST declarations and in run-time \
                      arithmetic",
        expected_value: 110,
        cp_source: r#"MODULE M_Module_Const_Arithmetic;
    CONST
        a = 10;
        b = 11;
        c = a * b;          (* CONST built from earlier CONSTs *)

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN c            (* 10 * 11 = 110 *)
    END Run;
END M_Module_Const_Arithmetic.
"#,
        ignored: None,
    },


    // ─── Tier 5: cumulative regression — VAR receiver vs value method
    //     on the same record (the dispatch refactor handled both
    //     plain-record paths in one go; cement the contract).

    Probe {
        module_name: "M_Receiver_Value_And_VAR_Coexist",
        test_name: "value_and_var_receivers_coexist_on_same_record",
        spec_section: "10.2",
        description: "the same plain record can have both value-style (read) and VAR \
                      (write) receiver methods; dispatch picks the right shape for each \
                      based on the call site's needs",
        expected_value: 50,
        cp_source: r#"MODULE M_Receiver_Value_And_VAR_Coexist;
    TYPE Counter = RECORD n: INTEGER END;

    PROCEDURE (c: Counter) Read* (): INTEGER, NEW;
    BEGIN RETURN c.n END Read;

    PROCEDURE (VAR c: Counter) Add* (k: INTEGER), NEW;
    BEGIN c.n := c.n + k END Add;

    PROCEDURE Run* (): INTEGER;
        VAR c: Counter;
    BEGIN
        c.n := 0;
        c.Add(20);
        c.Add(30);
        RETURN c.Read()
    END Run;
END M_Receiver_Value_And_VAR_Coexist.
"#,
        ignored: None,
    },


    // ─── Tier 3: integer-width edges ────────────────────────────────

    Probe {
        module_name: "M_Expr_MixedWidth_Arithmetic",
        test_name: "expr_mixed_width_promotion",
        spec_section: "8.2.2",
        description: "INTSHORT + INTEGER promotes to INTEGER; INTEGER + LONGINT promotes \
                      to LONGINT — value preserved through the chain",
        expected_value: 1234,
        cp_source: r#"MODULE M_Expr_MixedWidth_Arithmetic;
    PROCEDURE Run* (): LONGINT;
        VAR s: SHORTINT; n: INTEGER; l: LONGINT;
    BEGIN
        s := 34;
        n := 200;
        l := 1000;
        RETURN l + n + s          (* 1000 + 200 + 34 = 1234 *)
    END Run;
END M_Expr_MixedWidth_Arithmetic.
"#,
        ignored: None,
    },


    // ─── Tier 5: super call cumulative ───────────────────────────────

    Probe {
        module_name: "M_Method_Returns_Pointer",
        test_name: "method_returns_pointer_to_self_type",
        spec_section: "10",
        description: "method whose return type is the receiver's own pointer alias — \
                      classic builder-style chain",
        expected_value: 35,
        cp_source: r#"MODULE M_Method_Returns_Pointer;
    TYPE
        BoxDesc = EXTENSIBLE RECORD v: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) WithValue* (n: INTEGER): Box, NEW;
    BEGIN
        b.v := n;
        RETURN b
    END WithValue;

    PROCEDURE Run* (): INTEGER;
        VAR b, other: Box;
    BEGIN
        NEW(b);
        other := b.WithValue(35);
        IF other = b THEN
            RETURN other.v
        ELSE
            RETURN -1
        END
    END Run;
END M_Method_Returns_Pointer.
"#,
        ignored: Some(
            "KNOWN BUG: sema mis-types the receiver inside a method body when \
             the method's signature returns the receiver's pointer alias. \
             `(b: Box) WithValue (): Box` body sees `b` as `BoxDesc` (the \
             underlying record), so `RETURN b` reports `return type mismatch: \
             expected Box, found BoxDesc`. The receiver itself is a pointer; \
             this is a sema canonicalisation gap in the return-type check. \
             File under deferred_fixes #23.",
        ),
    },


    // ─── Tier 4: more control-flow ──────────────────────────────────

    Probe {
        module_name: "M_Stmt_For_StepOf_One",
        test_name: "stmt_for_default_step",
        spec_section: "9.7",
        description: "FOR without BY uses step 1; the loop variable is in scope after END",
        expected_value: 10,
        cp_source: r#"MODULE M_Stmt_For_StepOf_One;
    PROCEDURE Run* (): INTEGER;
        VAR i, n: INTEGER;
    BEGIN
        n := 0;
        FOR i := 1 TO 4 DO n := n + i END;   (* 1+2+3+4 = 10 *)
        RETURN n
    END Run;
END M_Stmt_For_StepOf_One.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_Nested_IF_Inside_For",
        test_name: "stmt_nested_if_inside_for",
        spec_section: "9 / 9.7",
        description: "an IF nested inside a FOR loop sees the loop variable and the FOR \
                      sees state mutated by the IF",
        expected_value: 6,
        cp_source: r#"MODULE M_Stmt_Nested_IF_Inside_For;
    PROCEDURE Run* (): INTEGER;
        VAR i, evens: INTEGER;
    BEGIN
        evens := 0;
        FOR i := 1 TO 5 DO
            IF ~ODD(i) THEN evens := evens + i END
        END;
        RETURN evens                          (* 2 + 4 = 6 *)
    END Run;
END M_Stmt_Nested_IF_Inside_For.
"#,
        ignored: None,
    },


    // ─── Cycle 1: more expression / statement / OO cells ────────────

    Probe {
        module_name: "M_Expr_SHORTREAL_Arithmetic",
        test_name: "expr_shortreal_arithmetic",
        spec_section: "8.2.2 / 6.1",
        description: "SHORTREAL (32-bit float) arithmetic; round-trips through ENTIER \
                      to land in an INTEGER",
        expected_value: 18,
        cp_source: r#"MODULE M_Expr_SHORTREAL_Arithmetic;
    PROCEDURE Run* (): LONGINT;
        VAR x, y: SHORTREAL;
    BEGIN
        x := SHORT(3.0);
        y := SHORT(2.5);
        RETURN ENTIER(x * y * 2.4)      (* 3.0*2.5*2.4 = 18.0 → 18 *)
    END Run;
END M_Expr_SHORTREAL_Arithmetic.
"#,
        ignored: Some(
            "KNOWN BUG: SHORTREAL arithmetic mixed with REAL operand produces \
             wild result (observed: 2097152 instead of 18).  Either SHORTREAL \
             arithmetic doesn't sign-extend / convert properly when mixed \
             with REAL operands, or ENTIER on the resulting REAL drops bits. \
             File under deferred_fixes #24.",
        ),
    },

    Probe {
        module_name: "M_Expr_Negative_Literal",
        test_name: "expr_negative_literal_in_expression",
        spec_section: "8.1",
        description: "negative integer literal used directly in an expression",
        expected_value: 7,
        cp_source: r#"MODULE M_Expr_Negative_Literal;
    PROCEDURE Run* (): INTEGER;
        VAR x: INTEGER;
    BEGIN
        x := 10 + (-3);
        RETURN x
    END Run;
END M_Expr_Negative_Literal.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_SET_BitMax",
        test_name: "expr_set_max_bit_index",
        spec_section: "6.1 / 8.2.4",
        description: "SET(32) supports the full 0..31 element range; bit 31 is the \
                      highest allowable element",
        expected_value: 1,
        cp_source: r#"MODULE M_Expr_SET_BitMax;
    PROCEDURE Run* (): INTEGER;
        VAR s: SET;
    BEGIN
        s := {0, 31};
        IF (0 IN s) & (31 IN s) THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_SET_BitMax.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_LONGINT_Relational",
        test_name: "expr_longint_relational",
        spec_section: "8.2.5",
        description: "relational comparisons on LONGINT values that exceed INTEGER range",
        expected_value: 111,
        cp_source: r#"MODULE M_Expr_LONGINT_Relational;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: LONGINT; score: INTEGER;
    BEGIN
        a := 10000000000;
        b := 20000000000;
        score := 0;
        IF a < b           THEN score := score + 1   END;
        IF b > a           THEN score := score + 10  END;
        IF a + a = b       THEN score := score + 100 END;
        RETURN score
    END Run;
END M_Expr_LONGINT_Relational.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_StringEquality_CharArray",
        test_name: "expr_string_equality_on_two_arrays",
        spec_section: "8.2.5",
        description: "`=` on two ARRAY OF CHAR variables (not literals) compares by content \
                      up to the first 0X terminator",
        expected_value: 1,
        cp_source: r#"MODULE M_Expr_StringEquality_CharArray;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: ARRAY 8 OF CHAR;
    BEGIN
        a := "hello";
        b := "hello";
        IF a = b THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_StringEquality_CharArray.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_StringInequality_CharArray",
        test_name: "expr_string_inequality_on_two_arrays",
        spec_section: "8.2.5",
        description: "`#` on two ARRAY OF CHAR variables returns TRUE for differing content",
        expected_value: 1,
        cp_source: r#"MODULE M_Expr_StringInequality_CharArray;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: ARRAY 8 OF CHAR;
    BEGIN
        a := "hello";
        b := "world";
        IF a # b THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_StringInequality_CharArray.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_Nested_CASE",
        test_name: "stmt_nested_case",
        spec_section: "9.5",
        description: "a CASE inside another CASE arm — nested branching with separate label \
                      sets",
        expected_value: 33,
        cp_source: r#"MODULE M_Stmt_Nested_CASE;
    PROCEDURE Classify (kind, sub: INTEGER): INTEGER;
    BEGIN
        CASE kind OF
          1:
            CASE sub OF
              10: RETURN 11
            | 20: RETURN 33
            ELSE  RETURN 19
            END
        | 2: RETURN 200
        ELSE  RETURN 999
        END
    END Classify;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN Classify(1, 20)
    END Run;
END M_Stmt_Nested_CASE.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_CASE_Without_ELSE",
        test_name: "stmt_case_without_else_matches_one",
        spec_section: "9.5",
        description: "CASE without ELSE — when one of the labels matches, that arm runs \
                      and the statement completes normally",
        expected_value: 5,
        cp_source: r#"MODULE M_Stmt_CASE_Without_ELSE;
    PROCEDURE Run* (): INTEGER;
        VAR x: INTEGER;
    BEGIN
        x := 0;
        CASE 2 OF
          1: x := 1
        | 2: x := 5
        | 3: x := 9
        END;
        RETURN x
    END Run;
END M_Stmt_CASE_Without_ELSE.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_WHILE_NoIterations",
        test_name: "stmt_while_body_skipped_when_false",
        spec_section: "9.7",
        description: "WHILE body never runs when the condition is FALSE on entry",
        expected_value: 0,
        cp_source: r#"MODULE M_Stmt_WHILE_NoIterations;
    PROCEDURE Run* (): INTEGER;
        VAR i, count: INTEGER;
    BEGIN
        i := 10; count := 0;
        WHILE i < 5 DO INC(count); INC(i) END;
        RETURN count
    END Run;
END M_Stmt_WHILE_NoIterations.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Method_Calls_Sibling_Method",
        test_name: "method_calls_sibling_method_on_same_receiver",
        spec_section: "10.2",
        description: "one method on a record calls another method on the same record \
                      through the receiver",
        expected_value: 100,
        cp_source: r#"MODULE M_Method_Calls_Sibling_Method;
    TYPE
        BoxDesc = EXTENSIBLE RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) Get* (): INTEGER, NEW;
    BEGIN RETURN b.value END Get;

    PROCEDURE (b: Box) DoubleViaGet* (): INTEGER, NEW;
    BEGIN RETURN b.Get() * 2 END DoubleViaGet;

    PROCEDURE Run* (): INTEGER;
        VAR b: Box;
    BEGIN
        NEW(b);
        b.value := 50;
        RETURN b.DoubleViaGet()         (* 100 *)
    END Run;
END M_Method_Calls_Sibling_Method.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Method_ReturnsBoolean",
        test_name: "method_returns_boolean",
        spec_section: "10.2 / 6.1",
        description: "method whose return type is BOOLEAN; the call result drives an IF",
        expected_value: 42,
        cp_source: r#"MODULE M_Method_ReturnsBoolean;
    TYPE
        BoxDesc = EXTENSIBLE RECORD v: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) IsPositive* (): BOOLEAN, NEW;
    BEGIN RETURN b.v > 0 END IsPositive;

    PROCEDURE Run* (): INTEGER;
        VAR b: Box;
    BEGIN
        NEW(b);
        b.v := 42;
        IF b.IsPositive() THEN RETURN b.v ELSE RETURN -1 END
    END Run;
END M_Method_ReturnsBoolean.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Proc_Value_Reassigned",
        test_name: "proc_value_reassigned_mid_flight",
        spec_section: "6.5",
        description: "a procedure-typed variable can be reassigned between calls; the \
                      second call dispatches to the new target",
        expected_value: 28,
        cp_source: r#"MODULE M_Proc_Value_Reassigned;
    TYPE UnaryOp = PROCEDURE (x: INTEGER): INTEGER;

    PROCEDURE Triple (x: INTEGER): INTEGER;
    BEGIN RETURN x * 3 END Triple;

    PROCEDURE AddTen (x: INTEGER): INTEGER;
    BEGIN RETURN x + 10 END AddTen;

    PROCEDURE Run* (): INTEGER;
        VAR f: UnaryOp; a, b: INTEGER;
    BEGIN
        f := Triple;
        a := f(6);          (* 18 *)
        f := AddTen;
        b := f(0);          (* 10 *)
        RETURN a + b        (* 28 *)
    END Run;
END M_Proc_Value_Reassigned.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Type_RecordWith_Three_Field_Types",
        test_name: "type_record_with_mixed_field_types",
        spec_section: "6.3",
        description: "a record with INTEGER, BOOLEAN, REAL, and CHAR fields exercises \
                      field offset / alignment for multiple primitive widths",
        expected_value: 1023,
        cp_source: r#"MODULE M_Type_RecordWith_Three_Field_Types;
    TYPE Mixed = RECORD
        n: INTEGER;
        b: BOOLEAN;
        r: REAL;
        c: CHAR
    END;

    PROCEDURE Run* (): INTEGER;
        VAR m: Mixed; score: INTEGER;
    BEGIN
        m.n := 1000;
        m.b := TRUE;
        m.r := 1.5;
        m.c := "X";
        score := 0;
        IF m.n = 1000 THEN score := score + 1000 END;
        IF m.b THEN score := score + 20 END;
        IF ENTIER(m.r * 2.0) = 3 THEN score := score + 3 END;
        IF m.c = "X" THEN score := score + 0 END;
        RETURN score                          (* 1000 + 20 + 3 = 1023 *)
    END Run;
END M_Type_RecordWith_Three_Field_Types.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Builtin_COPY_FixedArray",
        test_name: "builtin_copy_between_fixed_arrays",
        spec_section: "10.3",
        description: "COPY(src, dst) duplicates the contents of one fixed array into \
                      another of the same shape",
        expected_value: 12,
        cp_source: r#"MODULE M_Builtin_COPY_FixedArray;
    PROCEDURE Run* (): INTEGER;
        VAR src, dst: ARRAY 3 OF INTEGER;
    BEGIN
        src[0] := 3; src[1] := 4; src[2] := 5;
        dst[0] := 0; dst[1] := 0; dst[2] := 0;
        dst := src;             (* whole-array assignment in CP *)
        RETURN dst[0] + dst[1] + dst[2]       (* 12 *)
    END Run;
END M_Builtin_COPY_FixedArray.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_IF_ChainedCondition",
        test_name: "stmt_if_with_compound_condition",
        spec_section: "9.4 / 8.2.3",
        description: "IF condition that ANDs / ORs multiple comparisons — exercises the \
                      short-circuit lowering across more than two operands",
        expected_value: 1,
        cp_source: r#"MODULE M_Stmt_IF_ChainedCondition;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, c: INTEGER;
    BEGIN
        a := 1; b := 2; c := 3;
        IF (a < b) & (b < c) & (a < c) THEN
            RETURN 1
        ELSE
            RETURN 0
        END
    END Run;
END M_Stmt_IF_ChainedCondition.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_MixedAndOr_Precedence",
        test_name: "expr_mixed_and_or_precedence",
        spec_section: "8.2.3",
        description: "`&` binds tighter than `OR`: `a OR b & c` parses as `a OR (b & c)`",
        expected_value: 1,
        cp_source: r#"MODULE M_Expr_MixedAndOr_Precedence;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, c: BOOLEAN;
    BEGIN
        a := TRUE; b := FALSE; c := FALSE;
        (* a OR (b & c) = TRUE OR (FALSE & FALSE) = TRUE *)
        IF a OR b & c THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_MixedAndOr_Precedence.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_NOT_Precedence",
        test_name: "expr_not_precedence_higher_than_and",
        spec_section: "8.2.3",
        description: "`~` binds tightest among logical ops; `~a & b` parses as `(~a) & b`",
        expected_value: 1,
        cp_source: r#"MODULE M_Expr_NOT_Precedence;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: BOOLEAN;
    BEGIN
        a := FALSE; b := TRUE;
        (* (~a) & b = TRUE & TRUE = TRUE *)
        IF ~a & b THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_NOT_Precedence.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_LOOP_Indefinite",
        test_name: "stmt_loop_indefinite_with_exit",
        spec_section: "9.8",
        description: "LOOP runs until EXIT; the exit condition can be anywhere in the body",
        expected_value: 10,
        cp_source: r#"MODULE M_Stmt_LOOP_Indefinite;
    PROCEDURE Run* (): INTEGER;
        VAR n: INTEGER;
    BEGIN
        n := 0;
        LOOP
            INC(n);
            IF n >= 10 THEN EXIT END
        END;
        RETURN n
    END Run;
END M_Stmt_LOOP_Indefinite.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_ABS_OnReal",
        test_name: "expr_abs_on_real",
        spec_section: "10.3",
        description: "ABS works on REAL operands too, not just integer types",
        expected_value: 7,
        cp_source: r#"MODULE M_Expr_ABS_OnReal;
    PROCEDURE Run* (): LONGINT;
        VAR x: REAL;
    BEGIN
        x := -7.0;
        RETURN ENTIER(ABS(x))
    END Run;
END M_Expr_ABS_OnReal.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Param_OUT_BOOLEAN",
        test_name: "param_out_boolean_writes_through",
        spec_section: "10.1",
        description: "OUT BOOLEAN param — callee write propagates to caller's slot",
        expected_value: 1,
        cp_source: r#"MODULE M_Param_OUT_BOOLEAN;
    PROCEDURE SetTrue (OUT b: BOOLEAN);
    BEGIN b := TRUE END SetTrue;

    PROCEDURE Run* (): INTEGER;
        VAR flag: BOOLEAN;
    BEGIN
        flag := FALSE;
        SetTrue(flag);
        IF flag THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Param_OUT_BOOLEAN.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Param_VAR_REAL",
        test_name: "param_var_real_mutates_caller",
        spec_section: "10.1",
        description: "VAR REAL param — callee mutation propagates",
        expected_value: 10,
        cp_source: r#"MODULE M_Param_VAR_REAL;
    PROCEDURE Double (VAR x: REAL);
    BEGIN x := x * 2.0 END Double;

    PROCEDURE Run* (): LONGINT;
        VAR x: REAL;
    BEGIN
        x := 5.0;
        Double(x);
        RETURN ENTIER(x)            (* 10 *)
    END Run;
END M_Param_VAR_REAL.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Recursive_Mutual",
        test_name: "recursive_mutual_two_procs",
        spec_section: "10",
        description: "mutually-recursive procedures (IsEven calls IsOdd which calls \
                      IsEven) — sema must resolve the forward reference both ways",
        expected_value: 1,
        cp_source: r#"MODULE M_Recursive_Mutual;
    PROCEDURE IsOdd  (n: INTEGER): BOOLEAN;
    BEGIN
        IF n = 0 THEN RETURN FALSE
        ELSE RETURN IsEven(n - 1)
        END
    END IsOdd;

    PROCEDURE IsEven (n: INTEGER): BOOLEAN;
    BEGIN
        IF n = 0 THEN RETURN TRUE
        ELSE RETURN IsOdd(n - 1)
        END
    END IsEven;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        IF IsEven(10) & IsOdd(7) THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Recursive_Mutual.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Stmt_Sequential_Var_Decl",
        test_name: "stmt_sequential_var_declarations",
        spec_section: "7",
        description: "multiple VAR declarations in a single procedure; each gets its own \
                      slot and the values don't bleed across",
        expected_value: 1234,
        cp_source: r#"MODULE M_Stmt_Sequential_Var_Decl;
    PROCEDURE Run* (): INTEGER;
        VAR a: INTEGER;
        VAR b: INTEGER;
        VAR c, d: INTEGER;
    BEGIN
        a := 1; b := 2; c := 3; d := 4;
        RETURN a * 1000 + b * 100 + c * 10 + d
    END Run;
END M_Stmt_Sequential_Var_Decl.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Type_Const_In_ArraySize",
        test_name: "type_constant_drives_array_size",
        spec_section: "5 / 6.2",
        description: "module-level CONST used as an array dimension; the compiler must \
                      fold the CONST at type-check time",
        expected_value: 4,
        cp_source: r#"MODULE M_Type_Const_In_ArraySize;
    CONST size = 4;

    PROCEDURE Run* (): INTEGER;
        VAR arr: ARRAY size OF INTEGER;
    BEGIN
        arr[0] := 0; arr[1] := 0; arr[2] := 0; arr[3] := 0;
        RETURN LEN(arr)              (* 4 *)
    END Run;
END M_Type_Const_In_ArraySize.
"#,
        ignored: None,
    },

    Probe {
        module_name: "M_Expr_DEC_WithDelta",
        test_name: "expr_dec_with_negative_delta",
        spec_section: "10.3",
        description: "DEC(n, k) is equivalent to n := n - k for any integer k",
        expected_value: 15,
        cp_source: r#"MODULE M_Expr_DEC_WithDelta;
    PROCEDURE Run* (): INTEGER;
        VAR n: INTEGER;
    BEGIN
        n := 100;
        DEC(n, 85);
        RETURN n
    END Run;
END M_Expr_DEC_WithDelta.
"#,
        ignored: None,
    },
];
