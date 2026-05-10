MODULE M_Method_On_ArrayElement;
    TYPE
        ItemDesc = EXTENSIBLE RECORD v: INTEGER END;
        Item     = POINTER TO ItemDesc;

    PROCEDURE (i: Item) Treble* (): INTEGER, NEW;
    BEGIN RETURN i.v * 3 END Treble;

    PROCEDURE Run* (): INTEGER;
        VAR arr: ARRAY 3 OF Item;
    BEGIN
        NEW(arr[0]); arr[0].v := 5;
        NEW(arr[1]); arr[1].v := 7;
        NEW(arr[2]); arr[2].v := 9;
        RETURN arr[2].Treble()      (* 9 * 3 = 27 *)
    END Run;
END M_Method_On_ArrayElement.
