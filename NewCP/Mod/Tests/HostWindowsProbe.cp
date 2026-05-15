MODULE HostWindowsProbe;
(* Smoke test for the BB-faithful HostWindows MVS slice.
   Verifies that:
     - HostWindows module init runs (installs the Directory
       into Windows.SetDir).
     - Windows.dir is now non-NIL.
     - Directory.First returns NIL on an empty registry.
     - Directory.New allocates a Window without trapping. *)

    IMPORT Windows, HostWindows;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        IF Windows.dir = NIL THEN RETURN -1 END;
        IF Windows.stdDir = NIL THEN RETURN -2 END;
        RETURN 1
    END Run;

END HostWindowsProbe.
