MODULE M_Receiver_Value_And_VAR_Coexist;
    TYPE Counter = RECORD n: INTEGER END;

    PROCEDURE (c: Counter) Read* (): INTEGER, NEW;
    BEGIN RETURN c.n END Read;

    PROCEDURE (VAR c: Counter) Add* (k: INTEGER), NEW;
    BEGIN c.n := c.n + k END Add;

    PROCEDURE Run* (): INTEGER;
        VAR c: Counter;
    BEGIN
        c.n := 0;
        c.Add(20);
        c.Add(30);
        RETURN c.Read()
    END Run;
END M_Receiver_Value_And_VAR_Coexist.
