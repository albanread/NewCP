MODULE M_Type_SHORTINT_Arithmetic;
    PROCEDURE Run* (): INTEGER;
        VAR s: SHORTINT;
    BEGIN
        s := 100;
        s := SHORT(s + s);       (* 200 fits in SHORTINT *)
        RETURN s
    END Run;
END M_Type_SHORTINT_Arithmetic.
