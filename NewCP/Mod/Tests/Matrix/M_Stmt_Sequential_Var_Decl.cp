MODULE M_Stmt_Sequential_Var_Decl;
    PROCEDURE Run* (): INTEGER;
        VAR a: INTEGER;
        VAR b: INTEGER;
        VAR c, d: INTEGER;
    BEGIN
        a := 1; b := 2; c := 3; d := 4;
        RETURN a * 1000 + b * 100 + c * 10 + d
    END Run;
END M_Stmt_Sequential_Var_Decl.
