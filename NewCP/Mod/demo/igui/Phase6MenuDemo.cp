MODULE Phase6MenuDemo;

(* iGui Phase 6 acceptance demo: menu bar + MDI verbs.

   - Installs a File / Edit / Window menu on the iGui frame.
   - File > New opens a fresh document window.
   - File > Close All closes all document windows (via MDI verb).
   - File > Exit closes the frame.
   - Window menu has the standard MDI verbs: Cascade, Tile H/V,
     Close All, Arrange Icons.
   - Edit menu items just print their click to the console.

   Run:
     newcp-driver run-igui Phase6MenuDemo.Run
*)

IMPORT iGui, Console;

CONST
  CmdNew      = 1001;
  CmdCloseAll = 1002;
  CmdAbout    = 1003;
  CmdExit     = 1099;
  CmdCut      = 1101;
  CmdCopy     = 1102;
  CmdPaste    = 1103;

VAR
  nextDocLabel: INTEGER;

PROCEDURE Paint(childId: INTEGER; tint: REAL);
  VAR ok: INTSHORT;
BEGIN
  iGui.BeginBatch(childId);
  iGui.EmitClear(0.10 + tint * 0.05, 0.14, 0.22 - tint * 0.04, 1.0);
  iGui.EmitFillRect(40.0, 40.0, 200.0, 100.0, 8.0,
                    0.85, 0.65, 0.30, 1.0);
  iGui.EmitStrokeRect(20.0, 20.0, 480.0, 280.0, 4.0, 1.5,
                      0.95, 0.95, 0.95, 1.0);
  ok := iGui.SubmitBatch();
  IF ok = 0 THEN
    Console.WriteShortString("paint: SubmitBatch failed"); Console.WriteLn
  END
END Paint;

PROCEDURE OpenNewDoc;
  VAR
    title: ARRAY 64 OF SHORTCHAR;
    docId: INTEGER;
    ok: INTSHORT;
    n: INTEGER;
    pos, i: INTEGER;
    base: INTEGER;
    label: REAL;
BEGIN
  INC(nextDocLabel);
  (* Hand-fill "Document N" since NewCP doesn't have string concat. *)
  pos := 0;
  title[pos] := SHORT("D"); INC(pos);
  title[pos] := SHORT("o"); INC(pos);
  title[pos] := SHORT("c"); INC(pos);
  title[pos] := SHORT("u"); INC(pos);
  title[pos] := SHORT("m"); INC(pos);
  title[pos] := SHORT("e"); INC(pos);
  title[pos] := SHORT("n"); INC(pos);
  title[pos] := SHORT("t"); INC(pos);
  title[pos] := SHORT(" "); INC(pos);
  n := nextDocLabel;
  IF n = 0 THEN
    title[pos] := SHORT("0"); INC(pos)
  ELSE
    base := 1;
    WHILE base * 10 <= n DO base := base * 10 END;
    WHILE base > 0 DO
      title[pos] := SHORT(CHR(ORD("0") + (n DIV base) MOD 10));
      INC(pos);
      base := base DIV 10
    END
  END;
  title[pos] := 0X;
  ok := iGui.OpenChild(title, docId);
  IF ok # 0 THEN
    Console.WriteShortString("opened ");
    Console.WriteShortString(title);
    Console.WriteShortString(" id=");
    Console.WriteInt(docId);
    Console.WriteLn;
    label := 1.0;
    i := 0;
    WHILE i < nextDocLabel DO label := label + 1.0; INC(i) END;
    Paint(docId, 0.05 * label)
  END
END OpenNewDoc;

PROCEDURE Run*;
  VAR
    kind, childId, timeMs, p1, p2, p3, p4: INTEGER;
    ok: INTSHORT;
BEGIN
  Console.WriteShortString("Phase 6 demo: menu bar + MDI verbs"); Console.WriteLn;

  nextDocLabel := 0;

  ok := iGui.SetMenu("MENU &File;ITEM 1001 &New Document;ITEM 1002 Close &All Documents;SEP;ITEM 1003 &About;SEP;ITEM 1099 E&xit;MENU &Edit;ITEM 1101 Cu&t;ITEM 1102 &Copy;ITEM 1103 &Paste;MENU &Window;MDI cascade;MDI tile-h;MDI tile-v;SEP;MDI close-all;MDI arrange-icons");
  IF ok = 0 THEN
    Console.WriteShortString("SetMenu failed"); Console.WriteLn;
    RETURN
  END;
  Console.WriteShortString("menu installed; opening 3 starting docs"); Console.WriteLn;

  (* Open three documents to start so Cascade / Tile have something
     to arrange. *)
  OpenNewDoc;
  OpenNewDoc;
  OpenNewDoc;

  Console.WriteShortString("use the File / Window menus; close the frame to exit");
  Console.WriteLn;

  REPEAT
    ok := iGui.NextEvent(kind, childId, timeMs, p1, p2, p3, p4, -1);
    IF ok # 0 THEN
      IF kind = iGui.EvMenu THEN
        IF p2 = CmdNew THEN
          OpenNewDoc
        ELSIF p2 = CmdCloseAll THEN
          Console.WriteShortString("File > Close All"); Console.WriteLn;
          iGui.MdiCloseAll
        ELSIF p2 = CmdAbout THEN
          Console.WriteShortString("File > About: iGui Phase 6 menu demo"); Console.WriteLn
        ELSIF p2 = CmdExit THEN
          Console.WriteShortString("File > Exit"); Console.WriteLn;
          iGui.Quit;
          EXIT
        ELSIF p2 = CmdCut THEN
          Console.WriteShortString("Edit > Cut"); Console.WriteLn
        ELSIF p2 = CmdCopy THEN
          Console.WriteShortString("Edit > Copy"); Console.WriteLn
        ELSIF p2 = CmdPaste THEN
          Console.WriteShortString("Edit > Paste"); Console.WriteLn
        ELSE
          Console.WriteShortString("[menu] item="); Console.WriteInt(p2); Console.WriteLn
        END
      ELSIF kind = iGui.EvFrameClose THEN
        EXIT
      END
    END
  UNTIL FALSE
END Run;

END Phase6MenuDemo.
