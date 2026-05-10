MODULE M_Builtin_COPY_FixedArray;
    PROCEDURE Run* (): INTEGER;
        VAR src, dst: ARRAY 3 OF INTEGER;
    BEGIN
        src[0] := 3; src[1] := 4; src[2] := 5;
        dst[0] := 0; dst[1] := 0; dst[2] := 0;
        dst := src;             (* whole-array assignment in CP *)
        RETURN dst[0] + dst[1] + dst[2]       (* 12 *)
    END Run;
END M_Builtin_COPY_FixedArray.
