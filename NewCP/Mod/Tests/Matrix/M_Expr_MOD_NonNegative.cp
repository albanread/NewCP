MODULE M_Expr_MOD_NonNegative;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: INTEGER;
    BEGIN
        a :=    7  MOD 3;     (* 1 *)
        b := (-7) MOD 3;      (* 2 — C would say -1 *)
        RETURN a * 1000 + b * 100 + 12      (* 1000 + 200 + 12 = 1212 *)
    END Run;
END M_Expr_MOD_NonNegative.
