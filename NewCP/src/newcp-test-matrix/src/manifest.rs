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
];
