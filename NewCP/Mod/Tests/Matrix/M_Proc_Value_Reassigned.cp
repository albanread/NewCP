MODULE M_Proc_Value_Reassigned;
    TYPE UnaryOp = PROCEDURE (x: INTEGER): INTEGER;

    PROCEDURE Triple (x: INTEGER): INTEGER;
    BEGIN RETURN x * 3 END Triple;

    PROCEDURE AddTen (x: INTEGER): INTEGER;
    BEGIN RETURN x + 10 END AddTen;

    PROCEDURE Run* (): INTEGER;
        VAR f: UnaryOp; a, b: INTEGER;
    BEGIN
        f := Triple;
        a := f(6);          (* 18 *)
        f := AddTen;
        b := f(0);          (* 10 *)
        RETURN a + b        (* 28 *)
    END Run;
END M_Proc_Value_Reassigned.
