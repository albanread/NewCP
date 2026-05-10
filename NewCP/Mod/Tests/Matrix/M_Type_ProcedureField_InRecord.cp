MODULE M_Type_ProcedureField_InRecord;
    TYPE
        Op = PROCEDURE (x: INTEGER): INTEGER;
        DispatcherDesc = RECORD f: Op END;
        Dispatcher     = POINTER TO DispatcherDesc;

    PROCEDURE Square (x: INTEGER): INTEGER;
    BEGIN RETURN x * x END Square;

    PROCEDURE Run* (): INTEGER;
        VAR d: Dispatcher;
    BEGIN
        NEW(d);
        d.f := Square;
        RETURN d.f(7)                          (* 49 *)
    END Run;
END M_Type_ProcedureField_InRecord.
