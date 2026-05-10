MODULE M_Stmt_Procedure_Recursion;
    PROCEDURE Fact (n: INTEGER): INTEGER;
    BEGIN
        IF n <= 1 THEN RETURN 1 END;
        RETURN n * Fact(n - 1)
    END Fact;

    PROCEDURE Run* (): INTEGER;
    BEGIN RETURN Fact(6)                    (* 720 *)
    END Run;
END M_Stmt_Procedure_Recursion.
