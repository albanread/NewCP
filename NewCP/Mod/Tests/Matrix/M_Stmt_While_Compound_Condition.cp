MODULE M_Stmt_While_Compound_Condition;
    TYPE
        NodeDesc = RECORD value: INTEGER; next: Node END;
        Node     = POINTER TO NodeDesc;

    PROCEDURE Run* (): INTEGER;
        VAR head, a, p: Node; sum: INTEGER;
    BEGIN
        NEW(head); head.value := 1;
        NEW(a);    a.value    := 2;
        head.next := a;
        a.next    := NIL;
        sum := 0;
        p := head;
        (* WHILE (p # NIL) & (p.value < 10) — the second conjunct
           must NOT be evaluated when p is NIL.  Without short-circuit
           the loop would crash on p = NIL. *)
        WHILE (p # NIL) & (p.value < 10) DO
            sum := sum + p.value * 2;
            p := p.next
        END;
        RETURN sum                            (* 2 + 4 = 6 *)
    END Run;
END M_Stmt_While_Compound_Condition.
