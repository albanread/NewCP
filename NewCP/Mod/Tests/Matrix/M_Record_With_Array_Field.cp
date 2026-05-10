MODULE M_Record_With_Array_Field;
    TYPE Vec3 = RECORD elems: ARRAY 3 OF INTEGER END;

    PROCEDURE Run* (): INTEGER;
        VAR v: Vec3; i, sum: INTEGER;
    BEGIN
        v.elems[0] := 10;
        v.elems[1] := 20;
        v.elems[2] := 30;
        sum := 0;
        FOR i := 0 TO 2 DO sum := sum + v.elems[i] END;
        RETURN sum                              (* 60 *)
    END Run;
END M_Record_With_Array_Field.
