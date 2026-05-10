MODULE M_Stmt_RETURN_Many_Paths;
    PROCEDURE Classify (n: INTEGER): INTEGER;
    BEGIN
        IF n < 0 THEN RETURN -1 END;
        IF n = 0 THEN RETURN 0 END;
        IF n > 100 THEN RETURN 999 END;
        RETURN n
    END Classify;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN Classify(30)                   (* 30 *)
    END Run;
END M_Stmt_RETURN_Many_Paths.
