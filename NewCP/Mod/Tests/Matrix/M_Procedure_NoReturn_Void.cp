MODULE M_Procedure_NoReturn_Void;
    VAR result: INTEGER;

    PROCEDURE SetResult (n: INTEGER);
    BEGIN result := n END SetResult;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        result := 0;
        SetResult(99);
        RETURN result
    END Run;
END M_Procedure_NoReturn_Void.
