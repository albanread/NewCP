DEFINITION MODULE HostFrame;

(* Interface only. The actual implementations are Rust-hosted exports
   registered by hostframe_module_artifact() in newcp-runtime. *)

PROCEDURE TextGridWriteCell*(paneId: INTEGER;
                             row, col: INTSHORT;
                             codepoint: INTEGER;
                             fg, bg: INTEGER): INTSHORT;

PROCEDURE TextGridClearRegion*(paneId: INTEGER;
                               row, col, width, height: INTSHORT;
                               fillCodepoint: INTEGER;
                               fg, bg: INTEGER): INTSHORT;

PROCEDURE DrawLine*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1, halfThickness: REAL;
                    r, g, b, a: REAL): INTSHORT;

PROCEDURE FillRect*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1, cornerRadius: REAL;
                    r, g, b, a: REAL): INTSHORT;

PROCEDURE StrokeRect*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      x0, y0, x1, y1, halfThickness, cornerRadius: REAL;
                      r, g, b, a: REAL): INTSHORT;

PROCEDURE FillCircle*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      cx, cy, radius: REAL;
                      r, g, b, a: REAL): INTSHORT;

PROCEDURE FillOval*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1: REAL;
                    r, g, b, a: REAL): INTSHORT;

PROCEDURE StrokeCircle*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                        clearR, clearG, clearB, clearA: REAL;
                        cx, cy, radius, halfThickness: REAL;
                        r, g, b, a: REAL): INTSHORT;

PROCEDURE StrokeOval*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      x0, y0, x1, y1, halfThickness: REAL;
                      r, g, b, a: REAL): INTSHORT;

PROCEDURE DrawArc*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                   clearR, clearG, clearB, clearA: REAL;
                   cx, cy, radius, halfThickness, rotationRad, halfApertureRad: REAL;
                   r, g, b, a: REAL): INTSHORT;

PROCEDURE DrawPath*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    pointsXY: ARRAY OF REAL;
                    pointCount, closed: INTSHORT;
                    halfThickness: REAL;
                    r, g, b, a: REAL): INTSHORT;

PROCEDURE DrawText*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    text: ARRAY OF SHORTCHAR;
                    originX, originY: REAL;
                    r, g, b, a: REAL): INTSHORT;

PROCEDURE DrawTextRun*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                       clearR, clearG, clearB, clearA: REAL;
                       text: ARRAY OF SHORTCHAR;
                       originX, originY: REAL;
                       r, g, b, a: REAL): INTSHORT;

PROCEDURE MeasureTextRun*(text: ARRAY OF SHORTCHAR;
                          VAR width, height: REAL;
                          VAR charCount: INTEGER): INTSHORT;

PROCEDURE CharIndexAtPoint*(text: ARRAY OF SHORTCHAR;
                            originX, originY, x, y: REAL;
                            VAR charIndex: INTEGER): INTSHORT;

PROCEDURE PointAtCharIndex*(text: ARRAY OF SHORTCHAR;
                            originX, originY: REAL;
                            charIndex: INTEGER;
                            VAR x, y: REAL): INTSHORT;

END HostFrame.
