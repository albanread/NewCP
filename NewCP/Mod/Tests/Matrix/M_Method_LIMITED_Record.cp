MODULE M_Method_LIMITED_Record;
    TYPE
        BoxDesc* = LIMITED RECORD value*: INTEGER END;
        Box*     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) Set* (v: INTEGER), NEW;
    BEGIN b.value := v END Set;

    PROCEDURE Make* (v: INTEGER): Box;
        VAR b: Box;
    BEGIN
        NEW(b); b.Set(v); RETURN b
    END Make;

    PROCEDURE Run* (): INTEGER;
        VAR b: Box;
    BEGIN
        b := Make(99);
        RETURN b.value
    END Run;
END M_Method_LIMITED_Record.
