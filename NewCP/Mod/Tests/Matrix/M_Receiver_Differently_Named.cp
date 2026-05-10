MODULE M_Receiver_Differently_Named;
    TYPE Counter = RECORD value: INTEGER END;

    PROCEDURE (self: Counter) Read* (): INTEGER, NEW;
    BEGIN RETURN self.value END Read;

    PROCEDURE (VAR this: Counter) Set* (n: INTEGER), NEW;
    BEGIN this.value := n END Set;

    PROCEDURE Run* (): INTEGER;
        VAR c: Counter;
    BEGIN
        c.value := 0;
        c.Set(28);
        RETURN c.Read()
    END Run;
END M_Receiver_Differently_Named.
