MODULE WinView;

IMPORT HostWindows, WinSpec;

CONST
  SpecMax = 16384;
  TitleMax = 256;

TYPE
  RenderProc* = PROCEDURE;

VAR
  title*: ARRAY TitleMax OF SHORTCHAR;
  spec: ARRAY SpecMax OF SHORTCHAR;
  renderer: RenderProc;
  hasRenderer: BOOLEAN;

PROCEDURE CopyStr(src: ARRAY OF SHORTCHAR; VAR dst: ARRAY OF SHORTCHAR);
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (src[i] # 0X) & (i < TitleMax - 1) DO
    dst[i] := src[i];
    INC(i)
  END;
  dst[i] := 0X
END CopyStr;

PROCEDURE SetRenderer*(p: RenderProc);
BEGIN
  renderer := p;
  hasRenderer := TRUE
END SetRenderer;

PROCEDURE SetTitle*(nextTitle: ARRAY OF SHORTCHAR);
BEGIN
  CopyStr(nextTitle, title)
END SetTitle;

PROCEDURE GetTitle*(VAR dst: ARRAY OF SHORTCHAR; cap: INTEGER);
  VAR i, limit: INTEGER;
BEGIN
  i := 0;
  limit := cap - 1;
  IF limit < 0 THEN
    RETURN
  END;
  WHILE (title[i] # 0X) & (i < limit) DO
    dst[i] := title[i];
    INC(i)
  END;
  dst[i] := 0X
END GetTitle;

PROCEDURE Render*;
BEGIN
  IF ~hasRenderer THEN RETURN END;
  renderer();
  IF WinSpec.GetSpec(spec) # 0 THEN
    HostWindows.PublishUi(spec)
  END
END Render;

BEGIN
  hasRenderer := FALSE;
  spec[0] := 0X;
  CopyStr("NewCP", title)
END WinView.