MODULE WinFrame;

CONST
  BufFrame* = 0;
  BufPersistent* = 1;

TYPE
  FrameProc* = PROCEDURE;
  PaneProc* = PROCEDURE (paneId: INTEGER);

PROCEDURE SetRenderer*(p: FrameProc);
BEGIN
END SetRenderer;

PROCEDURE RegisterPaneRenderer*(paneId: INTEGER; p: PaneProc);
BEGIN
END RegisterPaneRenderer;

PROCEDURE UnregisterPaneRenderer*(paneId: INTEGER);
BEGIN
END UnregisterPaneRenderer;

PROCEDURE FrameIndex*(): INTEGER;
BEGIN
  RETURN 0
END FrameIndex;

PROCEDURE ElapsedMs*(): INTEGER;
BEGIN
  RETURN 0
END ElapsedMs;

PROCEDURE DeltaMs*(): INTEGER;
BEGIN
  RETURN 0
END DeltaMs;

PROCEDURE ResolvePaneId*(nodeId: ARRAY OF SHORTCHAR; VAR paneId: INTEGER): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END ResolvePaneId;

PROCEDURE PaneLayout*(paneId: INTEGER;
                      VAR x, y, width, height: INTSHORT): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END PaneLayout;

PROCEDURE RequestPresent*;
BEGIN
END RequestPresent;

PROCEDURE PostPaneMsg*(paneId: INTEGER;
                       kind, detail: ARRAY OF SHORTCHAR): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END PostPaneMsg;

PROCEDURE PollPaneMsg*(paneId: INTEGER;
                       VAR kind: ARRAY OF SHORTCHAR;
                       VAR detail: ARRAY OF SHORTCHAR): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END PollPaneMsg;

END WinFrame.