MODULE M_Expr_Constant_Fold_InArraySize;
    CONST k = 2*3 + 4 - 1;                            (* folds to 9 *)

    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY k OF INTEGER;
    BEGIN
        RETURN LEN(a)
    END Run;
END M_Expr_Constant_Fold_InArraySize.
