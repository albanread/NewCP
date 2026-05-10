MODULE M_Method_Recursive;
    TYPE
        WrapperDesc = RECORD END;
        Wrapper     = POINTER TO WrapperDesc;

    PROCEDURE (w: Wrapper) Fib* (n: INTEGER): INTEGER, NEW;
    BEGIN
        IF n < 2 THEN RETURN n END;
        RETURN w.Fib(n - 1) + w.Fib(n - 2)
    END Fib;

    PROCEDURE Run* (): INTEGER;
        VAR w: Wrapper;
    BEGIN
        NEW(w);
        RETURN w.Fib(8)                     (* fib(8) = 21 *)
    END Run;
END M_Method_Recursive.
