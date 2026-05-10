MODULE M_Builtin_ASH_Shifts;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: INTEGER;
    BEGIN
        a := ASH(1, 5);      (* 1 << 5 = 32 *)
        b := ASH(64, -1);    (* 64 >> 1 = 32 *)
        IF a = b THEN RETURN a ELSE RETURN -1 END
    END Run;
END M_Builtin_ASH_Shifts.
