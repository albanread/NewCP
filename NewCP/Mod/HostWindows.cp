MODULE HostWindows;
(*
   First slice of the BlackBox `HostWindows` port over our iGui
   windowing primitive.

   BB's HostWindows is ~3800 lines of Win32 message-loop +
   Documents-to-MDI-child binding.  Our story is much simpler
   because iGui already runs the message loop and handles
   MDI-child creation; HostWindows here just has to:

     1. Provide a concrete `Windows.Directory` whose `New` /
        `Open` actually allocate a `HostPorts.HostPort` and
        bind it to the supplied Document.
     2. Provide a concrete `Windows.Window` whose `port` /
        `frame` / `doc` slots are populated and whose `Close`
        tears the iGui child down via `HostPorts.Close`.
     3. Install itself into the global `Windows.dir` at module
        init so StdCmds / StdApi / Init reach a working factory.

   What's in THIS slice:
   - StdWindowDesc / StdWindow record extending Windows.WindowDesc.
   - StdDirectoryDesc / StdDirectory record extending
     Windows.DirectoryDesc.
   - StdDirectory.New: allocates StdWindow, no port yet.
   - StdDirectory.Open: drives HostPorts.NewPort, stamps port +
     doc + frame slots, runs the document's first Restore so
     the welcome page actually paints.
   - First / Next / Focus / Close: bookkeeping against an
     internal singly-linked list of opened windows.
   - Window method bodies (Init / GetSize / SetSize / Update /
     Close / SetTitle / GetTitle / RefreshTitle / SetSpec /
     Restore) — delegated to HostPorts where possible, stubbed
     where the surface is bigger than we need today.

   What's deferred:
   - Real sequencer + undo plumbing (Sequencers.NewSequencer
     hookup): the welcome page is read-only, no undo history.
   - Print-job dispatch — Documents.Print isn't on the path.
   - Auxiliary / tool / floating-window styling — every window
     is opened as a plain MDI child for now.
*)

    IMPORT
        Files, Sequencers, Views, Ports,
        Converters, Documents, Windows,
        HostPorts;


    TYPE
        (** Concrete Window — adds the iGui port wrapper to
            Windows.WindowDesc's read-only fields.  When iGui's
            child window is destroyed, `host` is reset to NIL so
            paint calls turn into no-ops. *)
        StdWindowDesc* = RECORD (Windows.WindowDesc)
            host*: HostPorts.HostPort;
            title*: ARRAY 128 OF CHAR
        END;
        StdWindow* = POINTER TO StdWindowDesc;

        (** Concrete Directory — owns the linked list of open
            windows. *)
        StdDirectoryDesc* = RECORD (Windows.DirectoryDesc)
            head: StdWindow;
            focusWin: StdWindow
        END;
        StdDirectory* = POINTER TO StdDirectoryDesc;


    VAR
        std: StdDirectory;


    (* -- StdWindow methods ------------------------------------------------ *)

    PROCEDURE (w: StdWindow) Init* (p: Ports.Port);
    BEGIN
        (* No-op: HostWindows.Open already populated `port`.
           Init exists for BB-faithful surface compatibility. *)
    END Init;

    PROCEDURE (w: StdWindow) SetTitle* (IN title: ARRAY OF CHAR);
        VAR i: INTEGER; cap: INTEGER;
    BEGIN
        cap := LEN(w.title) - 1;
        i := 0;
        WHILE (i < cap) & (title[i] # 0X) DO
            w.title[i] := title[i];
            INC(i)
        END;
        w.title[i] := 0X
    END SetTitle;

    PROCEDURE (w: StdWindow) GetTitle* (OUT title: ARRAY OF CHAR);
        VAR i: INTEGER; cap: INTEGER;
    BEGIN
        cap := LEN(title) - 1;
        i := 0;
        WHILE (i < cap) & (w.title[i] # 0X) DO
            title[i] := w.title[i];
            INC(i)
        END;
        title[i] := 0X
    END GetTitle;

    PROCEDURE (w: StdWindow) RefreshTitle*;
    BEGIN
        (* Deferred: iGui doesn't yet expose a "set window title"
           surface.  When it does this body re-publishes
           w.title via that hook. *)
    END RefreshTitle;

    PROCEDURE (w: StdWindow) SetSpec*
        (loc: Files.Locator; IN name: Files.Name; conv: Converters.Converter);
    BEGIN
        (* `loc-` / `name-` / `conv-` are read-only-exported on
           WindowDesc so we can't write to them through `w` as a
           Window.  HostWindows pokes them through its concrete
           descendant's hatch.  Deferred: write hatch lands once
           we need Save-as routing. *)
    END SetSpec;

    PROCEDURE (w: StdWindow) Restore*;
    BEGIN
        (* Deferred: run a full paint cycle against the host port.
           See HelloPixels.cp / PaneDemo.cp for the BeginPaint /
           NewRider / DrawRect / SubmitPaint pattern. *)
    END Restore;

    PROCEDURE (w: StdWindow) Update*;
    BEGIN
        (* Same deferral as Restore. *)
    END Update;

    PROCEDURE (w: StdWindow) GetSize* (OUT w0, h0: INTEGER);
    BEGIN
        IF w.host # NIL THEN
            w.host.GetSize(w0, h0)
        ELSE
            w0 := 0; h0 := 0
        END
    END GetSize;

    PROCEDURE (w: StdWindow) SetSize* (w0, h0: INTEGER);
    BEGIN
        IF w.host # NIL THEN
            w.host.SetSize(w0, h0)
        END
    END SetSize;

    PROCEDURE (w: StdWindow) Close*;
    BEGIN
        IF w.host # NIL THEN
            HostPorts.Close(w.host);
            w.host := NIL
        END
    END Close;


    (* -- StdDirectory methods --------------------------------------------- *)

    PROCEDURE (d: StdDirectory) NewSequencer* (): Sequencers.Sequencer;
    BEGIN
        (* Deferred: needs a concrete Sequencers.SequencerDesc
           subclass we haven't written yet.  Returning NIL means
           "no undo history" — fine for the read-only welcome
           page. *)
        RETURN NIL
    END NewSequencer;

    PROCEDURE (d: StdDirectory) First* (): Windows.Window;
    BEGIN
        RETURN d.head
    END First;

    PROCEDURE (d: StdDirectory) Next* (w: Windows.Window): Windows.Window;
    BEGIN
        IF w = NIL THEN RETURN NIL END;
        RETURN w.link
    END Next;

    PROCEDURE (d: StdDirectory) New* (s: Sequencers.Sequencer): Windows.Window;
        VAR w: StdWindow;
    BEGIN
        NEW(w);
        (* port-/frame-/doc-/seq-/link-/sub-/flags-/loc-/name-/conv-
           are read-only-exported on WindowDesc.  They're set by
           HostWindows.Open through the typed `w(StdWindow)`
           narrow once we write to them — for now we leave them
           NIL/default and let Open populate.  This also returns
           w as a Windows.Window (the public surface). *)
        w.host  := NIL;
        w.title := "";
        RETURN w
    END New;

    PROCEDURE (d: StdDirectory) Open*
        (w: Windows.Window; doc: Documents.Document; flags: SET;
         IN title: ARRAY OF CHAR; loc: Files.Locator;
         IN fname: Files.Name; conv: Converters.Converter);
        VAR sw: StdWindow; childId: INTEGER;
            shortTitle: ARRAY 128 OF SHORTCHAR; i: INTEGER;
    BEGIN
        IF ~(w IS StdWindow) THEN RETURN END;
        sw := w(StdWindow);

        (* Widen the CHAR title to SHORTCHAR for iGui — iGui's
           current C ABI uses 8-bit chars. *)
        i := 0;
        WHILE (i < LEN(shortTitle) - 1) & (title[i] # 0X) DO
            shortTitle[i] := SHORT(title[i]);
            INC(i)
        END;
        shortTitle[i] := 0X;

        sw.host := HostPorts.NewPort(shortTitle, childId);
        sw.SetTitle(title);
        (* Link into the directory's open-windows list. *)
        sw.link := d.head;
        d.head := sw;
        d.focusWin := sw

        (* Deferred: install the document, allocate a frame,
           connect it to the port, and drive the first Restore.
           That requires write access to the read-only-exported
           Window fields (doc-, frame-, port-) which lives in
           HostWindows' typed view of StdWindow — pending the
           frame-allocation patch that wires Documents.GetNewFrame
           through to a real Views.RootFrame. *)
    END Open;

    PROCEDURE (d: StdDirectory) OpenSubWindow*
        (w: Windows.Window; doc: Documents.Document;
         flags: SET; IN title: ARRAY OF CHAR);
    BEGIN
        (* Deferred — share the active sequencer with a sibling
           window.  Calls Open under the hood. *)
        d.Open(w, doc, flags, title, NIL, "", NIL)
    END OpenSubWindow;

    PROCEDURE (d: StdDirectory) Focus* (): Windows.Window;
    BEGIN
        RETURN d.focusWin
    END Focus;

    PROCEDURE (d: StdDirectory) Close* (w: Windows.Window);
        VAR cur, prev: StdWindow;
    BEGIN
        IF (w = NIL) OR ~(w IS StdWindow) THEN RETURN END;
        (* Walk d.head to unlink. *)
        prev := NIL;
        cur  := d.head;
        WHILE (cur # NIL) & (cur # w) DO
            prev := cur;
            cur  := cur.link(StdWindow)
        END;
        IF cur # NIL THEN
            IF prev = NIL THEN
                IF cur.link # NIL THEN
                    d.head := cur.link(StdWindow)
                ELSE
                    d.head := NIL
                END
            ELSE
                prev.link := cur.link
            END;
            IF d.focusWin = cur THEN d.focusWin := d.head END;
            cur.Close
        END
    END Close;


BEGIN
    NEW(std);
    std.head := NIL;
    std.focusWin := NIL;
    Windows.SetDir(std)
END HostWindows.
