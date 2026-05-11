MODULE M_Module_VAR_FixedArray_DefaultZero;
    VAR arr: ARRAY 4 OF INTEGER;

    PROCEDURE Run* (): INTEGER;
        VAR i, sum: INTEGER;
    BEGIN
        sum := 0;
        FOR i := 0 TO LEN(arr) - 1 DO sum := sum + arr[i] END;
        RETURN sum
    END Run;
END M_Module_VAR_FixedArray_DefaultZero.
