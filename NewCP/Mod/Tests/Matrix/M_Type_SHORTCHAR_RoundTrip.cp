MODULE M_Type_SHORTCHAR_RoundTrip;
    PROCEDURE Run* (): INTEGER;
        VAR c: SHORTCHAR;
    BEGIN
        c := SHORT(CHR(88));          (* CHR returns CHAR; SHORT narrows to SHORTCHAR *)
        RETURN ORD(c)                 (* 88 *)
    END Run;
END M_Type_SHORTCHAR_RoundTrip.
