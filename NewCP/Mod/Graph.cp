MODULE Graph;

IMPORT WinBatch, WinFrame, WinSpec;

CONST
  NodeId = "demo_surface";

VAR
  pane: INTEGER;

PROCEDURE QueueSurfaceDemo;
  VAR ok: INTSHORT;
      path: ARRAY 10 OF REAL;
BEGIN
  IF pane = 0 THEN
    RETURN
  END;
  path[0] := 40.0;  path[1] := 56.0;
  path[2] := 72.0;  path[3] := 28.0;
  path[4] := 120.0; path[5] := 44.0;
  path[6] := 144.0; path[7] := 92.0;
  path[8] := 94.0;  path[9] := 118.0;
  ok := WinBatch.Begin(pane, 1, SHORT(0));
  ok := WinBatch.FillRect(WinFrame.BufPersistent, SHORT(0), SHORT(1),
    0.07, 0.08, 0.11, 1.0,
    8.0, 8.0, 312.0, 172.0, 14.0,
    0.10, 0.12, 0.17, 1.0);
  ok := WinBatch.FillOval(WinFrame.BufPersistent, SHORT(0), SHORT(0),
    0.0, 0.0, 0.0, 0.0,
    34.0, 26.0, 154.0, 120.0,
    0.19, 0.23, 0.33, 0.88);
  ok := WinBatch.StrokeRect(WinFrame.BufPersistent, SHORT(0), SHORT(0),
    0.0, 0.0, 0.0, 0.0,
    18.0, 18.0, 302.0, 162.0, 1.5, 18.0,
    0.38, 0.71, 0.95, 1.0);
  ok := WinBatch.StrokeOval(WinFrame.BufPersistent, SHORT(0), SHORT(0),
    0.0, 0.0, 0.0, 0.0,
    168.0, 34.0, 284.0, 124.0, 2.0,
    0.98, 0.82, 0.42, 1.0);
  ok := WinBatch.FillCircle(WinFrame.BufPersistent, SHORT(0), SHORT(0),
    0.0, 0.0, 0.0, 0.0,
    88.0, 88.0, 28.0,
    0.95, 0.53, 0.22, 0.95);
  ok := WinBatch.StrokeCircle(WinFrame.BufPersistent, SHORT(0), SHORT(0),
    0.0, 0.0, 0.0, 0.0,
    228.0, 92.0, 34.0, 2.5,
    0.96, 0.92, 0.62, 1.0);
  ok := WinBatch.DrawLine(WinFrame.BufPersistent, SHORT(0), SHORT(0),
    0.0, 0.0, 0.0, 0.0,
    36.0, 136.0, 286.0, 42.0, 2.0,
    0.38, 0.90, 0.56, 1.0);
  ok := WinBatch.DrawArc(WinFrame.BufPersistent, SHORT(0), SHORT(0),
    0.0, 0.0, 0.0, 0.0,
    226.0, 92.0, 48.0, 3.0, 0.1, 1.8,
    0.95, 0.40, 0.68, 1.0);
  ok := WinBatch.DrawPath(WinFrame.BufPersistent, SHORT(0), SHORT(0),
    0.0, 0.0, 0.0, 0.0,
    path, SHORT(5), SHORT(1), 2.0,
    0.55, 0.88, 0.97, 1.0);
  ok := WinBatch.DrawTextRun(WinFrame.BufPersistent, SHORT(0), SHORT(0),
    0.0, 0.0, 0.0, 0.0,
    "SURFACE OK",
    116.0, 84.0,
    0.95, 0.97, 1.0, 1.0);
  ok := WinBatch.DrawText(WinFrame.BufPersistent, SHORT(0), SHORT(0),
    0.0, 0.0, 0.0, 0.0,
    "mvc primitives",
    104.0, 112.0,
    0.72, 0.78, 0.88, 1.0);
  ok := WinBatch.Submit()
END QueueSurfaceDemo;

PROCEDURE AddPane*;
BEGIN
  WinSpec.AddText("Surface pane:");
  WinSpec.AddSurface(NodeId, "", SHORT(320), SHORT(180))
END AddPane;

PROCEDURE Init*;
  VAR ok: INTSHORT;
BEGIN
  ok := WinFrame.ResolvePaneId(NodeId, pane);
  IF ok # 0 THEN
    QueueSurfaceDemo
  END
END Init;

BEGIN
  pane := 0
END Graph.