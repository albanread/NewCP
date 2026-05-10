MODULE M_Expr_INC_OnByte;
    PROCEDURE Run* (): INTEGER;
        VAR b: BYTE;
    BEGIN
        b := SHORT(SHORT(SHORT(100)));
        INC(b, 50);
        RETURN b
    END Run;
END M_Expr_INC_OnByte.
