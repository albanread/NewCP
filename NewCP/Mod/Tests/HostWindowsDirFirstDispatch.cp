MODULE HostWindowsDirFirstDispatch;

    IMPORT Windows, HostWindows;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        IF Windows.dir = NIL THEN RETURN -1 END;
        IF Windows.stdDir = NIL THEN RETURN -2 END;
        IF Windows.dir.First() # NIL THEN RETURN -3 END;
        RETURN 1
    END Run;

END HostWindowsDirFirstDispatch.