MODULE M_VAR_Receiver_Mutates_Record;
    TYPE Counter = RECORD value: INTEGER END;

    PROCEDURE (VAR c: Counter) SetAndDouble* (n: INTEGER), NEW;
    BEGIN
        c.value := n;
        c.value := c.value * 2
    END SetAndDouble;

    PROCEDURE Run* (): INTEGER;
        VAR c: Counter;
    BEGIN
        c.value := 0;
        c.SetAndDouble(44);
        RETURN c.value           (* 88 *)
    END Run;
END M_VAR_Receiver_Mutates_Record.
