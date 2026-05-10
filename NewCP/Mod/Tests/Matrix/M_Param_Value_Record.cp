MODULE M_Param_Value_Record;
    TYPE Box = RECORD value*: INTEGER END;

    PROCEDURE Mutate (b: Box);
    BEGIN b.value := 999 END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR caller: Box;
    BEGIN
        caller.value := 42;
        Mutate(caller);
        RETURN caller.value
    END Run;
END M_Param_Value_Record.
