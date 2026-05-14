MODULE HostPorts;

(* BlackBox-faithful concrete Ports implementation backed by iGui.

   First slice — provides:
     - HostPort: a Ports.Port bound to an iGui child window
     - HostRider: a Ports.Rider whose paint methods call through
       HostPortsSys to iGui's surface-batch primitives
     - Init(): open the iGui frame + a default child, return a
       ready-to-paint HostPort

   The Rider implements just the methods Pane.Restore drives
   today: DrawRect, DrawLine, DrawString.  The remaining methods
   (DrawOval, DrawPath, MarkRect, SaveRect/RestoreRect, scroll,
   cursor, hit-test, SHORTCHAR variants) are concrete stubs —
   they trap or return safe zeros so a Restore that walks past
   them doesn't break, and they're slot-filled in follow-up
   slices.

   The HostXxxSys layer pattern: HostPorts imports HostPortsSys
   imports iGui.  HostPortsSys converts Ports.Color (packed
   integer) to the four 0..1 reals iGui wants and forwards.
   HostPorts only deals with CP coordinate units and integer
   colors — never sees an iGui call directly. *)

IMPORT Ports, Fonts, HostPortsSys;

CONST
    (** Default font face / size when a DrawString call arrives
        with a NIL font.  Mirrors BB-faithful "Segoe UI 12" — the
        recording rider and the iGui DrawTextRun accept these. *)
    defaultFamily = "Segoe UI";
    defaultSize   = 12.0;

TYPE
    HostPortDesc* = RECORD (Ports.PortDesc)
        (** iGui child window this port paints into. *)
        childId-: INTEGER;
        (** Last-known size in DIPs.  Updated lazily; the iGui
            window's actual size is the source of truth. *)
        widthDip-, heightDip-: INTEGER
    END;
    HostPort* = POINTER TO HostPortDesc;

    HostRiderDesc* = RECORD (Ports.RiderDesc)
        base-: HostPort;
        (** Sub-rectangle for SetRect clipping.  Init'd to the
            full surface; SetRect narrows it. *)
        clipL, clipT, clipR, clipB: INTEGER;
        (** Pan offset applied to coords before passing to iGui. *)
        dx, dy: INTEGER
    END;
    HostRider* = POINTER TO HostRiderDesc;


(* ─── HostPort: concrete Ports.Port ──────────────────────────── *)

PROCEDURE (p: HostPortDesc) GetSize* (OUT w, h: INTEGER);
BEGIN
    w := p.widthDip;
    h := p.heightDip
END GetSize;

PROCEDURE (p: HostPortDesc) SetSize* (w, h: INTEGER);
BEGIN
    (* iGui owns the window's actual size — this is a cache hint
       only.  A real impl would call into iGui to resize. *)
    p.widthDip := w;
    p.heightDip := h
END SetSize;

PROCEDURE (p: HostPortDesc) NewRider* (): Ports.Rider;
    VAR rd: HostRider;
BEGIN
    NEW(rd);
    rd.base := p(HostPort);
    rd.clipL := 0; rd.clipT := 0;
    rd.clipR := p.widthDip; rd.clipB := p.heightDip;
    rd.dx := 0; rd.dy := 0;
    RETURN rd
END NewRider;

PROCEDURE (p: HostPortDesc) OpenBuffer* (l, t, r, b: INTEGER);
BEGIN
    (* Single-buffered: no per-Restore offscreen.  iGui's swap
       chain double-buffers the final composite. *)
END OpenBuffer;

PROCEDURE (p: HostPortDesc) CloseBuffer* ();
BEGIN END CloseBuffer;


(* ─── HostRider: concrete Ports.Rider ────────────────────────── *)

PROCEDURE (rd: HostRiderDesc) SetRect* (l, t, r, b: INTEGER);
BEGIN
    rd.clipL := l; rd.clipT := t;
    rd.clipR := r; rd.clipB := b
END SetRect;

PROCEDURE (rd: HostRiderDesc) GetRect* (OUT l, t, r, b: INTEGER);
BEGIN
    l := rd.clipL; t := rd.clipT;
    r := rd.clipR; b := rd.clipB
END GetRect;

PROCEDURE (rd: HostRiderDesc) Base* (): Ports.Port;
BEGIN
    RETURN rd.base
END Base;

PROCEDURE (rd: HostRiderDesc) Move* (dx, dy: INTEGER);
BEGIN
    rd.dx := rd.dx + dx;
    rd.dy := rd.dy + dy
END Move;

PROCEDURE (rd: HostRiderDesc) SaveRect*
    (l, t, r, b: INTEGER; VAR res: INTEGER);
BEGIN
    (* iGui's swap chain manages backing; no rider-level save. *)
    res := 0
END SaveRect;

PROCEDURE (rd: HostRiderDesc) RestoreRect*
    (l, t, r, b: INTEGER; dispose: BOOLEAN);
BEGIN END RestoreRect;

PROCEDURE (rd: HostRiderDesc) DrawRect*
    (l, t, r, b, s: INTEGER; col: Ports.Color);
BEGIN
    IF s = Ports.fill THEN
        HostPortsSys.FillRect(l, t, r, b, 0.0, col)
    ELSE
        (* Half-thickness in DIPs ~= s/2.  Ports passes s in
           device units already (Ports.Frame's DIV-by-unit). *)
        HostPortsSys.StrokeRect(l, t, r, b, 0.0, s / 2.0, col)
    END
END DrawRect;

PROCEDURE (rd: HostRiderDesc) DrawOval*
    (l, t, r, b, s: INTEGER; col: Ports.Color);
BEGIN
    (* Deferred to the geometry-completion slice. *)
END DrawOval;

PROCEDURE (rd: HostRiderDesc) DrawLine*
    (x0, y0, x1, y1, s: INTEGER; col: Ports.Color);
BEGIN
    HostPortsSys.DrawLine(x0, y0, x1, y1, s / 2.0, col)
END DrawLine;

PROCEDURE (rd: HostRiderDesc) DrawPath*
    (IN p: ARRAY OF Ports.Point; n, s: INTEGER; col: Ports.Color;
     path: INTEGER);
BEGIN END DrawPath;

PROCEDURE (rd: HostRiderDesc) MarkRect*
    (l, t, r, b, s, mode: INTEGER; show: BOOLEAN);
BEGIN END MarkRect;

PROCEDURE (rd: HostRiderDesc) Scroll* (dx, dy: INTEGER);
BEGIN END Scroll;

PROCEDURE (rd: HostRiderDesc) SetCursor* (cursor: INTEGER);
BEGIN END SetCursor;

PROCEDURE (rd: HostRiderDesc) Input*
    (OUT x, y: INTEGER; OUT modifiers: SET; OUT isDown: BOOLEAN);
BEGIN
    x := 0; y := 0; modifiers := {}; isDown := FALSE
END Input;


(* -- DrawString helpers --------------------------------------- *)

(** Narrow a CHAR string into a SHORTCHAR scratch buffer.
    ASCII / Latin-1 round-trip losslessly; out-of-range
    codepoints become "?". *)
PROCEDURE Narrow (IN src: ARRAY OF CHAR; VAR dst: ARRAY OF SHORTCHAR);
    VAR i, c: INTEGER;
BEGIN
    i := 0;
    WHILE (i < LEN(src) - 1) & (i < LEN(dst) - 1) & (src[i] # 0X) DO
        c := ORD(src[i]);
        IF c > 0FFH THEN c := ORD("?") END;
        dst[i] := SHORT(CHR(c));
        INC(i)
    END;
    dst[i] := 0X
END Narrow;

PROCEDURE FontParams (font: Fonts.Font;
                      VAR family: ARRAY OF SHORTCHAR;
                      VAR size: REAL;
                      VAR weight: INTSHORT);
    VAR familyCh: ARRAY 64 OF CHAR;
        i: INTEGER;
BEGIN
    IF font # NIL THEN
        (* Fonts.FontDesc.typeface is a Fonts.Typeface =
           ARRAY 32 OF CHAR.  Copy + narrow. *)
        i := 0;
        WHILE (i < LEN(font.typeface) - 1) & (font.typeface[i] # 0X) DO
            familyCh[i] := font.typeface[i];
            INC(i)
        END;
        familyCh[i] := 0X;
        Narrow(familyCh, family);
        size := font.size / 1000.0;     (* BB sub-mm -> DIP, rough *)
        IF size < 1.0 THEN size := defaultSize END;
        weight := SHORT(font.weight)
    ELSE
        Narrow(defaultFamily, family);
        size := defaultSize;
        weight := 400         (* normal *)
    END
END FontParams;

PROCEDURE (rd: HostRiderDesc) DrawString*
    (x, y: INTEGER; col: Ports.Color;
     IN s: ARRAY OF CHAR; font: Fonts.Font);
    VAR text: ARRAY 1024 OF SHORTCHAR;
        family: ARRAY 64 OF SHORTCHAR;
        locale: ARRAY 16 OF SHORTCHAR;
        size: REAL;
        weight: INTSHORT;
BEGIN
    Narrow(s, text);
    FontParams(font, family, size, weight);
    locale[0] := 0X;
    HostPortsSys.DrawTextRun(text, x, y, size, family,
                             weight, 0, 5, locale, -1.0, 0, 0, col)
END DrawString;

PROCEDURE (rd: HostRiderDesc) DrawSpace*
    (x, y, w: INTEGER; col: Ports.Color; font: Fonts.Font);
BEGIN
    (* "Render w DIPs of blank with this font" — interesting only
       once trailing-space rendering matters. *)
END DrawSpace;

PROCEDURE (rd: HostRiderDesc) DrawSString*
    (x, y: INTEGER; col: Ports.Color;
     IN s: ARRAY OF SHORTCHAR; font: Fonts.Font);
    VAR family: ARRAY 64 OF SHORTCHAR;
        locale: ARRAY 16 OF SHORTCHAR;
        size: REAL;
        weight: INTSHORT;
BEGIN
    FontParams(font, family, size, weight);
    locale[0] := 0X;
    HostPortsSys.DrawTextRun(s, x, y, size, family,
                             weight, 0, 5, locale, -1.0, 0, 0, col)
END DrawSString;

(* Hit-test methods — return 0 until DirectWrite layout caches
   land via iGui.MeasureTextRun. *)
PROCEDURE (rd: HostRiderDesc) CharIndex*
    (x, pos: INTEGER; IN s: ARRAY OF CHAR;
     font: Fonts.Font): INTEGER;
BEGIN RETURN 0 END CharIndex;

PROCEDURE (rd: HostRiderDesc) CharPos*
    (x, index: INTEGER; IN s: ARRAY OF CHAR;
     font: Fonts.Font): INTEGER;
BEGIN RETURN 0 END CharPos;

PROCEDURE (rd: HostRiderDesc) SCharIndex*
    (x, pos: INTEGER; IN s: ARRAY OF SHORTCHAR;
     font: Fonts.Font): INTEGER;
BEGIN RETURN 0 END SCharIndex;

PROCEDURE (rd: HostRiderDesc) SCharPos*
    (x, index: INTEGER; IN s: ARRAY OF SHORTCHAR;
     font: Fonts.Font): INTEGER;
BEGIN RETURN 0 END SCharPos;


(* ─── Lifecycle ──────────────────────────────────────────────── *)

(** Open an iGui child window and wrap it in a fresh HostPort.
    `title` is the child's window-title label; `childId` is set
    to the iGui-assigned id on success.  Returns the HostPort on
    success, NIL if iGui couldn't allocate the child (frame not
    running or MDI client missing).

    The returned port's `unit` is unset — the caller installs it
    via `port.Init(unit, FALSE)` before painting. *)
PROCEDURE NewPort* (IN title: ARRAY OF SHORTCHAR;
                    OUT childId: INTEGER): HostPort;
    VAR p: HostPort; ok: INTSHORT;
BEGIN
    childId := 0;
    ok := HostPortsSys.OpenChild(title, childId);
    IF ok = 0 THEN RETURN NIL END;
    NEW(p);
    p.childId := childId;
    p.widthDip := 800;       (* iGui's default child size *)
    p.heightDip := 600;
    RETURN p
END NewPort;

(** Bracket a paint sequence on `port`'s child window.  Caller
    uses pattern:
        HostPorts.BeginPaint(port);
        port.NewRider() ... DrawRect / DrawString / ...
        HostPorts.SubmitPaint() *)
PROCEDURE BeginPaint* (p: HostPort);
BEGIN
    ASSERT(p # NIL, 20);
    HostPortsSys.BeginBatch(p.childId)
END BeginPaint;

PROCEDURE SubmitPaint* (): INTSHORT;
BEGIN
    RETURN HostPortsSys.SubmitBatch()
END SubmitPaint;

(** Tear down a HostPort's iGui child.  After this the port's
    `childId` is invalid; the port itself can be GC'd. *)
PROCEDURE Close* (p: HostPort);
    VAR ok: INTSHORT;
BEGIN
    IF p # NIL THEN
        ok := HostPortsSys.CloseChild(p.childId)
    END
END Close;

END HostPorts.
