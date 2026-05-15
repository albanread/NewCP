MODULE HostMenus;
(*
   First slice of the BlackBox `HostMenus` port.

   BB's HostMenus is a ~2600-line Win32 wrapper — it owns the
   message loop, the menu bar, and the document-to-MDI-child
   binding.  Init.Init's first call is `HostMenus.OpenApp` and
   its last is `HostMenus.Run`; the entire framework lives
   inside that `Run` loop.

   Our story is simpler because iGui already runs the message
   loop.  HostMenus here is just two stubs — `OpenApp` (called
   before `Converters.Register`) and `Run` (called instead of
   the BB message loop).  Real menu management lands when iGui
   grows a menu surface and StdMenuTool can write into it.

   Deferred: every menu-management body.
*)


    (** Initialise the host app.  iGui's frame is started
        separately (see HelloPixels / run-igui mode), so this
        is a no-op. *)
    PROCEDURE OpenApp*;
    BEGIN
    END OpenApp;

    (** Enter the main event loop.  In BB this is where
        `PeekMessage` / `TranslateMessage` / `DispatchMessage`
        loop sits.  In our world iGui owns the message pump and
        runs CP code on a worker thread, so this returns
        immediately; the actual loop is the one iGui already
        started before Init ran. *)
    PROCEDURE Run*;
    BEGIN
    END Run;

    (** Install / refresh a menu — called by StdMenuTool.
        Stub: no-op until iGui's menu surface lands. *)
    PROCEDURE SetMenu* (IN title: ARRAY OF CHAR);
    BEGIN
    END SetMenu;

END HostMenus.
