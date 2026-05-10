MODULE ValueFixedArrayProbe;
(* CP §8.1: a value-mode fixed-array parameter is a private copy.
   Mutate's write to a[0] must not propagate back to the caller. *)

    PROCEDURE Mutate (a: ARRAY 4 OF INTEGER);
    BEGIN
        a[0] := 999
    END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR caller: ARRAY 4 OF INTEGER;
    BEGIN
        caller[0] := 42;
        Mutate(caller);
        RETURN caller[0]      (* CP says 42 *)
    END Run;

END ValueFixedArrayProbe.
