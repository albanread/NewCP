MODULE M_Param_OUT_OpenArray;
    PROCEDURE Fill (OUT p: ARRAY OF INTEGER);
        VAR i: INTEGER;
    BEGIN
        FOR i := 0 TO LEN(p) - 1 DO p[i] := (i + 1) * 10 END
    END Fill;

    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY 3 OF INTEGER;
    BEGIN
        Fill(a);
        RETURN a[0] + a[1] + a[2]           (* 10 + 20 + 30 = 60 *)
    END Run;
END M_Param_OUT_OpenArray.
