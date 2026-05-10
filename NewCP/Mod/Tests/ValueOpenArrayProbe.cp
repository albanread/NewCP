MODULE ValueOpenArrayProbe;
(* Probe: a value-mode open array parameter `p: ARRAY OF INTEGER`
   should — per CP §8.1 — be a private copy.  Mutations through `p`
   inside the callee must not leak back to the caller's array.

   Ports.DrawPath relies on this exact idiom (inner `Draw(p)` mutates
   a local copy).  If NewCP's codegen passes the open array by
   reference even in value mode, this probe will return 99 instead
   of the expected 7. *)

    PROCEDURE Mutate (p: ARRAY OF INTEGER);
    BEGIN
        p[0] := 99
    END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY 4 OF INTEGER;
    BEGIN
        a[0] := 7;
        Mutate(a);
        RETURN a[0]      (* CP says 7; if alias-by-ref, returns 99 *)
    END Run;

END ValueOpenArrayProbe.
