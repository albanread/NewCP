MODULE M_Expr_Relational_Pointer_NIL;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Run* (): INTEGER;
        VAR p, q, r: Box; score: INTEGER;
    BEGIN
        p := NIL;
        NEW(q); q.value := 42;
        r := q;
        score := 0;
        IF p = NIL THEN score := score + 10    END;
        IF q # NIL THEN score := score + 100   END;
        IF q = r   THEN score := score + 1000  END;   (* same heap object *)
        IF p # q   THEN score := score + 10000 END;
        RETURN score                          (* 11110 *)
    END Run;
END M_Expr_Relational_Pointer_NIL.
