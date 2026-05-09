MODULE DynArray;
(* Probe for dynamic open-array support.
   Three things must work:
     1. `NEW(p, n)` allocates an n-element array
     2. `p[i] := v` indexes through the pointer
     3. `LEN(p^)` reports the dynamic length back

   None of these work today; this fixture is the regression test as
   each piece lands. *)

TYPE
    DigitArr = ARRAY OF SHORTINT;
    Bag = POINTER TO DigitArr;

PROCEDURE NewAndIndex* (): INTEGER;
    VAR b: Bag;
BEGIN
    NEW(b, 4);
    b[0] := 7;
    b[1] := 11;
    b[2] := 13;
    b[3] := 17;
    RETURN b[0] + b[1] + b[2] + b[3]    (* expect 48 *)
END NewAndIndex;

PROCEDURE Length* (): INTEGER;
    VAR b: Bag;
BEGIN
    NEW(b, 5);
    RETURN LEN(b^)                      (* expect 5 *)
END Length;

END DynArray.
