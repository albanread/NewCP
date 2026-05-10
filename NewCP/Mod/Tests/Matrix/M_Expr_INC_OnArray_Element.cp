MODULE M_Expr_INC_OnArray_Element;
    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY 3 OF INTEGER;
    BEGIN
        a[0] := 10; a[1] := 20; a[2] := 30;
        INC(a[1], 28);
        RETURN a[0] + a[1] + a[2]            (* 10 + 48 + 30 = 88 *)
    END Run;
END M_Expr_INC_OnArray_Element.
