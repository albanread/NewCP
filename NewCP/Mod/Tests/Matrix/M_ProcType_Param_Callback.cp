MODULE M_ProcType_Param_Callback;
    TYPE Unary = PROCEDURE (x: INTEGER): INTEGER;

    PROCEDURE Square (x: INTEGER): INTEGER;
    BEGIN RETURN x * x END Square;

    PROCEDURE ApplyTwice (seed: INTEGER; f: Unary): INTEGER;
        VAR once: INTEGER;
    BEGIN
        once := f(seed);
        RETURN f(once)              (* Square(Square(seed)) via a temp *)
    END ApplyTwice;

    PROCEDURE Run* (): INTEGER;
        VAR cb: Unary;
    BEGIN
        cb := Square;
        (* ApplyTwice(Square, 3) = Square(Square(3)) = Square(9) = 81;
           plus a marker constant 40 so a stub returning 0 fails fast *)
        RETURN ApplyTwice(3, cb) + 40
    END Run;
END M_ProcType_Param_Callback.
