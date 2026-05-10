MODULE M_Builtin_ASSERT_TrueIsNoOp;
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
