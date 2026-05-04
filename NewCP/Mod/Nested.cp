MODULE Nested;

(* Nested procedure with no upvalue access (pure params). *)
PROCEDURE Outer*(n: INTEGER): INTEGER;

    PROCEDURE Double(v: INTEGER): INTEGER;
    BEGIN
        RETURN v * 2
    END Double;

BEGIN
    RETURN Double(n)
END Outer;

(* Nested procedure reading an outer local variable (upvalue read). *)
PROCEDURE WithCapture*(base: INTEGER): INTEGER;
    VAR offset: INTEGER;

    PROCEDURE Add(n: INTEGER): INTEGER;
    BEGIN
        RETURN offset + n
    END Add;

BEGIN
    offset := base * 3;
    RETURN Add(10)
END WithCapture;

(* Nested procedure writing to an outer local (upvalue write). *)
PROCEDURE WithMutation*(n: INTEGER): INTEGER;
    VAR accum: INTEGER;

    PROCEDURE Accumulate(v: INTEGER);
    BEGIN
        accum := accum + v
    END Accumulate;

BEGIN
    accum := 0;
    Accumulate(n);
    Accumulate(n * 2);
    RETURN accum
END WithMutation;

BEGIN
END Nested.
