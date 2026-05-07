MODULE HostFrame;

PROCEDURE TextGridWriteCell*(paneId: INTEGER;
                             row, col: INTSHORT;
                             codepoint: INTEGER;
                             fg, bg: INTEGER): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END TextGridWriteCell;

PROCEDURE TextGridClearRegion*(paneId: INTEGER;
                               row, col, width, height: INTSHORT;
                               fillCodepoint: INTEGER;
                               fg, bg: INTEGER): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END TextGridClearRegion;

PROCEDURE DrawLine*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1, halfThickness: REAL;
                    r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END DrawLine;

PROCEDURE FillRect*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1, cornerRadius: REAL;
                    r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END FillRect;

PROCEDURE StrokeRect*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      x0, y0, x1, y1, halfThickness, cornerRadius: REAL;
                      r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END StrokeRect;

PROCEDURE FillCircle*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      cx, cy, radius: REAL;
                      r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END FillCircle;

PROCEDURE FillOval*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1: REAL;
                    r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END FillOval;

PROCEDURE StrokeCircle*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                        clearR, clearG, clearB, clearA: REAL;
                        cx, cy, radius, halfThickness: REAL;
                        r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END StrokeCircle;

PROCEDURE StrokeOval*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      x0, y0, x1, y1, halfThickness: REAL;
                      r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END StrokeOval;

PROCEDURE DrawArc*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                   clearR, clearG, clearB, clearA: REAL;
                   cx, cy, radius, halfThickness, rotationRad, halfApertureRad: REAL;
                   r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END DrawArc;

PROCEDURE DrawPath*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
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

PROCEDURE DrawText*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    text: ARRAY OF SHORTCHAR;
                    originX, originY: REAL;
                    r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END DrawText;

PROCEDURE DrawTextRun*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                       clearR, clearG, clearB, clearA: REAL;
                       text: ARRAY OF SHORTCHAR;
                       originX, originY: REAL;
                       r, g, b, a: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END DrawTextRun;

PROCEDURE MeasureTextRun*(text: ARRAY OF SHORTCHAR;
                          VAR width, height: REAL;
                          VAR charCount: INTEGER): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END MeasureTextRun;

PROCEDURE CharIndexAtPoint*(text: ARRAY OF SHORTCHAR;
                            originX, originY, x, y: REAL;
                            VAR charIndex: INTEGER): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END CharIndexAtPoint;

PROCEDURE PointAtCharIndex*(text: ARRAY OF SHORTCHAR;
                            originX, originY: REAL;
                            charIndex: INTEGER;
                            VAR x, y: REAL): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END PointAtCharIndex;

END HostFrame.