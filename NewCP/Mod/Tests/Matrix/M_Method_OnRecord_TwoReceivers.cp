MODULE M_Method_OnRecord_TwoReceivers;
    TYPE Tally = RECORD running: INTEGER END;

    PROCEDURE (VAR t: Tally) Add* (n: INTEGER), NEW;
    BEGIN t.running := t.running + n END Add;

    PROCEDURE (t: Tally) Snapshot* (): INTEGER, NEW;
    BEGIN RETURN t.running END Snapshot;

    PROCEDURE Run* (): INTEGER;
        VAR t: Tally;
    BEGIN
        t.running := 0;
        t.Add(40);
        t.Add(44);
        RETURN t.Snapshot()
    END Run;
END M_Method_OnRecord_TwoReceivers.
