MODULE StdMenuTool;
(*
   First slice of the BlackBox `StdMenuTool` port.

   BB's StdMenuTool reads the "Menus" tool document at startup
   and pushes its commands into HostMenus to populate the host's
   menu bar.  Init.Init triggers it via
   `Dialog.Call("StdMenuTool.UpdateAllMenus", "", res)`.

   This slice ships only the public surface (`UpdateAllMenus` +
   the small constellation of update / list helpers).  Bodies
   are no-ops; the welcome-page chain doesn't need a populated
   menu bar to function (StdCmds.OpenToolDialog can be called
   directly from probes).

   Deferred: every body — port alongside HostMenus and the
   "Menus" tool-document parser.
*)


    (** Re-parse the "Menus" tool document and republish.
        Stub: no-op. *)
    PROCEDURE UpdateAllMenus*;
    BEGIN
    END UpdateAllMenus;

    (** Refresh the menu bar without re-reading the source.
        Stub: no-op. *)
    PROCEDURE UpdateMenus*;
    BEGIN
    END UpdateMenus;

    (** List all menus on StdLog.  Stub: no-op. *)
    PROCEDURE ListAllMenus*;
    BEGIN
    END ListAllMenus;

    (** Look up the menu owning the focus.  Stub: no-op. *)
    PROCEDURE ThisMenu*;
    BEGIN
    END ThisMenu;

END StdMenuTool.
