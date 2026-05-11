MODULE M_Method_Returns_AnyPtr;
    TYPE
        BoxDesc    = RECORD value: INTEGER END;
        Box        = POINTER TO BoxDesc;
        HolderDesc = RECORD END;
        Holder     = POINTER TO HolderDesc;

    PROCEDURE (h: Holder) Make* (): ANYPTR, NEW;
        VAR b: Box;
    BEGIN
        NEW(b); b.value := 77; RETURN b
    END Make;

    PROCEDURE Run* (): INTEGER;
        VAR h: Holder; ap: ANYPTR;
    BEGIN
        NEW(h);
        ap := h.Make();
        RETURN ap(Box).value
    END Run;
END M_Method_Returns_AnyPtr.
