MODULE M_RecvValueStyle_NewOnPlain;
    TYPE
        Counter = RECORD value: INTEGER END;

    PROCEDURE (VAR c: Counter) Bump* (n: INTEGER), NEW;
    BEGIN c.value := c.value + n END Bump;

    PROCEDURE (c: Counter) Read* (): INTEGER, NEW;
    BEGIN RETURN c.value END Read;

    PROCEDURE Run* (): INTEGER;
        VAR c: Counter;
    BEGIN
        c.value := 0;
        c.Bump(42);
        c.Bump(4200);
        RETURN c.Read()
    END Run;
END M_RecvValueStyle_NewOnPlain.
