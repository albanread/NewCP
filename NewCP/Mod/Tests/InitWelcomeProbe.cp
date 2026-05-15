MODULE InitWelcomeProbe;
(* End-to-end smoke test for the BB UI startup chain.

   Runs `Init.Run` which:
     1. HostMenus.OpenApp                (no-op stub)
     2. Converters.Register("Documents.ImportDocument", ...)
     3. StdMenuTool.UpdateAllMenus       (no-op stub)
     4. Config.Setup                     (more Converters.Register +
                                          StdLog.Open)
     5. StdLog.Open                      (Console banner)
     6. HostMenus.Run                    (no-op stub)

   Then attempts to open the welcome page via
   `StdCmds.OpenToolDialog("System/Rsrc/About", "About BlackBox")`.
   The chain runs all the way down to Converters.Import (whose
   reflection dispatch is currently stubbed), so no window
   actually appears yet — but the framework walks every layer
   without trapping.

   Returns 1 on success, negative on first surprise. *)

    IMPORT
        Init, Windows, Converters, StdCmds;

    PROCEDURE Run* (): INTEGER;
        VAR count: INTEGER; first: BOOLEAN;
    BEGIN
        Init.Run;

        (* After Init.Run the StdDirectory should be installed
           and the .odc + .txt converters registered. *)
        IF Windows.dir = NIL THEN RETURN -1 END;

        (* Count converters via the in-module helper — direct
           Converters.list walks from outside trigger the
           unresolved cross-module dispatch hang. *)
        count := Converters.CountAndFirstType(first);
        IF count < 2 THEN RETURN -2 END;

        (* Try opening the welcome page.  The chain stops at
           Converters.Import (stub body, no actual file load),
           but every layer above runs without trapping. *)
        StdCmds.OpenToolDialog("System/Rsrc/About", "About BlackBox");

        RETURN 1
    END Run;

END InitWelcomeProbe.
