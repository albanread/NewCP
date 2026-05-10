MODULE M_ProcType_IndirectCall;
    TYPE BinOp = PROCEDURE (a, b: INTEGER): INTEGER;

    PROCEDURE Mul (a, b: INTEGER): INTEGER;
    BEGIN RETURN a * b END Mul;

    PROCEDURE Apply (op: BinOp; x, y: INTEGER): INTEGER;
    BEGIN RETURN op(x, y) END Apply;

    PROCEDURE Run* (): INTEGER;
        VAR op: BinOp;
    BEGIN
        op := Mul;
        RETURN Apply(op, 7, 7)
    END Run;
END M_ProcType_IndirectCall.
