MODULE M_Type_Const_In_ArraySize;
    CONST size = 4;

    PROCEDURE Run* (): INTEGER;
        VAR arr: ARRAY size OF INTEGER;
    BEGIN
        arr[0] := 0; arr[1] := 0; arr[2] := 0; arr[3] := 0;
        RETURN LEN(arr)              (* 4 *)
    END Run;
END M_Type_Const_In_ArraySize.
