MODULE M_Expr_Pointer_IS_Test;
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
