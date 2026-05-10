MODULE M_Type_PointerToOpenArray_NEW;
    TYPE IntVec = POINTER TO ARRAY OF INTEGER;

    PROCEDURE Run* (): INTEGER;
        VAR p: IntVec; i, sum: INTEGER;
    BEGIN
        NEW(p, 4);
        FOR i := 0 TO LEN(p^) - 1 DO p[i] := (i + 1) * 3 END;
        sum := 0;
        FOR i := 0 TO LEN(p^) - 1 DO sum := sum + p[i] END;
        RETURN sum                            (* 3 + 6 + 9 + 12 = 30 *)
    END Run;
END M_Type_PointerToOpenArray_NEW.
