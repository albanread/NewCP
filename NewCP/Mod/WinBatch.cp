MODULE WinBatch;

PROCEDURE Begin*(paneId, sequence: INTEGER; flags: INTSHORT): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END Begin;

PROCEDURE Clear*(bufMode: INTSHORT; r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END Clear;

PROCEDURE PushClipRect*(x0, y0, x1, y1: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END PushClipRect;

PROCEDURE PopClipRect*(): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END PopClipRect;

PROCEDURE PushOffset*(dx, dy: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END PushOffset;

PROCEDURE PopOffset*(): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END PopOffset;

PROCEDURE TextCell*(row, col: INTSHORT;
                    codepoint, fg, bg: INTEGER): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END TextCell;

PROCEDURE DrawLine*(bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1, halfThickness: REAL;
                    r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END DrawLine;

PROCEDURE DrawText*(bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    text: ARRAY OF SHORTCHAR;
                    originX, originY: REAL;
                    r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END DrawText;

PROCEDURE DrawTextRun*(bufMode, blendMode, clearBefore: INTSHORT;
                       clearR, clearG, clearB, clearA: REAL;
                       text: ARRAY OF SHORTCHAR;
                       originX, originY: REAL;
                       r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END DrawTextRun;

PROCEDURE FillRect*(bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1, cornerRadius: REAL;
                    r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END FillRect;

PROCEDURE StrokeRect*(bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      x0, y0, x1, y1, halfThickness, cornerRadius: REAL;
                      r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END StrokeRect;

PROCEDURE FillCircle*(bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      cx, cy, radius: REAL;
                      r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END FillCircle;

PROCEDURE FillOval*(bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1: REAL;
                    r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END FillOval;

PROCEDURE StrokeCircle*(bufMode, blendMode, clearBefore: INTSHORT;
                        clearR, clearG, clearB, clearA: REAL;
                        cx, cy, radius, halfThickness: REAL;
                        r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END StrokeCircle;

PROCEDURE StrokeOval*(bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      x0, y0, x1, y1, halfThickness: REAL;
                      r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END StrokeOval;

PROCEDURE DrawArc*(bufMode, blendMode, clearBefore: INTSHORT;
                   clearR, clearG, clearB, clearA: REAL;
                   cx, cy, radius, halfThickness, rotationRad, halfApertureRad: REAL;
                   r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END DrawArc;

PROCEDURE DrawPath*(bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    pointsXY: ARRAY OF REAL;
                    pointCount, closed: INTSHORT;
                    halfThickness: REAL;
                    r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END DrawPath;

PROCEDURE MarkRect*(mode: INTSHORT; x0, y0, x1, y1: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END MarkRect;

PROCEDURE Caret*(x0, y0, x1, y1: REAL; r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END Caret;

PROCEDURE SelectionRange*(x0, y0, x1, y1: REAL; r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END SelectionRange;

PROCEDURE FocusRing*(x0, y0, x1, y1, halfThickness, cornerRadius: REAL;
                     r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END FocusRing;

PROCEDURE ScrollRect*(x0, y0, x1, y1, dx, dy: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END ScrollRect;

PROCEDURE PresentHint*(): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END PresentHint;

PROCEDURE InstallChildViewBounds*(childId: INTSHORT; x0, y0, x1, y1: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END InstallChildViewBounds;

PROCEDURE Submit*(): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END Submit;

END WinBatch.