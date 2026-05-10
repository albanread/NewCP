MODULE ValueRecordParamProbe;
(* CP §8.1: a value-mode record parameter is a private copy.  Mutate's
   write to b.value must not propagate back to the caller. *)

    TYPE
        Box = RECORD value*: INTEGER END;

    PROCEDURE Mutate (b: Box);
    BEGIN
        b.value := 999
    END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR caller: Box;
    BEGIN
        caller.value := 42;
        Mutate(caller);
        RETURN caller.value      (* CP says 42 *)
    END Run;

END ValueRecordParamProbe.
