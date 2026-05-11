MODULE M_Method_TwoAliases_SameObject_SeeMutation;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) Bump* (n: INTEGER), NEW;
    BEGIN b.value := b.value + n END Bump;

    PROCEDURE Run* (): INTEGER;
        VAR p, q: Box;
    BEGIN
        NEW(p);
        p.value := 10;
        q := p;                       (* alias *)
        p.Bump(5);                    (* mutate via p *)
        q.Bump(7);                    (* mutate via q — same object *)
        RETURN q.value                (* 10 + 5 + 7 = 22 *)
    END Run;
END M_Method_TwoAliases_SameObject_SeeMutation.
