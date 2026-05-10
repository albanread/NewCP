MODULE M_OpenArray_Of_CHAR;
    PROCEDURE Sum (IN s: ARRAY OF CHAR): INTEGER;
        VAR i, total: INTEGER;
    BEGIN
        i := 0; total := 0;
        WHILE (i < LEN(s)) & (s[i] # 0X) DO
            total := total + ORD(s[i]);
            INC(i)
        END;
        RETURN total
    END Sum;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        (* "ABC" → ORD('A')+ORD('B')+ORD('C') = 65+66+67 = 198; plus
           a length marker (97 = "a") so the sum has to include the
           trailing char before the NUL. *)
        RETURN Sum("ABCa")
    END Run;
END M_OpenArray_Of_CHAR.
