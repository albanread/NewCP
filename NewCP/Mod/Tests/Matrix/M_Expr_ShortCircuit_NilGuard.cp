MODULE M_Expr_ShortCircuit_NilGuard;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Probe (p: Box): INTEGER;
    BEGIN
        IF (p # NIL) & (p.value > 0) THEN
            RETURN p.value
        ELSE
            RETURN 42
        END
    END Probe;

    PROCEDURE Run* (): INTEGER;
        VAR nilBox: Box;
    BEGIN
        nilBox := NIL;
        (* p is NIL — second conjunct must NOT execute *)
        RETURN Probe(nilBox)
    END Run;
END M_Expr_ShortCircuit_NilGuard.
