MODULE App;

IMPORT Factorial, Graph, HostWindows, Log, WinBatch, WinFrame, WinSpec, WinView;

VAR
  gridPane: INTEGER;

PROCEDURE QueueDemoGrid;
  CONST
    Fg = 0FFFFFFL;
    Bg = 0;
  VAR
    ok: INTSHORT;
BEGIN
  IF gridPane = 0 THEN
    RETURN
  END;
  ok := WinBatch.Begin(gridPane, 1, SHORT(0));
  ok := WinBatch.TextCell(SHORT(0), SHORT(0), ORD('B'), Fg, Bg);
  ok := WinBatch.TextCell(SHORT(0), SHORT(1), ORD('A'), Fg, Bg);
  ok := WinBatch.TextCell(SHORT(0), SHORT(2), ORD('T'), Fg, Bg);
  ok := WinBatch.TextCell(SHORT(0), SHORT(3), ORD('C'), Fg, Bg);
  ok := WinBatch.TextCell(SHORT(0), SHORT(4), ORD('H'), Fg, Bg);
  ok := WinBatch.TextCell(SHORT(0), SHORT(5), ORD(' '), Fg, Bg);
  ok := WinBatch.TextCell(SHORT(0), SHORT(6), ORD('O'), Fg, Bg);
  ok := WinBatch.TextCell(SHORT(0), SHORT(7), ORD('K'), Fg, Bg);
  ok := WinBatch.Submit()
END QueueDemoGrid;

PROCEDURE StrEq(a, b: ARRAY OF SHORTCHAR): BOOLEAN;
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (a[i] = b[i]) & (a[i] # 0X) DO INC(i) END;
  RETURN a[i] = b[i]
END StrEq;

PROCEDURE BuildWindow;
  VAR logText: ARRAY 4096 OF SHORTCHAR;
      titleText: ARRAY 256 OF SHORTCHAR;
BEGIN
  WinView.GetTitle(titleText, 256);
  WinSpec.Begin(titleText);
  Log.GetText(logText, 4096);
  WinSpec.OpenStack(-1);
  WinSpec.OpenRow(-1);
  WinSpec.AddButton("run_factorial", "Factorial 20", "run_factorial");
  WinSpec.AddButton("clear_log", "Clear", "clear_log");
  WinSpec.AddButton("open_doc", "Open Document", "open_doc");
  WinSpec.CloseContainer;
  WinSpec.AddText("Frame text-grid pane:");
  WinSpec.AddTextGrid("demo_grid", "", SHORT(40), SHORT(8));
  Graph.AddPane;
  WinSpec.AddTextarea("log", "Log", logText, 1);
  WinSpec.CloseContainer
END BuildWindow;

PROCEDURE OnOpenDoc(name, payload: ARRAY OF SHORTCHAR);
  VAR id: INTEGER; ok: INTSHORT;
      spec: ARRAY 1024 OF SHORTCHAR;
      title: ARRAY 64 OF SHORTCHAR;
BEGIN
  (* Minimal demo of HostWindows.OpenChildWindow: create an MDI child
     under the frame (parent id = 1) carrying a small spec with a button
     and a surface pane. The button event names are unique per-doc-id
     in production code; here we just prove the API works. *)
  title := "Document";
  spec := "{""type"":""window"",""title"":""Document"",""body"":{""type"":""stack"",""children"":[{""type"":""text"",""text"":""Opened from CP""},{""type"":""surface"",""id"":""doc_surface"",""scrollBars"":""both"",""width"":400,""height"":300}]}}";
  ok := HostWindows.OpenChildWindow(1, title, spec, id);
  IF ok # 0 THEN
    Log.String("Opened MDI child id=");
    (* Log.Int does not exist; print id as decimal via Log primitives *)
    Log.String("ok"); Log.Ln
  ELSE
    Log.String("OpenChildWindow failed"); Log.Ln
  END
END OnOpenDoc;

PROCEDURE OnClose*(name, payload: ARRAY OF SHORTCHAR);
BEGIN
END OnClose;

PROCEDURE Run*;
  VAR ok: INTSHORT;
      name: ARRAY 256 OF SHORTCHAR;
      payload: ARRAY 4096 OF SHORTCHAR;
BEGIN
  gridPane := 0;
  Log.Open;
  Log.String("NewCP ready."); Log.Ln;
  WinView.SetTitle("NewCP");
  WinView.SetRenderer(BuildWindow);
  WinView.Render;
  ok := WinFrame.ResolvePaneId("demo_grid", gridPane);
  IF ok # 0 THEN
    QueueDemoGrid
  END;
  Graph.Init;
  LOOP
    ok := HostWindows.WaitNamedEvent(name, payload, -1);
    IF ok # 0 THEN
      IF StrEq(name, "__close_requested") OR StrEq(name, "__host_stopping") THEN
        EXIT
      ELSIF StrEq(name, "run_factorial") THEN
        Factorial.OnRun(name, payload)
      ELSIF StrEq(name, "clear_log") THEN
        Log.OnClear(name, payload);
        WinView.Render
      ELSIF StrEq(name, "open_doc") THEN
        OnOpenDoc(name, payload);
        WinView.Render
      END
    END
  END
END Run;

END App.
