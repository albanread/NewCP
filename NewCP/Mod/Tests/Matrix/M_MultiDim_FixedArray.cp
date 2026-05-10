MODULE M_MultiDim_FixedArray;
    PROCEDURE Run* (): INTEGER;
        VAR grid: ARRAY 3, 3 OF INTEGER; i, j, sum: INTEGER;
    BEGIN
        FOR i := 0 TO 2 DO
            FOR j := 0 TO 2 DO
                grid[i, j] := (i + 1) * 10 + j
            END
        END;
        sum := 0;
        FOR i := 0 TO 2 DO
            FOR j := 0 TO 2 DO
                sum := sum + grid[i, j]
            END
        END;
        (* Values: row0 = 10,11,12; row1 = 20,21,22; row2 = 30,31,32
           sum = 33 + 63 + 93 = 189; +61 = 250 *)
        RETURN sum + 61
    END Run;
END M_MultiDim_FixedArray.
