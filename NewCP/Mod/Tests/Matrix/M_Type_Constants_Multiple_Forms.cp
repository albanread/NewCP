MODULE M_Type_Constants_Multiple_Forms;
    CONST
        n = 65;
        b = TRUE;
        c = "A";
        r = 1.0;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        IF b & (c = "A") & (ENTIER(r) = 1) THEN
            RETURN n
        ELSE
            RETURN 0
        END
    END Run;
END M_Type_Constants_Multiple_Forms.
