MODULE M_Expr_Bit_Style_Via_SET;
    IMPORT SYSTEM;

    PROCEDURE Run* (): INTEGER;
        VAR s: SET; n: INTEGER;
    BEGIN
        s := {};
        INCL(s, 1);   (* 0000_0010 = 2  *)
        INCL(s, 3);   (* + 0000_1000 = 10 *)
        INCL(s, 5);   (* + 0010_0000 = 42 *)
        n := SYSTEM.VAL(INTEGER, s);
        RETURN n      (* 2 + 8 + 32 = 42 *)
    END Run;
END M_Expr_Bit_Style_Via_SET.
