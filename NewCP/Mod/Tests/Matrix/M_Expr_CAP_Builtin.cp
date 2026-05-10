MODULE M_Expr_CAP_Builtin;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, c: CHAR;
    BEGIN
        a := CAP("a");      (* "A" = 65 *)
        b := CAP("Z");      (* still "Z" = 90 *)
        c := CAP("0");      (* still "0" = 48 *)
        (* Pack: ORD(a) - 64 = 1; ORD(b) - 90 = 0; ORD(c) - 48 = 0
           Combine 1 + 0 + 0 + 66 = 67 *)
        RETURN (ORD(a) - 64) + (ORD(b) - 90) + (ORD(c) - 48) + 66
    END Run;
END M_Expr_CAP_Builtin.
