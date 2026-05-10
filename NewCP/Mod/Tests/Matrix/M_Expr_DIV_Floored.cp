MODULE M_Expr_DIV_Floored;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, c, d: INTEGER;
    BEGIN
        a :=    7  DIV   3;     (*  2 *)
        b := (-7) DIV   3;      (* -3 (C would say -2) *)
        c :=    7  DIV (-3);    (* -3 *)
        d := (-7) DIV (-3);     (*  2 *)
        (* pack into one int: a*1000 + (b+10)*100 + (c+10)*10 + (d+10)
           = 2000 + 7*100 + 7*10 + 12 = 2782; offset by 7222 to land on a stable signature *)
        RETURN a * 1000 + (b + 10) * 100 + (c + 10) * 10 + (d + 10) + 7222
    END Run;
END M_Expr_DIV_Floored.
