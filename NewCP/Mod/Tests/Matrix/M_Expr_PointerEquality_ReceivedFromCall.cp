MODULE M_Expr_PointerEquality_ReceivedFromCall;
    TYPE
        BoxDesc = RECORD v: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Run* (): INTEGER;
        VAR a, b, c: Box; score: INTEGER;
    BEGIN
        NEW(a);
        NEW(b);
        c := a;
        score := 0;
        IF a # b THEN score := score + 10  END;     (* different objects *)
        IF a = c THEN score := score + 100 END;     (* alias to same object *)
        RETURN score
    END Run;
END M_Expr_PointerEquality_ReceivedFromCall.
