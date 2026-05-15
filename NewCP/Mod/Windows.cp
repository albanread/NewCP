MODULE Windows;
(*
   First slice of the BlackBox `Windows` port.

   BB's `Windows` (1530 lines) is the window-manager and
   document-binding layer.  `Window` wraps a Documents.Document
   in a Ports.Port-backed frame tree; `Directory` is the host-
   integration factory (one concrete subclass per platform — for
   us that will be HostWindows, riding on iGui).  StdCmds and
   StdApi route every "open / close / focus" through the
   exported `dir` global.

   What's in THIS slice:
   - The full surface — types (Window, Directory, Context,
     Message), constants (isTool/isAux/noHScroll/...),
     exported VAR `dir`/`stdDir`, top-level procedures
     (SetDir, SelectBySpec, SelectByTitle), and every ABSTRACT
     method heading that downstream callers (StdCmds, StdApi,
     Documents) reference.
   - `SetDir` body — installs the host-supplied Directory.

   What's deferred (every concrete Directory / Window behaviour
   beyond surface):
   - `Init` for hooks (Sequencers.SetDir, Views.SetMsgHook,
     Controllers.Register, Kernel.InstallReducer,
     Services.DoLater, Dialog.RegisterLangNotifier) — these
     hook installers will appear once each producer exists.
   - The platform-side Directory body (`New`, `Open`,
     `OpenSubWindow`, `First`, `Next`, `Focus`, `Close`) —
     that's HostWindows' job, lands in Phase 4b.
   - `Window` method bodies (Init, GetSize, SetSize, Update,
     Close) — surface-only here; HostWindows installs the
     concrete subclass.
*)

    IMPORT
        Files, Stores, Sequencers, Models, Views,
        Ports, Properties, Converters, Containers, Documents;

    CONST
        (** Window.flags — bit numbers OR'd together by callers. *)
        isTool*           = 0;
        isAux*            = 1;
        noHScroll*        = 2;
        noVScroll*        = 3;
        noResize*         = 4;
        allowDuplicates*  = 5;
        neverDirty*       = 6;

        (** Directory.Select `lazy` parameter — BB stays with
            BOOLEAN sentinels rather than an enum. *)
        eager* = FALSE;
        lazy*  = TRUE;


    TYPE
        (** Abstract Window — concrete subclass owned by
            HostWindows.  Fields are read-only-exported so
            callers can observe binding without poking. *)
        WindowDesc* = ABSTRACT RECORD
            port-:  Ports.Port;
            frame-: Views.Frame;            (** BB uses Views.RootFrame; our Views slice
                                                hasn't lifted RootFrame to the public surface
                                                yet — Views.Frame is the published parent. *)
            doc-:   Documents.Document;
            seq-:   Sequencers.Sequencer;
            link-:  Window;
            sub-:   BOOLEAN;
            flags-: SET;
            loc-:   Files.Locator;
            name-:  Files.Name;
            conv-:  Converters.Converter
        END;
        Window* = POINTER TO WindowDesc;

        (** Abstract Directory — window factory.  Concrete
            implementation lives in HostWindows. *)
        DirectoryDesc* = ABSTRACT RECORD END;
        Directory*     = POINTER TO DirectoryDesc;


    VAR
        (** Current Directory.  Set by HostWindows at startup; we
            export it (read-only) so StdCmds / StdApi can route
            through it without an import cycle on the host module. *)
        dir-:    Directory;

        (** Default-fallback Directory — first one installed.
            BB keeps this so user-installed test directories can
            be uninstalled cleanly. *)
        stdDir-: Directory;


    (* -- Window methods --------------------------------------------------- *)

    PROCEDURE (w: Window) Init*    (p: Ports.Port),                          NEW, ABSTRACT;
    PROCEDURE (w: Window) SetTitle* (IN title: ARRAY OF CHAR),               NEW, ABSTRACT;
    PROCEDURE (w: Window) GetTitle* (OUT title: ARRAY OF CHAR),              NEW, ABSTRACT;
    PROCEDURE (w: Window) RefreshTitle*,                                     NEW, ABSTRACT;
    PROCEDURE (w: Window) SetSpec*  (loc: Files.Locator; IN name: Files.Name;
                                     conv: Converters.Converter),            NEW, ABSTRACT;
    PROCEDURE (w: Window) Restore*,                                          NEW, ABSTRACT;
    PROCEDURE (w: Window) Update*,                                           NEW, ABSTRACT;
    PROCEDURE (w: Window) GetSize* (OUT w0, h0: INTEGER),                    NEW, ABSTRACT;
    PROCEDURE (w: Window) SetSize* (w0, h0: INTEGER),                        NEW, ABSTRACT;
    PROCEDURE (w: Window) Close*,                                            NEW, ABSTRACT;


    (* -- Directory methods ------------------------------------------------ *)

    PROCEDURE (d: Directory) NewSequencer* (): Sequencers.Sequencer, NEW, ABSTRACT;
    PROCEDURE (d: Directory) First*        (): Window,               NEW, ABSTRACT;
    PROCEDURE (d: Directory) Next*         (w: Window): Window,      NEW, ABSTRACT;
    PROCEDURE (d: Directory) New*          (s: Sequencers.Sequencer): Window, NEW, ABSTRACT;
    PROCEDURE (d: Directory) Open*         (w: Window; doc: Documents.Document;
                                            flags: SET; IN title: ARRAY OF CHAR;
                                            loc: Files.Locator; IN fname: Files.Name;
                                            conv: Converters.Converter),     NEW, ABSTRACT;
    PROCEDURE (d: Directory) OpenSubWindow* (w: Window; doc: Documents.Document;
                                             flags: SET; IN title: ARRAY OF CHAR),
                                                                             NEW, ABSTRACT;
    PROCEDURE (d: Directory) Focus*        (): Window,               NEW, ABSTRACT;
    PROCEDURE (d: Directory) Close*        (w: Window),              NEW, ABSTRACT;


    (* -- Top-level entry points ------------------------------------------- *)

    (** Install a host-supplied Directory.  BB caches the first
        Directory as `stdDir` so framework tests can swap a
        custom Directory in and restore. *)
    PROCEDURE SetDir* (d: Directory);
    BEGIN
        dir := d;
        IF stdDir = NIL THEN
            stdDir := d
        END
    END SetDir;

    (** Look up the open window holding (loc, name) — used by
        the "Open file" UI to avoid duplicate windows for the
        same document.  Deferred until the Directory walk is
        backed by a real implementation. *)
    PROCEDURE SelectBySpec* (loc: Files.Locator; IN name: Files.Name;
                             conv: Converters.Converter; VAR done: BOOLEAN);
    BEGIN
        done := FALSE
    END SelectBySpec;

    (** Look up the open window with title `title` — used by
        StdApi.OpenToolDialog to bring the welcome page to front
        if it's already open.  Deferred for the same reason. *)
    PROCEDURE SelectByTitle* (v: Views.View; flags: SET;
                              IN title: ARRAY OF CHAR; VAR done: BOOLEAN);
    BEGIN
        done := FALSE
    END SelectByTitle;


BEGIN
    dir    := NIL;
    stdDir := NIL
END Windows.
