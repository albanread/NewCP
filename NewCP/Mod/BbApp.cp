MODULE BbApp;
(*
   NewCP application entry point — full slice.

   Builds on BbInit but adds:
   - A proper File / Edit / Window / Log menu.
   - Menu command routing to StdCmds / TextCmds / StdLog.
   - An initial "Welcome" window opened at startup.

   Run with:
       newcp-driver run-igui BbApp.Run

   Menu layout:
     File > New          (1001)
     File > Open…        (1002)
     File > Save As…     (1003)
     File > Close Window (1004)
     File > ─────────
     File > Quit         (1099)

     Edit > Select All   (1101)
     Edit > Deselect     (1102)
     Edit > ─────────
     Edit > Find…        (1103)
     Edit > Find Again   (1104)
     Edit > ─────────
     Edit > Bold         (1110)
     Edit > Italic       (1111)
     Edit > Plain        (1112)

     Log  > Show Log     (1200)

     Window > Cascade    (MDI)
     Window > Tile H     (MDI)
     Window > Tile V     (MDI)
     Window > ─────────
     Window > Close All  (MDI)
*)

    IMPORT iGui, Console, HostWindows,
           Documents, Windows, TextModels, TextViews,
           StdCmds, TextCmds, TextControllers, StdLog, Services;

    CONST
        (* File menu *)
        CmdNew     = 1001;
        CmdOpen    = 1002;
        CmdSave    = 1003;
        CmdCloseW  = 1004;
        CmdQuit    = 1099;

        (* Edit menu *)
        CmdUndo    = 1100;
        CmdRedo    = 1108;
        CmdSelAll  = 1101;
        CmdDesel   = 1102;
        CmdFind    = 1103;
        CmdFindAgn = 1104;
        CmdCut     = 1105;
        CmdCopy    = 1106;
        CmdPaste   = 1107;
        CmdBold    = 1110;
        CmdItalic  = 1111;
        CmdPlain   = 1112;

        (* Log menu *)
        CmdShowLog = 1200;

        (* Window dimensions *)
        winW = 800;
        winH = 600;

        (* No string constant for the menu spec — pass directly to SetMenu below. *)


    (* -- Private: open the initial "Welcome" document ------------------- *)

    PROCEDURE OpenWelcome;
        VAR d:   TextModels.Doc;
            wr:  TextModels.Writer;
            v:   TextViews.View;
            doc: Documents.Document;
            win: Windows.Window;
    BEGIN
        NEW(d);
        wr := d.NewWriter(NIL);
        wr.WriteString("Welcome to NewCP!");
        wr.WriteChar(TextModels.line);
        wr.WriteString("Use File > Open to open a text file.");
        wr.WriteChar(TextModels.line);
        wr.WriteString("Use Log > Show Log to open the log window.");
        v := TextViews.dir.New(d);
        IF v = NIL THEN RETURN END;
        doc := Documents.dir.New(v, winW, winH);
        IF doc = NIL THEN RETURN END;
        win := Windows.Open(doc, "Welcome to NewCP", winW, winH)
    END OpenWelcome;


    (* -- Main entry point ----------------------------------------------- *)

    PROCEDURE Run*;
        VAR ok:   INTSHORT;
            kind, childId, timeMs, p1, p2, p3, p4: INTEGER;
            ch: CHAR;
            mouseOp: INTEGER;
    BEGIN
        Console.WriteShortString("BbApp: starting"); Console.WriteLn;

        (* Install the menu bar — single literal, no & concatenation. *)
        ok := iGui.SetMenu("MENU &File;ITEM 1001 &New;ITEM 1002 &Open...;ITEM 1003 Save &As...;ITEM 1004 &Close Window;SEP;ITEM 1099 &Quit;MENU &Edit;ITEM 1100 &Undo;ITEM 1108 &Redo;SEP;ITEM 1101 Select &All;ITEM 1102 &Deselect;SEP;ITEM 1105 Cu&t;ITEM 1106 &Copy;ITEM 1107 &Paste;SEP;ITEM 1103 &Find...;ITEM 1104 Find &Again;SEP;ITEM 1110 &Bold;ITEM 1111 &Italic;ITEM 1112 P&lain;MENU &Log;ITEM 1200 &Show Log;MENU &Window;MDI cascade;MDI tile-h;MDI tile-v;SEP;MDI close-all;MDI arrange-icons");
        IF ok = 0 THEN
            Console.WriteShortString("BbApp: SetMenu failed"); Console.WriteLn
        END;

        (* Open the welcome document. *)
        OpenWelcome;

        Console.WriteShortString("BbApp: running — close the frame to exit");
        Console.WriteLn;

        (* Event loop.
           50 ms timeout so Services.Step drains deferred actions
           even when the UI is idle.  ok = 0 means timeout with
           no event; ok ≠ 0 means an event was delivered. *)
        REPEAT
            ok := iGui.NextEvent(kind, childId, timeMs, p1, p2, p3, p4, 50);
            IF ok # 0 THEN
                IF    kind = iGui.EvPaint      THEN HostWindows.PaintChild(childId)
                ELSIF kind = iGui.EvResize     THEN HostWindows.ResizeChild(childId, p1, p2)
                ELSIF kind = iGui.EvClose      THEN HostWindows.CloseChild(childId)
                ELSIF kind = iGui.EvFrameClose THEN EXIT
                ELSIF kind = iGui.EvFocus      THEN HostWindows.FocusChild(childId)
                ELSIF kind = iGui.EvKey        THEN
                    (* p4 high bit 0 = key-down; p3 = mods *)
                    IF p4 MOD 2 = 1 THEN  (* key down *)
                        IF TextControllers.HandleNavKey(p1, ODD(p3 DIV iGui.ModShift)) THEN
                            HostWindows.PaintChild(childId)
                        END
                    END
                ELSIF kind = iGui.EvMouse      THEN
                    (* p3 = mods | button<<8 | op<<16 *)
                    mouseOp := p3 DIV 65536;
                    IF mouseOp = iGui.MouseLeftDown THEN
                        HostWindows.HandleMouseDown(childId, p1, p2);
                        HostWindows.PaintChild(childId)
                    ELSIF mouseOp = iGui.MouseLeftUp THEN
                        HostWindows.HandleMouseUp(childId)
                    ELSIF mouseOp = iGui.MouseMove THEN
                        (* Drag selection: extend from the click anchor to here. *)
                        HostWindows.HandleMouseDrag(childId, p1, p2);
                        HostWindows.PaintChild(childId)
                    ELSIF mouseOp = iGui.MouseWheel THEN
                        (* p4 = wheel_delta | wheel_lines<<16
                           positive = up (wheel_delta > 0 → scroll up = lines < 0) *)
                        IF p4 > 0 THEN
                            HostWindows.ScrollLines(childId, -3)
                        ELSE
                            HostWindows.ScrollLines(childId, 3)
                        END
                    END
                ELSIF kind = iGui.EvChar       THEN
                    (* p2 = modifier bits; ODD(p2 DIV 2) = Ctrl held *)
                    IF ODD(p2 DIV iGui.ModControl) THEN
                        IF    (p1 = ORD('x')) OR (p1 = ORD('X')) THEN
                            TextCmds.Cut; HostWindows.PaintChild(childId)
                        ELSIF (p1 = ORD('c')) OR (p1 = ORD('C')) THEN
                            TextCmds.Copy
                        ELSIF (p1 = ORD('v')) OR (p1 = ORD('V')) THEN
                            TextCmds.Paste; HostWindows.PaintChild(childId)
                        ELSIF (p1 = ORD('a')) OR (p1 = ORD('A')) THEN
                            TextCmds.SelectAll; HostWindows.PaintChild(childId)
                        ELSIF (p1 = ORD('z')) OR (p1 = ORD('Z')) THEN
                            TextCmds.Undo; HostWindows.PaintChild(childId)
                        ELSIF (p1 = ORD('y')) OR (p1 = ORD('Y')) THEN
                            TextCmds.Redo; HostWindows.PaintChild(childId)
                        ELSIF (p1 = ORD('f')) OR (p1 = ORD('F')) THEN
                            TextCmds.Find; HostWindows.PaintChild(childId)
                        ELSIF (p1 = ORD('g')) OR (p1 = ORD('G')) THEN
                            TextCmds.FindAgain; HostWindows.PaintChild(childId)
                        ELSIF (p1 = ORD('b')) OR (p1 = ORD('B')) THEN
                            TextCmds.Bold; HostWindows.PaintChild(childId)
                        ELSIF (p1 = ORD('i')) OR (p1 = ORD('I')) THEN
                            TextCmds.Italic; HostWindows.PaintChild(childId)
                        ELSIF (p1 = ORD('m')) OR (p1 = ORD('M')) THEN
                            TextCmds.Plain; HostWindows.PaintChild(childId)
                        END
                    ELSE
                        (* Pass printable chars, Tab (9), and Enter (13) to
                           HandleKey.  Skip Backspace (8) — it arrives as both
                           EvKey (handled by HandleNavKey/VkBack) and EvChar;
                           passing it here too would cause a double-delete.
                           All other control chars (< 32 except 9, 13) are
                           silently discarded. *)
                        IF (p1 >= 32) OR (p1 = 9) OR (p1 = 13) THEN
                            ch := CHR(p1);
                            IF TextControllers.HandleKey(ch) THEN
                                HostWindows.PaintChild(childId)
                            END
                        END
                    END
                ELSIF kind = iGui.EvMenu       THEN
                    IF    p2 = CmdNew     THEN StdCmds.New
                    ELSIF p2 = CmdOpen    THEN StdCmds.Open
                    ELSIF p2 = CmdSave    THEN StdCmds.Save
                    ELSIF p2 = CmdCloseW  THEN StdCmds.CloseWin
                    ELSIF p2 = CmdQuit    THEN StdCmds.Quit; EXIT
                    ELSIF p2 = CmdUndo    THEN TextCmds.Undo; HostWindows.PaintChild(childId)
                    ELSIF p2 = CmdRedo    THEN TextCmds.Redo; HostWindows.PaintChild(childId)
                    ELSIF p2 = CmdSelAll  THEN TextCmds.SelectAll;               HostWindows.PaintChild(childId)
                    ELSIF p2 = CmdDesel   THEN TextCmds.Deselect;                HostWindows.PaintChild(childId)
                    ELSIF p2 = CmdCut     THEN TextCmds.Cut;                     HostWindows.PaintChild(childId)
                    ELSIF p2 = CmdCopy    THEN TextCmds.Copy
                    ELSIF p2 = CmdPaste   THEN TextCmds.Paste;                   HostWindows.PaintChild(childId)
                    ELSIF p2 = CmdFind    THEN TextCmds.Find;                    HostWindows.PaintChild(childId)
                    ELSIF p2 = CmdFindAgn THEN TextCmds.FindAgain;               HostWindows.PaintChild(childId)
                    ELSIF p2 = CmdBold    THEN TextCmds.Bold;                    HostWindows.PaintChild(childId)
                    ELSIF p2 = CmdItalic  THEN TextCmds.Italic;                  HostWindows.PaintChild(childId)
                    ELSIF p2 = CmdPlain   THEN TextCmds.Plain;                   HostWindows.PaintChild(childId)
                    ELSIF p2 = CmdShowLog THEN StdLog.Open
                    END
                END
            END;
            (* Drain any deferred Services.Action items that
               became due during this tick. *)
            Services.Step
        UNTIL FALSE;

        Console.WriteShortString("BbApp: done"); Console.WriteLn
    END Run;

END BbApp.
