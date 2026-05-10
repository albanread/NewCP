MODULE M_Stmt_IF_ElsIf_Chain;
    PROCEDURE Pick (n: INTEGER): INTEGER;
    BEGIN
        IF n < 0 THEN RETURN -1
        ELSIF n = 0 THEN RETURN 0
        ELSIF n < 10 THEN RETURN n * 10
        ELSIF n < 100 THEN RETURN n + 100
        ELSE RETURN 999
        END
    END Pick;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        (* Pick(-3)=-1, Pick(0)=0, Pick(5)=50, Pick(7)=70, Pick(15)=115 → sum = 234;
           offset to land on a stable signature *)
        RETURN Pick(-3) + Pick(0) + Pick(5) + Pick(7) + Pick(15) - 201
    END Run;
END M_Stmt_IF_ElsIf_Chain.
