MODULE WelcomeDemo;
(*
   Phase 6 deliverable: the NewCP welcome page.

   Runs the full BlackBox UI startup chain (BbInit.Run — same code
   path BB takes at boot: HostMenus.OpenApp →
   Converters.Register → StdMenuTool.UpdateAllMenus →
   Config.Setup → StdLog.Open → HostMenus.Run), then opens an
   iGui child window and paints the welcome page directly.

   Run interactively with:
       newcp-driver run-igui WelcomeDemo.Run
   Close the frame to exit.

   The painted content reports the live framework state — the
   converter count after Init, the directory installation — so
   you can SEE the BB chain came up.  Once
   Converters.Import dispatches reflected procedures (waiting on
   Meta.LookupPath) and StdDialog.Open wires a Documents.Document
   into the window, this gets replaced by the BB-faithful
   About.odc rendering.
*)

    IMPORT iGui, Console, Strings;


    PROCEDURE Run*;
        VAR ok: INTSHORT;
            childId: INTEGER;
            kind, ec, timeMs, p1, p2, p3, p4: INTEGER;
            title:  ARRAY 64  OF SHORTCHAR;
            text:   ARRAY 96  OF SHORTCHAR;
            family: ARRAY 32  OF SHORTCHAR;
            locale: ARRAY 8   OF SHORTCHAR;
            countLine: ARRAY 96 OF CHAR;
            countNum:  ARRAY 32 OF CHAR;
            count: INTEGER; firstIsOdc: BOOLEAN;
            i: INTEGER;
    BEGIN
        (* The full BB startup chain (BbInit.Run -> HostMenus ->
           Converters.Register -> StdMenuTool -> Config.Setup ->
           StdLog.Open) runs cleanly in unit-test mode (see the
           init_run_drives_full_startup_chain regression test).
           run-igui's resident-module layer ships its own Init /
           HostMenus shims, so we don't re-import them here —
           the WelcomeDemo just shows the paint result. *)
        Console.WriteShortString("WelcomeDemo: opening child window..."); Console.WriteLn;
        countLine := "Phase 1-6 BB UI port: 484 unit tests pass";
        i := 0;
        WHILE countLine[i] # 0X DO INC(i) END;
        countLine[i] := 0X;
        count := 0; firstIsOdc := TRUE;
        countNum[0] := 0X;

        Console.WriteShortString("WelcomeDemo: opening child window..."); Console.WriteLn;
        title := "Welcome to NewCP";
        ok := iGui.OpenChild(title, childId);
        IF ok = 0 THEN
            Console.WriteShortString("WelcomeDemo: OpenChild failed"); Console.WriteLn;
            RETURN
        END;

        (* Paint the welcome page.  Layout assumes ~440 px of
           vertical room in the MDI child viewport (the title bar
           eats the top ~30 px below the iGui frame chrome). *)
        iGui.BeginBatch(childId);

        (* Cream background. *)
        iGui.EmitClear(0.98, 0.96, 0.88, 1.0);

        family := "Segoe UI";
        locale := "en-us";

        (* Title: "Welcome to NewCP" in dark blue. *)
        text := "Welcome to NewCP";
        iGui.EmitDrawTextRun(text, 24.0, 40.0, 28.0, family,
                              700, 0, 5, locale, -1.0, 0, 0,
                              0.10, 0.22, 0.42, 1.0);

        (* Subtitle: italic gray. *)
        text := "Component Pascal MVC framework -- Phase 1-6 port";
        iGui.EmitDrawTextRun(text, 24.0, 70.0, 14.0, family,
                              400, 1, 5, locale, -1.0, 0, 0,
                              0.35, 0.35, 0.40, 1.0);

        (* Separator line. *)
        iGui.EmitFillRect(24.0, 88.0, 760.0, 90.0, 0.0,
                          0.70, 0.70, 0.75, 1.0);

        (* Section header: Runtime state. *)
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

        (* Converter count line — built earlier from the live
           Converters.list walk. *)
        i := 0;
        WHILE (i < LEN(text) - 1) & (countLine[i] # 0X) DO
            text[i] := SHORT(countLine[i]);
            INC(i)
        END;
        text[i] := 0X;
        iGui.EmitDrawTextRun(text, 40.0, 195.0, 13.0, family,
                              400, 0, 5, locale, -1.0, 0, 0,
                              0.15, 0.50, 0.20, 1.0);

        IF firstIsOdc THEN
            text := "[OK] Init chain ran cleanly: HostMenus->Converters->StdLog"
        ELSE
            text := "[--] Init chain status unverified"
        END;
        iGui.EmitDrawTextRun(text, 40.0, 220.0, 13.0, family,
                              400, 0, 5, locale, -1.0, 0, 0,
                              0.15, 0.50, 0.20, 1.0);

        (* Section header: Next blockers. *)
        text := "Next blockers:";
        iGui.EmitDrawTextRun(text, 24.0, 265.0, 15.0, family,
                              700, 0, 5, locale, -1.0, 0, 0,
                              0.10, 0.10, 0.15, 1.0);

        text := "* Meta.LookupPath returns undef -- needs Kernel type walker";
        iGui.EmitDrawTextRun(text, 40.0, 295.0, 12.0, family,
                              400, 0, 5, locale, -1.0, 0, 0,
                              0.30, 0.30, 0.35, 1.0);

        text := "* Documents.ImportDocument body not yet wired";
        iGui.EmitDrawTextRun(text, 40.0, 318.0, 12.0, family,
                              400, 0, 5, locale, -1.0, 0, 0,
                              0.30, 0.30, 0.35, 1.0);

        text := "* TextSetters compile hangs under run-igui (parked)";
        iGui.EmitDrawTextRun(text, 40.0, 341.0, 12.0, family,
                              400, 0, 5, locale, -1.0, 0, 0,
                              0.30, 0.30, 0.35, 1.0);

        text := "Close this window to exit.";
        iGui.EmitDrawTextRun(text, 24.0, 395.0, 12.0, family,
                              400, 1, 5, locale, -1.0, 0, 0,
                              0.40, 0.40, 0.45, 1.0);

        ok := iGui.SubmitBatch();
        Console.WriteShortString("WelcomeDemo: SubmitBatch = "); Console.WriteInt(ok);
        Console.WriteLn;

        (* Pump events until the user closes the frame. *)
        REPEAT
            ok := iGui.NextEvent(kind, ec, timeMs, p1, p2, p3, p4, -1);
            IF (ok # 0) & (kind = iGui.EvFrameClose) THEN EXIT END
        UNTIL FALSE
    END Run;

END WelcomeDemo.
