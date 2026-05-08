DEFINITION MODULE WinBatch;

(* Interface only. The actual implementations are Rust-hosted exports
   registered by winbatch_module_artifact() in newcp-runtime. *)

PROCEDURE Begin*(paneId, sequence: INTEGER; flags: INTSHORT): INTSHORT;

PROCEDURE Clear*(bufMode: INTSHORT; r, g, b, a: REAL): INTSHORT;

PROCEDURE PushClipRect*(x0, y0, x1, y1: REAL): INTSHORT;

PROCEDURE PopClipRect*(): INTSHORT;

PROCEDURE PushOffset*(dx, dy: REAL): INTSHORT;

PROCEDURE PopOffset*(): INTSHORT;

PROCEDURE TextCell*(row, col: INTSHORT;
                    codepoint, fg, bg: INTEGER): INTSHORT;

PROCEDURE DrawLine*(bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1, halfThickness: REAL;
                    r, g, b, a: REAL): INTSHORT;

PROCEDURE DrawText*(bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    text: ARRAY OF SHORTCHAR;
                    originX, originY: REAL;
                    r, g, b, a: REAL): INTSHORT;

PROCEDURE DrawTextRun*(bufMode, blendMode, clearBefore: INTSHORT;
                       clearR, clearG, clearB, clearA: REAL;
                       text: ARRAY OF SHORTCHAR;
                       originX, originY: REAL;
                       r, g, b, a: REAL): INTSHORT;

PROCEDURE FillRect*(bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1, cornerRadius: REAL;
                    r, g, b, a: REAL): INTSHORT;

PROCEDURE StrokeRect*(bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      x0, y0, x1, y1, halfThickness, cornerRadius: REAL;
                      r, g, b, a: REAL): INTSHORT;

PROCEDURE FillCircle*(bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      cx, cy, radius: REAL;
                      r, g, b, a: REAL): INTSHORT;

PROCEDURE FillOval*(bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1: REAL;
                    r, g, b, a: REAL): INTSHORT;

PROCEDURE StrokeCircle*(bufMode, blendMode, clearBefore: INTSHORT;
                        clearR, clearG, clearB, clearA: REAL;
                        cx, cy, radius, halfThickness: REAL;
                        r, g, b, a: REAL): INTSHORT;

PROCEDURE StrokeOval*(bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      x0, y0, x1, y1, halfThickness: REAL;
                      r, g, b, a: REAL): INTSHORT;

PROCEDURE DrawArc*(bufMode, blendMode, clearBefore: INTSHORT;
                   clearR, clearG, clearB, clearA: REAL;
                   cx, cy, radius, halfThickness, rotationRad, halfApertureRad: REAL;
                   r, g, b, a: REAL): INTSHORT;

PROCEDURE DrawPath*(bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    pointsXY: ARRAY OF REAL;
                    pointCount, closed: INTSHORT;
                    halfThickness: REAL;
                    r, g, b, a: REAL): INTSHORT;

PROCEDURE MarkRect*(mode: INTSHORT; x0, y0, x1, y1: REAL): INTSHORT;

PROCEDURE Caret*(x0, y0, x1, y1: REAL; r, g, b, a: REAL): INTSHORT;

PROCEDURE SelectionRange*(x0, y0, x1, y1: REAL; r, g, b, a: REAL): INTSHORT;

PROCEDURE FocusRing*(x0, y0, x1, y1, halfThickness, cornerRadius: REAL;
                     r, g, b, a: REAL): INTSHORT;

PROCEDURE ScrollRect*(x0, y0, x1, y1, dx, dy: REAL): INTSHORT;

PROCEDURE PresentHint*(): INTSHORT;

PROCEDURE InstallChildViewBounds*(childId: INTSHORT; x0, y0, x1, y1: REAL): INTSHORT;

PROCEDURE Submit*(): INTSHORT;

END WinBatch.
