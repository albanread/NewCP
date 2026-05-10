MODULE M_Param_VAR_Pointer;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Replace (VAR b: Box);
        VAR fresh: Box;
    BEGIN
        NEW(fresh);
        fresh.value := 99;
        b := fresh
    END Replace;

    PROCEDURE Run* (): INTEGER;
        VAR orig: Box;
    BEGIN
        NEW(orig);
        orig.value := 1;
        Replace(orig);
        RETURN orig.value      (* 99 if the new pointer landed in the caller's slot *)
    END Run;
END M_Param_VAR_Pointer.
