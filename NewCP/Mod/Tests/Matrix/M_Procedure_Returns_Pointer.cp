MODULE M_Procedure_Returns_Pointer;
    TYPE
        BoxDesc = RECORD v: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Make (n: INTEGER): Box;
        VAR b: Box;
    BEGIN
        NEW(b);
        b.v := n;
        RETURN b
    END Make;

    PROCEDURE Run* (): INTEGER;
        VAR b: Box;
    BEGIN
        b := Make(500);
        b.v := b.v + 500;       (* mutation through the returned pointer *)
        RETURN b.v
    END Run;
END M_Procedure_Returns_Pointer.
