MODULE WindowsProbe;
(* Smoke test for the BB-faithful Windows MVS slice.
   Just exercises the public surface — Window / Directory types
   compile, dir/stdDir start NIL, constants are reachable,
   SelectByTitle returns done = FALSE without trapping. *)

    IMPORT Windows;

    PROCEDURE Run* (): INTEGER;
        VAR done: BOOLEAN; flags: SET;
    BEGIN
        IF Windows.dir # NIL THEN RETURN -1 END;
        IF Windows.stdDir # NIL THEN RETURN -2 END;

        (* Constants are at compile-time positions; verify by
           assembling a flags set and asking IN. *)
        flags := {Windows.isTool, Windows.noResize, Windows.neverDirty};
        IF ~(Windows.isTool IN flags) THEN RETURN -3 END;
        IF Windows.isAux IN flags THEN RETURN -4 END;

        (* SelectByTitle's stub: returns done=FALSE without
           touching the (NIL) view argument or accessing dir. *)
        Windows.SelectByTitle(NIL, {}, "Welcome", done);
        IF done THEN RETURN -5 END;

        RETURN 1
    END Run;

END WindowsProbe.
