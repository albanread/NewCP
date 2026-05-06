MODULE Procs;
(* Stand-alone procedures: arithmetic, comparisons, loops. *)

PROCEDURE Add*(a, b: INTEGER): INTEGER;
BEGIN
    RETURN a + b
END Add;

PROCEDURE Sub*(a, b: INTEGER): INTEGER;
BEGIN
    RETURN a - b
END Sub;

PROCEDURE Max*(a, b: INTEGER): INTEGER;
BEGIN
    IF a > b THEN RETURN a
    ELSE RETURN b
    END
END Max;

PROCEDURE Clamp*(x, lo, hi: INTEGER): INTEGER;
BEGIN
    IF x < lo THEN RETURN lo
    ELSIF x > hi THEN RETURN hi
    ELSE RETURN x
    END
END Clamp;

PROCEDURE SumTo*(n: INTEGER): INTEGER;
    VAR i, s: INTEGER;
BEGIN
    s := 0;
    FOR i := 1 TO n DO
        INC(s, i)
    END;
    RETURN s
END SumTo;

PROCEDURE Factorial*(n: INTEGER): INTEGER;
    VAR result: INTEGER;
BEGIN
    result := 1;
    WHILE n > 1 DO
        result := result * n;
        DEC(n)
    END;
    RETURN result
END Factorial;

BEGIN
END Procs.
