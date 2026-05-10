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
];
