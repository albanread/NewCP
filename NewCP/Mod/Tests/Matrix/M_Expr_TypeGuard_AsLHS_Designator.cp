MODULE M_Expr_TypeGuard_AsLHS_Designator;
    TYPE
        BaseDesc = EXTENSIBLE RECORD END;
        Base     = POINTER TO BaseDesc;
        SubDesc  = RECORD (BaseDesc) extra: INTEGER END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub; b: Base;
    BEGIN
        NEW(s);
        b := s;
        b(Sub).extra := 99;          (* type guard on the LHS *)
        RETURN b(Sub).extra
    END Run;
END M_Expr_TypeGuard_AsLHS_Designator.
