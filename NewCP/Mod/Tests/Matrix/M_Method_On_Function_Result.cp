MODULE M_Method_On_Function_Result;
    TYPE
        BoxDesc = EXTENSIBLE RECORD v: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) Get* (): INTEGER, NEW;
    BEGIN RETURN b.v END Get;

    PROCEDURE Make (n: INTEGER): Box;
        VAR b: Box;
    BEGIN
        NEW(b);
        b.v := n;
        RETURN b
    END Make;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN Make(99).Get()
    END Run;
END M_Method_On_Function_Result.
