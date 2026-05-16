MODULE HostPortsSys;

(* CP-shaped wrapper over the iGui surface-batch primitives.

   This is the Sys layer between HostPorts (BlackBox-faithful
   concrete Ports.Rider impl) and iGui (the native runtime).  Only
   this module imports iGui; HostPorts and Ports stay free of
   iGui specifics so the BlackBox-faithful surface stays
   recognizable.

   Coordinate convention: inputs are device-independent pixels
   (DIPs).  HostPorts converts BlackBox sub-millimeter coords to
   DIPs at the boundary before calling this layer.

   Color convention: inputs are 0xAARRGGBB packed in an INTEGER
   (BlackBox Ports.Color shape).  We unpack to four 0..1 reals on
   the way to iGui, which uses sRGB float channels.

   These wrappers do NOT manage iGui batches.  A caller wanting
   to actually paint needs to bracket a sequence of these calls
   with iGui.BeginBatch / iGui.SubmitBatch.  Keeping batch
   management above this layer lets HostPorts coalesce per-Restore
   draw calls into one submission.
*)

IMPORT iGui;

CONST
    (** Channel multiplier — Ports.Color packs each channel into
        a byte; iGui wants 0..1 reals. *)
    byteMax = 255.0;

(** Unpack Ports.Color (0xAARRGGBB) into 4 reals in [0, 1]. *)
PROCEDURE UnpackColor* (col: INTEGER;
                        OUT r, g, b, a: REAL);
BEGIN
    r := (col MOD 256) / byteMax;
    b := ((col DIV 65536) MOD 256) / byteMax;
    g := ((col DIV 256) MOD 256) / byteMax;
    (* Ports.Color top byte is alpha-or-zero; treat 0 as opaque
       since BB code never sets it explicitly. *)
    a := (col DIV 16777216) MOD 256 / byteMax;
    IF a = 0 THEN a := 1.0 END
END UnpackColor;

(** Filled-rect primitive.  `cornerRadius` 0 = sharp corners. *)
PROCEDURE FillRect* (x0, y0, x1, y1: REAL;
                     cornerRadius: REAL;
                     col: INTEGER);
    VAR r, g, b, a: REAL;
BEGIN
    UnpackColor(col, r, g, b, a);
    iGui.EmitFillRect(x0, y0, x1, y1, cornerRadius, r, g, b, a)
END FillRect;

(** Stroked-rect primitive.  `halfThickness` 0.5 = 1-DIP line. *)
PROCEDURE StrokeRect* (x0, y0, x1, y1: REAL;
                       cornerRadius, halfThickness: REAL;
                       col: INTEGER);
    VAR r, g, b, a: REAL;
BEGIN
    UnpackColor(col, r, g, b, a);
    iGui.EmitStrokeRect(x0, y0, x1, y1, cornerRadius, halfThickness, r, g, b, a)
END StrokeRect;

(** Straight-line primitive. *)
PROCEDURE DrawLine* (x0, y0, x1, y1: REAL;
                     halfThickness: REAL;
                     col: INTEGER);
    VAR r, g, b, a: REAL;
BEGIN
    UnpackColor(col, r, g, b, a);
    iGui.EmitDrawLine(x0, y0, x1, y1, halfThickness, r, g, b, a)
END DrawLine;

(** Text-run primitive.  `text` is a SHORTCHAR buffer (CHAR→ASCII
    narrowing happens in HostPorts).  Family / locale empty
    strings default to "Segoe UI" / "en-us" on the iGui side.

    `weight` is DirectWrite-style 100..900 (400 = normal, 700 =
    bold).  `style` / `stretch` pick from iGui.Fs*/Fw* tags.
    `alignment` / `trimming` from iGui.Align*/Trim* tags.

    Defaults the caller can pass: weight=400, style=0, stretch=5,
    locale="", maxWidth=-1, alignment=0, trimming=0. *)
PROCEDURE DrawTextRun* (IN text: ARRAY OF SHORTCHAR;
                        x, y, fontSize: REAL;
                        IN family: ARRAY OF SHORTCHAR;
                        weight, style, stretch: INTSHORT;
                        IN locale: ARRAY OF SHORTCHAR;
                        maxWidth: REAL;
                        alignment, trimming: INTSHORT;
                        col: INTEGER);
    VAR r, g, b, a: REAL;
BEGIN
    UnpackColor(col, r, g, b, a);
    iGui.EmitDrawTextRun(text, x, y, fontSize, family,
                         weight, style, stretch, locale,
                         maxWidth, alignment, trimming, r, g, b, a)
END DrawTextRun;

(** Oval primitive.  s = Ports.fill (-1) means filled; s > 0 is
    halfThickness in device dots. *)
PROCEDURE DrawOval* (x0, y0, x1, y1: REAL; s: INTEGER; col: INTEGER);
    VAR r, g, b, a: REAL;
BEGIN
    UnpackColor(col, r, g, b, a);
    IF s = -1 THEN
        iGui.EmitFillOval(x0, y0, x1, y1, r, g, b, a)
    ELSE
        iGui.EmitStrokeOval(x0, y0, x1, y1, s / 2.0, r, g, b, a)
    END
END DrawOval;

(** MarkRect overlay.  show = FALSE is a no-op (immediate-mode: next
    repaint erases naturally).  mode maps Ports.invert/hilite/dimNN
    to a translucent overlay fill. *)
PROCEDURE MarkRect* (x0, y0, x1, y1: REAL; mode: INTEGER; show: BOOLEAN);
BEGIN
    IF ~show THEN RETURN END;
    CASE mode OF
      0: iGui.EmitFillRect(x0, y0, x1, y1, 0.0, 1.0, 1.0, 1.0, 0.5)  (* invert *)
    | 1: iGui.EmitFillRect(x0, y0, x1, y1, 0.0, 0.2, 0.5, 0.9, 0.3)  (* hilite *)
    | 2: iGui.EmitFillRect(x0, y0, x1, y1, 0.0, 0.0, 0.0, 0.0, 0.25) (* dim25  *)
    | 3: iGui.EmitFillRect(x0, y0, x1, y1, 0.0, 0.0, 0.0, 0.0, 0.5)  (* dim50  *)
    | 4: iGui.EmitFillRect(x0, y0, x1, y1, 0.0, 0.0, 0.0, 0.0, 0.75) (* dim75  *)
    ELSE END
END MarkRect;

(** Hit-test: character index at a layout-relative x position.
    `x` is pos - stringLeft (relative to the layout origin).
    `y` = 0.0 for single-line queries.  Returns 1 on success. *)
PROCEDURE CharIndexAtPoint* (childId: INTEGER;
                             IN text: ARRAY OF SHORTCHAR;
                             fontSize: REAL;
                             IN family: ARRAY OF SHORTCHAR;
                             weight, style, stretch: INTSHORT;
                             IN locale: ARRAY OF SHORTCHAR;
                             x, y: REAL;
                             VAR charIndex: INTEGER;
                             VAR isInside, isTrailing: INTSHORT): INTSHORT;
BEGIN
    RETURN iGui.CharIndexAtPoint(childId, text, fontSize, family,
                                 weight, style, stretch, locale,
                                 x, y, charIndex, isInside, isTrailing)
END CharIndexAtPoint;

(** Hit-test: pixel x of the left edge of character `charIndex`.
    Returns 1 on success; on success `x` is the DIP offset from
    the layout origin (i.e. add your string's left edge for screen
    coords). *)
PROCEDURE PointAtCharIndex* (childId: INTEGER;
                              IN text: ARRAY OF SHORTCHAR;
                              fontSize: REAL;
                              IN family: ARRAY OF SHORTCHAR;
                              weight, style: INTSHORT;
                              charIndex: INTEGER;
                              VAR x, y, height: REAL): INTSHORT;
BEGIN
    RETURN iGui.PointAtCharIndex(childId, text, fontSize, family,
                                  weight, style, charIndex, x, y, height)
END PointAtCharIndex;

(** Solid-color clear for a child's full bitmap surface. *)
PROCEDURE Clear* (col: INTEGER);
    VAR r, g, b, a: REAL;
BEGIN
    UnpackColor(col, r, g, b, a);
    iGui.EmitClear(r, g, b, a)
END Clear;

(** Bracket a sequence of paint calls.  Returns the SubmitBatch
    result — 1 on success, 0 on failure. *)
PROCEDURE BeginBatch* (childId: INTEGER);
BEGIN
    iGui.BeginBatch(childId)
END BeginBatch;

PROCEDURE SubmitBatch* (): INTSHORT;
BEGIN
    RETURN iGui.SubmitBatch()
END SubmitBatch;

(** Open / close an iGui child window.  Returns 1 on success,
    0 on failure (frame not running, MDI client missing, etc). *)
PROCEDURE OpenChild* (IN title: ARRAY OF SHORTCHAR;
                      VAR childId: INTEGER): INTSHORT;
BEGIN
    RETURN iGui.OpenChild(title, childId)
END OpenChild;

PROCEDURE CloseChild* (childId: INTEGER): INTSHORT;
BEGIN
    RETURN iGui.CloseChild(childId)
END CloseChild;

END HostPortsSys.
