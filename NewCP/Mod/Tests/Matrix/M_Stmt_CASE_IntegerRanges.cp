MODULE M_Stmt_CASE_IntegerRanges;
    PROCEDURE Bucket (n: INTEGER): INTEGER;
    BEGIN
        CASE n OF
          0:        RETURN 100
        | 1..5:     RETURN 200 + n
        | 7, 9, 11: RETURN 300 + n
        ELSE        RETURN 999
        END
    END Bucket;

    PROCEDURE Run* (): INTEGER;
        VAR score: INTEGER;
    BEGIN
        score := 0;
        IF Bucket(0)  = 100 THEN score := score + 1   END;
        IF Bucket(3)  = 203 THEN score := score + 5   END;
        IF Bucket(9)  = 309 THEN score := score + 40  END;
        IF Bucket(99) = 999 THEN score := score + 200 END;
        RETURN score
    END Run;
END M_Stmt_CASE_IntegerRanges.
