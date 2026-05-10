MODULE M_Stmt_Empty_If_Arm;
    PROCEDURE Run* (): INTEGER;
        VAR n: INTEGER;
    BEGIN
        n := 5;
        IF n < 0 THEN
            (* empty arm — semantic no-op *)
        ELSE
            n := n
        END;
        RETURN n
    END Run;
END M_Stmt_Empty_If_Arm.
