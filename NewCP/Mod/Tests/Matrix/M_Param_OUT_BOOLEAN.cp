MODULE M_Param_OUT_BOOLEAN;
    PROCEDURE SetTrue (OUT b: BOOLEAN);
    BEGIN b := TRUE END SetTrue;

    PROCEDURE Run* (): INTEGER;
        VAR flag: BOOLEAN;
    BEGIN
        flag := FALSE;
        SetTrue(flag);
        IF flag THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Param_OUT_BOOLEAN.
