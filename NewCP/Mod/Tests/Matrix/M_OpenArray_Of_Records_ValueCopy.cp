MODULE M_OpenArray_Of_Records_ValueCopy;
    TYPE Point = RECORD x*, y*: INTEGER END;

    PROCEDURE Mutate (a: ARRAY OF Point);
    BEGIN
        a[0].x := 999; a[0].y := 999;
        a[1].x := 999; a[1].y := 999
    END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY 2 OF Point;
    BEGIN
        a[0].x := 10; a[0].y := 11;
        a[1].x := 14; a[1].y := 15;
        Mutate(a);
        RETURN a[0].x + a[0].y + a[1].x + a[1].y    (* 10+11+14+15 = 50 *)
    END Run;
END M_OpenArray_Of_Records_ValueCopy.
