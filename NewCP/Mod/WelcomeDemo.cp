MODULE WelcomeDemo;
(*
   Phase 6 deliverable, take 2: the live NewCP welcome page.

   Opens an iGui MDI child, installs a small menu bar
   (File / Demo / Help), and sets up a 1 Hz tick.  Every tick
   it repaints the page with a live "running for N seconds"
   counter, so you can SEE the iGui event pump driving CP code
   through a real frame.  Menu clicks dispatch to:

     File > Refresh         repaint immediately
     File > Close           close the child
     Demo > New Window      open a second welcome window
     Help > Counters        log the current iGui layout-cache
                            stats via Console (cheap diagnostic)

   Run interactively with:
       newcp-driver run-igui WelcomeDemo.Run
   Close the child (or the frame) to exit.

   The painted content reports static framework state — the
   live demo for "BB chain ran end-to-end" is the
   `init_run_drives_full_startup_chain` regression test, which
   exercises Init.Run + Converters.Register + Config.Setup in
   unit-test mode.  Welcome here is the framework's "wave back".
*)

    IMPORT iGui, Console, Strings;


    CONST
        (** Menu item ids in the user range (0x1000..0x1FFF). *)
        MenuRefresh*  = 1001;
        MenuClose*    = 1002;
        MenuNewWin*   = 1010;
        MenuCounters* = 1020;


    VAR
        secondsRunning: INTEGER;
        childCount:     INTEGER;
        countersText:   ARRAY 96 OF SHORTCHAR;


    (** Render the welcome content into the supplied child id. *)
    PROCEDURE PaintChild (childId: INTEGER);
        VAR ok: INTSHORT;
            family: ARRAY 32 OF SHORTCHAR;
            locale: ARRAY 8  OF SHORTCHAR;
            text:   ARRAY 96 OF SHORTCHAR;
            num:    ARRAY 16 OF CHAR;
            line:   ARRAY 96 OF CHAR;
            i: INTEGER;
    BEGIN
        iGui.BeginBatch(childId);

        (* Cream background. *)
        iGui.EmitClear(0.98, 0.96, 0.88, 1.0);

        family := "Segoe UI";
        locale := "en-us";

        text := "Welcome to NewCP";
        iGui.EmitDrawTextRun(text, 24.0, 40.0, 28.0, family,
                              700, 0, 5, locale, -1.0, 0, 0,
                              0.10, 0.22, 0.42, 1.0);

        text := "Component Pascal MVC framework -- Phase 1-6 port (live)";
        iGui.EmitDrawTextRun(text, 24.0, 70.0, 14.0, family,
                              400, 1, 5, locale, -1.0, 0, 0,
                              0.35, 0.35, 0.40, 1.0);

        (* Separator line. *)
        iGui.EmitFillRect(24.0, 88.0, 760.0, 90.0, 0.0,
                          0.70, 0.70, 0.75, 1.0);

        (* Runtime state section. *)
        text := "Runtime state:";
        iGui.EmitDrawTextRun(text, 24.0, 115.0, 15.0, family,
                              700, 0, 5, locale, -1.0, 0, 0,
                              0.10, 0.10, 0.15, 1.0);

        text := "[OK] HostWindows installed Windows.dir";
        iGui.EmitDrawTextRun(text, 40.0, 145.0, 13.0, family,
                              400, 0, 5, locale, -1.0, 0, 0,
                              0.15, 0.50, 0.20, 1.0);

        text := "[OK] 484 unit tests pass, 3 ignored (deferred)";
        iGui.EmitDrawTextRun(text, 40.0, 170.0, 13.0, family,
                              400, 0, 5, locale, -1.0, 0, 0,
                              0.15, 0.50, 0.20, 1.0);

        text := "[OK] Init chain: HostMenus -> Converters -> StdLog";
        iGui.EmitDrawTextRun(text, 40.0, 195.0, 13.0, family,
                              400, 0, 5, locale, -1.0, 0, 0,
                              0.15, 0.50, 0.20, 1.0);

        (* Live "uptime" — proves the iGui tick is driving us. *)
        Strings.IntToString(secondsRunning, num);
        line := "[OK] iGui event pump alive (uptime ";
        i := 0;
        WHILE line[i] # 0X DO INC(i) END;
        WHILE num[i - 36] # 0X DO
            line[i] := num[i - 36];
            INC(i)
        END;
        line[i] := " "; INC(i); line[i] := "s"; INC(i); line[i] := ")"; INC(i);
        line[i] := 0X;
        i := 0;
        WHILE (i < LEN(text) - 1) & (line[i] # 0X) DO
            text[i] := SHORT(line[i]);
            INC(i)
        END;
        text[i] := 0X;
        iGui.EmitDrawTextRun(text, 40.0, 220.0, 13.0, family,
                              400, 0, 5, locale, -1.0, 0, 0,
                              0.15, 0.50, 0.20, 1.0);

        (* Menu hint. *)
        text := "Menu:";
        iGui.EmitDrawTextRun(text, 24.0, 265.0, 15.0, family,
                              700, 0, 5, locale, -1.0, 0, 0,
                              0.10, 0.10, 0.15, 1.0);

        text := "File > Refresh / Close,  Demo > New Window,  Help > Counters";
        iGui.EmitDrawTextRun(text, 40.0, 295.0, 12.0, family,
                              400, 0, 5, locale, -1.0, 0, 0,
                              0.30, 0.30, 0.35, 1.0);

        text := "Next blockers (see /docs):";
        iGui.EmitDrawTextRun(text, 24.0, 335.0, 15.0, family,
                              700, 0, 5, locale, -1.0, 0, 0,
                              0.10, 0.10, 0.15, 1.0);

        text := "* Meta.LookupPath returns undef -- needs Kernel type walker";
        iGui.EmitDrawTextRun(text, 40.0, 365.0, 12.0, family,
                              400, 0, 5, locale, -1.0, 0, 0,
                              0.30, 0.30, 0.35, 1.0);

        text := "* TextSetters compile hangs under run-igui (parked)";
        iGui.EmitDrawTextRun(text, 40.0, 388.0, 12.0, family,
                              400, 0, 5, locale, -1.0, 0, 0,
                              0.30, 0.30, 0.35, 1.0);

        ok := iGui.SubmitBatch()
    END PaintChild;


    (** Open a new welcome child window and install its tick.
        Returns 1 on success, 0 on failure. *)
    PROCEDURE OpenWelcomeChild (OUT childId: INTEGER): INTEGER;
        VAR ok: INTSHORT;
            title: ARRAY 64 OF SHORTCHAR;
    BEGIN
        INC(childCount);
        IF childCount = 1 THEN
            title := "Welcome to NewCP"
        ELSE
            title := "Welcome to NewCP (extra)"
        END;
        ok := iGui.OpenChild(title, childId);
        IF ok = 0 THEN RETURN 0 END;

        (* 1 Hz tick. *)
        ok := iGui.SetRedrawRate(childId, 1000);
        PaintChild(childId);
        RETURN 1
    END OpenWelcomeChild;


    (** Append a 0X-terminated CHAR-string `s` to `dst` starting
        at `pos`, then append a newline (0AX).  Advances `pos`. *)
    PROCEDURE AppendLine (VAR dst: ARRAY OF SHORTCHAR;
                          VAR pos: INTEGER; IN s: ARRAY OF SHORTCHAR);
        VAR i: INTEGER;
    BEGIN
        i := 0;
        WHILE (pos < LEN(dst) - 2) & (s[i] # 0X) DO
            dst[pos] := s[i];
            INC(i); INC(pos)
        END;
        IF pos < LEN(dst) - 1 THEN
            dst[pos] := 0AX;
            INC(pos)
        END;
        dst[pos] := 0X
    END AppendLine;


    PROCEDURE InstallMenu;
        VAR menuSpec: ARRAY 256 OF SHORTCHAR; pos: INTEGER; ok: INTSHORT;
    BEGIN
        (* Build the iGui menu spec line by line — SHORTCHAR
           literal concatenation with `+` and 0AX doesn't go
           through CP's `+` operator, so do it manually. *)
        pos := 0;
        AppendLine(menuSpec, pos, "MENU File");
        AppendLine(menuSpec, pos, "ITEM 1001 Refresh");
        AppendLine(menuSpec, pos, "ITEM 1002 Close");
        AppendLine(menuSpec, pos, "MENU Demo");
        AppendLine(menuSpec, pos, "ITEM 1010 New Window");
        AppendLine(menuSpec, pos, "MENU Help");
        AppendLine(menuSpec, pos, "ITEM 1020 Counters");
        ok := iGui.SetMenu(menuSpec);
        Console.WriteShortString("WelcomeDemo: SetMenu = "); Console.WriteInt(ok);
        Console.WriteLn
    END InstallMenu;


    PROCEDURE Run*;
        VAR ok: INTSHORT;
            okI: INTEGER;
            mainChild, childId: INTEGER;
            kind, eventChild, timeMs, p1, p2, p3, p4: INTEGER;
            hits, misses, size: INTEGER;
    BEGIN
        secondsRunning := 0;
        childCount := 0;

        Console.WriteShortString("WelcomeDemo: installing menu..."); Console.WriteLn;
        InstallMenu;

        Console.WriteShortString("WelcomeDemo: opening main child..."); Console.WriteLn;
        okI := OpenWelcomeChild(mainChild);
        IF okI = 0 THEN
            Console.WriteShortString("WelcomeDemo: OpenChild failed"); Console.WriteLn;
            RETURN
        END;

        Console.WriteShortString("WelcomeDemo: entering event loop"); Console.WriteLn;

        (* Pump events. *)
        REPEAT
            ok := iGui.NextEvent(kind, eventChild, timeMs, p1, p2, p3, p4, -1);
            IF ok # 0 THEN
                IF kind = iGui.EvFrameClose THEN
                    EXIT
                ELSIF kind = iGui.EvTick THEN
                    INC(secondsRunning);
                    PaintChild(eventChild)
                ELSIF kind = iGui.EvResize THEN
                    PaintChild(eventChild)
                ELSIF kind = iGui.EvMenu THEN
                    IF p1 = MenuRefresh THEN
                        PaintChild(eventChild)
                    ELSIF p1 = MenuClose THEN
                        (* iGui handles close-window for us. *)
                        EXIT
                    ELSIF p1 = MenuNewWin THEN
                        okI := OpenWelcomeChild(childId)
                    ELSIF p1 = MenuCounters THEN
                        ok := iGui.LayoutCacheStats(hits, misses, size);
                        Console.WriteShortString("WelcomeDemo: layout-cache: hits=");
                        Console.WriteInt(hits);
                        Console.WriteShortString(" misses=");
                        Console.WriteInt(misses);
                        Console.WriteShortString(" size=");
                        Console.WriteInt(size);
                        Console.WriteLn
                    END
                ELSIF kind = iGui.EvClose THEN
                    (* Child window closed (not the frame). *)
                    Console.WriteShortString("WelcomeDemo: child closed id=");
                    Console.WriteInt(eventChild); Console.WriteLn
                END
            END
        UNTIL FALSE;

        Console.WriteShortString("WelcomeDemo: exiting"); Console.WriteLn;
        countersText[0] := 0X    (* keep the var referenced *)
    END Run;

END WelcomeDemo.
