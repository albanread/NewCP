MODULE M_Stmt_Nested_CASE;
    PROCEDURE Classify (kind, sub: INTEGER): INTEGER;
    BEGIN
        CASE kind OF
          1:
            CASE sub OF
              10: RETURN 11
            | 20: RETURN 33
            ELSE  RETURN 19
            END
        | 2: RETURN 200
        ELSE  RETURN 999
        END
    END Classify;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN Classify(1, 20)
    END Run;
END M_Stmt_Nested_CASE.
