MODULE M_Param_OUT_Record;
    TYPE Box = RECORD value*: INTEGER END;

    PROCEDURE Init (OUT b: Box);
    BEGIN b.value := 7 END Init;

    PROCEDURE Run* (): INTEGER;
        VAR caller: Box;
    BEGIN
        caller.value := 100;
        Init(caller);
        RETURN caller.value
    END Run;
END M_Param_OUT_Record.
