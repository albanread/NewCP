MODULE BbInit;
(*
   First slice of the BlackBox `Init` port.

   BB's `Init` is the startup orchestrator — its body runs at
   load time and walks:
     1. HostMenus.OpenApp                (host app shell)
     2. Kernel.ThisMod/LoadMod("StdDebug")
     3. Converters.Register("Documents.ImportDocument", ...,
                            ".odc", {})
     4. Dialog.Call("StdMenuTool.UpdateAllMenus", "", res)
     5. Kernel.LoadMod("OleServer")
     6. Dialog.Call("Config.Setup", "", res)
     7. HostMenus.Run                    (main event loop)

   This slice ships a callable `Run` proc that walks the same
   sequence with direct procedure calls instead of the Meta-
   reflection-dispatched `Dialog.Call` bounce — our
   Meta.LookupPath is still a surface stub.  Once the
   reflection lands, the BB-faithful body restores by swapping
   the direct calls back to `Dialog.Call`.

   Module body is empty by design — Init.Run is called
   explicitly by the user-facing entry point (e.g. a probe or
   the run-igui driver).  Implicit module-init invocation
   ordering with HostMenus / HostWindows / StdLog all running
   their own BEGIN bodies in load order means a module-body-
   driven Init would arrive too early.
*)

    IMPORT
        Converters,
        HostMenus, HostWindows,
        Config, StdMenuTool, StdLog;


    (** Walk the BB startup sequence with direct calls. *)
    PROCEDURE Run*;
    BEGIN
        HostMenus.OpenApp;

        (* HostWindows' module body has already run by the time
           Init.Run is called — it installed the StdDirectory
           into Windows.dir.  The import statement above is what
           keeps that side-effect anchored. *)

        (* BB-faithful: register the ODC handler before
           StdMenuTool reads its menu document.  We call
           Converters.Register directly rather than going
           through Dialog.Call("Config.Setup"). *)
        Converters.Register("Documents.ImportDocument",
                            "Documents.ExportDocument",
                            "",
                            "odc",
                            {});

        (* Pull menu definitions in from the "Menus" tool doc.
           Stub today, but the call site is BB-faithful. *)
        StdMenuTool.UpdateAllMenus;

        (* Wire up the rest of Config.Setup — registers more
           converters + opens the system log. *)
        Config.Setup;

        (* Hand off to the host event loop.  Stub today
           (iGui's pump is already running); BB's body never
           returns. *)
        StdLog.Open;
        HostMenus.Run
    END Run;

END BbInit.
