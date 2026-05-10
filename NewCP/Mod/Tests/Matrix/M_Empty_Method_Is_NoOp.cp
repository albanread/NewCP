MODULE M_Empty_Method_Is_NoOp;
    TYPE
        BaseDesc* = EXTENSIBLE RECORD value*: INTEGER END;
        Base*     = POINTER TO BaseDesc;

    PROCEDURE (b: Base) Visit* (), NEW, EMPTY;

    PROCEDURE Run* (): INTEGER;
        VAR b: Base;
    BEGIN
        NEW(b);
        b.value := 5;
        b.Visit();             (* no-op; value untouched *)
        RETURN b.value
    END Run;
END M_Empty_Method_Is_NoOp.
