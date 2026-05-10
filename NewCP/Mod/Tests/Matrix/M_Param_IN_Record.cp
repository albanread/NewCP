MODULE M_Param_IN_Record;
    TYPE Box = RECORD value: INTEGER END;

    PROCEDURE Peek (IN b: Box): INTEGER;
    BEGIN RETURN b.value END Peek;

    PROCEDURE Run* (): INTEGER;
        VAR caller: Box;
    BEGIN
        caller.value := 77;
        RETURN Peek(caller)
    END Run;
END M_Param_IN_Record.
