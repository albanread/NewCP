MODULE Loops;
(* Exercises REPEAT/UNTIL and LOOP/EXIT control flow. *)

(* REPEAT/UNTIL: count down from n to 1, return sum. *)
PROCEDURE SumDown*(n: INTEGER): INTEGER;
    VAR s: INTEGER;
BEGIN
    s := 0;
    REPEAT
        INC(s, n);
        DEC(n)
    UNTIL n < 1;
    RETURN s
END SumDown;

(* REPEAT/UNTIL: count set bits in x using bit-shifting. *)
PROCEDURE PopCount*(x: INTEGER): INTEGER;
    VAR count: INTEGER;
BEGIN
    count := 0;
    REPEAT
        IF ODD(x) THEN INC(count) END;
        x := ASH(x, -1)
    UNTIL x = 0;
    RETURN count
END PopCount;

(* LOOP/EXIT: find first occurrence of v in array a of length len.
   Returns index, or -1 if not found. *)
PROCEDURE IndexOf*(v, len: INTEGER): INTEGER;
    VAR i: INTEGER;
BEGIN
    i := 0;
    LOOP
        IF i >= len THEN EXIT END;
        IF i = v THEN EXIT END;
        INC(i)
    END;
    IF i >= len THEN RETURN -1
    ELSE RETURN i
    END
END IndexOf;

(* LOOP/EXIT: collatz sequence length starting from n. *)
PROCEDURE CollatzLen*(n: INTEGER): INTEGER;
    VAR steps: INTEGER;
BEGIN
    steps := 0;
    LOOP
        IF n = 1 THEN EXIT END;
        IF ODD(n) THEN
            n := 3 * n + 1
        ELSE
            n := ASH(n, -1)
        END;
        INC(steps)
    END;
    RETURN steps
END CollatzLen;

BEGIN
END Loops.
