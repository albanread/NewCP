MODULE M_Expr_LONGINT_BigArithmetic;
    PROCEDURE Run* (): LONGINT;
        VAR x: LONGINT;
    BEGIN
        x := 1000000;
        RETURN x * x                                  (* 10^12; overflows i32 *)
    END Run;
END M_Expr_LONGINT_BigArithmetic.
