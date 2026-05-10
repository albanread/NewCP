MODULE M_Type_Array_Of_Records;
    TYPE Point = RECORD x, y: INTEGER END;

    PROCEDURE Run* (): INTEGER;
        VAR pts: ARRAY 3 OF Point; i, sum: INTEGER;
    BEGIN
        FOR i := 0 TO 2 DO
            pts[i].x := (i + 1) * 10;
            pts[i].y := i + 1
        END;
        sum := 0;
        FOR i := 0 TO 2 DO sum := sum + pts[i].x END;
        RETURN sum                              (* 10 + 20 + 30 = 60 *)
    END Run;
END M_Type_Array_Of_Records.
