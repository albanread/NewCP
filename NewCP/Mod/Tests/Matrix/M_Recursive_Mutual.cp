MODULE M_Recursive_Mutual;
    PROCEDURE IsOdd  (n: INTEGER): BOOLEAN;
    BEGIN
        IF n = 0 THEN RETURN FALSE
        ELSE RETURN IsEven(n - 1)
        END
    END IsOdd;

    PROCEDURE IsEven (n: INTEGER): BOOLEAN;
    BEGIN
        IF n = 0 THEN RETURN TRUE
        ELSE RETURN IsOdd(n - 1)
        END
    END IsEven;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        IF IsEven(10) & IsOdd(7) THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Recursive_Mutual.
