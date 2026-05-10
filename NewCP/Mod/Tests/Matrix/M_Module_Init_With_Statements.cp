MODULE M_Module_Init_With_Statements;
    VAR a, b, c: INTEGER;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN a + b + c                        (* 10 + 20 + 30 = 60 *)
    END Run;

BEGIN
    a := 10;
    b := 20;
    c := 30
END M_Module_Init_With_Statements.
