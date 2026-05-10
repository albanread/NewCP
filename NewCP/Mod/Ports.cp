MODULE Ports;
(*
   NewCP `Ports` port — coordinate-system + drawing primitives.

   Direct port of BlackBox `System/Mod/Ports.odc`.  The module is mostly
   ABSTRACT: `Port` and `Rider` declare the bottom-of-stack contract that
   a host backend (Win32 GDI, DirectX, X11, …) implements; `Frame`
   wraps a `Rider` with the coordinate-translation arithmetic every
   View needs (1/36000-mm user units → unit-stepped device dots).

   Two layers:
   - `Port`  — abstract device.  Knows its `unit` (one device dot in
     user units) and whether it's a printer (`printerMode`).  Hands
     out `Rider`s anchored at the origin.
   - `Frame` — concrete user-space wrapper.  Carries the `(gx, gy)`
     origin offset relative to a parent frame and forwards every
     drawing call to its `Rider` after dividing every coordinate by
     the unit.  All `(l, t, r, b)` rectangle args, `(x0, y0, x1, y1)`
     line args, point arrays, etc. arrive in user units; the rider
     sees them in device dots.

   Drawing protocol is fixed by `Rider`'s ABSTRACT method list and
   left for the backend to implement.  This file declares every
   contract method; concrete `HostPorts` and friends will land later.

   Read-only fields: `Port.unit`, `Port.printerMode`, `Frame.unit`,
   `Frame.dot`, `Frame.rider`, `Frame.gx`, `Frame.gy` — all use the
   `-` (read-only export) modifier so clients can inspect them but
   only Frame's own methods may write.
*)

    IMPORT Fonts;

    CONST
        (** colors **)
        black*         = 00000000H;
        white*         = 00FFFFFFH;
        grey6*         = 00F0F0F0H;
        grey12*        = 00E0E0E0H;
        grey25*        = 00C0C0C0H;
        grey50*        = 00808080H;
        grey75*        = 00404040H;
        red*           = 000000FFH;
        green*         = 0000FF00H;
        blue*          = 00FF0000H;
        defaultColor*  = 01000000H;

        (** measures (1/36000 mm = unit) **)
        mm*    = 36000;
        point* = 12700;
        inch*  = 914400;

        (** size parameter for DrawRect / DrawOval / DrawLine /
            DrawPath / MarkRect — `fill` means "filled", any non-
            negative value is the stroke width in user units. *)
        fill* = -1;

        (** path parameter for DrawPath **)
        openPoly*     = 0;
        closedPoly*   = 1;
        openBezier*   = 2;
        closedBezier* = 3;

        (** modes for MarkRect **)
        invert* = 0;
        hilite* = 1;
        dim25*  = 2;
        dim50*  = 3;
        dim75*  = 4;

        (** show flag (MarkRect / SetCursor) **)
        hide* = FALSE;
        show* = TRUE;

        (** cursors **)
        arrowCursor*    = 0;
        textCursor*     = 1;
        graphicsCursor* = 2;
        tableCursor*    = 3;
        bitmapCursor*   = 4;
        refCursor*      = 5;

        (** RestoreRect dispose flag **)
        keepBuffer*    = FALSE;
        disposeBuffer* = TRUE;

        (** PageMode flag **)
        printer* = TRUE;
        screen*  = FALSE;

    TYPE
        Color* = INTEGER;

        Point* = RECORD
            x*, y*: INTEGER
        END;

        (** Abstract device port.  `unit` is one device dot in user
            units (1/36000 mm); `printerMode` true ⇒ raster output is
            non-interactive (no flicker, no scroll). *)
        PortDesc* = ABSTRACT RECORD
            unit-:        INTEGER;
            printerMode-: BOOLEAN
        END;
        Port* = POINTER TO PortDesc;

        (** Abstract drawing rider — the device-space cursor a Frame
            forwards every paint call to.  Operates in `unit` ticks,
            not user units. *)
        RiderDesc* = ABSTRACT RECORD END;
        Rider*     = POINTER TO RiderDesc;

        (** Concrete user-space frame.  `(gx, gy)` is the origin offset
            relative to the parent frame in user units; `dot` is one
            device dot rounded down to a user-unit multiple
            (`= point - point MOD unit`).  Every drawing method
            translates user coords to device coords by adding `(gx, gy)`
            then dividing by `unit`. *)
        FrameDesc* = ABSTRACT RECORD
            unit-:  INTEGER;
            dot-:   INTEGER;        (** inv: dot = point - point MOD unit **)
            rider-: Rider;
            gx-:    INTEGER;
            gy-:    INTEGER
        END;
        Frame* = POINTER TO FrameDesc;

    VAR
        background*:       Color;
        dialogBackground*: Color;


    (* -- Port -------------------------------------------------------------- *)

    PROCEDURE (p: Port) Init* (unit: INTEGER; printerMode: BOOLEAN), NEW;
    BEGIN
        ASSERT((p.unit = 0) OR (p.unit = unit), 20);
        ASSERT(unit > 0, 21);
        ASSERT((p.unit = 0) OR (p.printerMode = printerMode), 22);
        p.unit := unit;
        p.printerMode := printerMode
    END Init;

    PROCEDURE (p: Port) GetSize*    (OUT w, h: INTEGER), NEW, ABSTRACT;
    PROCEDURE (p: Port) SetSize*    (w, h: INTEGER), NEW, ABSTRACT;
    PROCEDURE (p: Port) NewRider*   (): Rider, NEW, ABSTRACT;
    PROCEDURE (p: Port) OpenBuffer* (l, t, r, b: INTEGER), NEW, ABSTRACT;
    PROCEDURE (p: Port) CloseBuffer* (), NEW, ABSTRACT;


    (* -- Rider ------------------------------------------------------------- *)

    PROCEDURE (rd: Rider) SetRect* (l, t, r, b: INTEGER), NEW, ABSTRACT;
    PROCEDURE (rd: Rider) GetRect* (OUT l, t, r, b: INTEGER), NEW, ABSTRACT;
    PROCEDURE (rd: Rider) Base*    (): Port, NEW, ABSTRACT;
    PROCEDURE (rd: Rider) Move*    (dx, dy: INTEGER), NEW, ABSTRACT;

    PROCEDURE (rd: Rider) SaveRect*
        (l, t, r, b: INTEGER; VAR res: INTEGER), NEW, ABSTRACT;

    PROCEDURE (rd: Rider) RestoreRect*
        (l, t, r, b: INTEGER; dispose: BOOLEAN), NEW, ABSTRACT;

    PROCEDURE (rd: Rider) DrawRect*
        (l, t, r, b, s: INTEGER; col: Color), NEW, ABSTRACT;

    PROCEDURE (rd: Rider) DrawOval*
        (l, t, r, b, s: INTEGER; col: Color), NEW, ABSTRACT;

    PROCEDURE (rd: Rider) DrawLine*
        (x0, y0, x1, y1, s: INTEGER; col: Color), NEW, ABSTRACT;

    PROCEDURE (rd: Rider) DrawPath*
        (IN p: ARRAY OF Point; n, s: INTEGER; col: Color; path: INTEGER),
        NEW, ABSTRACT;

    PROCEDURE (rd: Rider) MarkRect*
        (l, t, r, b, s, mode: INTEGER; show: BOOLEAN), NEW, ABSTRACT;

    PROCEDURE (rd: Rider) Scroll*    (dx, dy: INTEGER), NEW, ABSTRACT;
    PROCEDURE (rd: Rider) SetCursor* (cursor: INTEGER), NEW, ABSTRACT;

    PROCEDURE (rd: Rider) Input*
        (OUT x, y: INTEGER; OUT modifiers: SET; OUT isDown: BOOLEAN),
        NEW, ABSTRACT;

    PROCEDURE (rd: Rider) DrawString*
        (x, y: INTEGER; col: Color; IN s: ARRAY OF CHAR; font: Fonts.Font),
        NEW, ABSTRACT;

    PROCEDURE (rd: Rider) DrawSpace*
        (x, y, w: INTEGER; col: Color; font: Fonts.Font), NEW, ABSTRACT;

    PROCEDURE (rd: Rider) CharIndex*
        (x, pos: INTEGER; IN s: ARRAY OF CHAR; font: Fonts.Font): INTEGER,
        NEW, ABSTRACT;

    PROCEDURE (rd: Rider) CharPos*
        (x, index: INTEGER; IN s: ARRAY OF CHAR; font: Fonts.Font): INTEGER,
        NEW, ABSTRACT;

    PROCEDURE (rd: Rider) DrawSString*
        (x, y: INTEGER; col: Color; IN s: ARRAY OF SHORTCHAR; font: Fonts.Font),
        NEW, ABSTRACT;

    PROCEDURE (rd: Rider) SCharIndex*
        (x, pos: INTEGER; IN s: ARRAY OF SHORTCHAR; font: Fonts.Font): INTEGER,
        NEW, ABSTRACT;

    PROCEDURE (rd: Rider) SCharPos*
        (x, index: INTEGER; IN s: ARRAY OF SHORTCHAR; font: Fonts.Font): INTEGER,
        NEW, ABSTRACT;


    (* -- Frame ------------------------------------------------------------- *)

    PROCEDURE (f: Frame) ConnectTo* (p: Port), NEW, EXTENSIBLE;
        VAR w, h: INTEGER;
    BEGIN
        IF p # NIL THEN
            f.rider := p.NewRider();
            f.unit  := p.unit;
            p.GetSize(w, h);
            f.dot   := point - point MOD f.unit
        ELSE
            f.rider := NIL;
            f.unit  := 0
        END
    END ConnectTo;

    PROCEDURE (f: Frame) SetOffset* (gx, gy: INTEGER), NEW, EXTENSIBLE;
        VAR u: INTEGER;
    BEGIN
        u := f.unit;
        IF ((gx - f.gx) MOD u = 0) & ((gy - f.gy) MOD u = 0) THEN
            f.rider.Move((gx - f.gx) DIV u, (gy - f.gy) DIV u)
        END;
        f.gx := gx;
        f.gy := gy
    END SetOffset;

    PROCEDURE (f: Frame) SaveRect*
        (l, t, r, b: INTEGER; VAR res: INTEGER), NEW;
        VAR u: INTEGER;
    BEGIN
        ASSERT((l <= r) & (t <= b), 20);
        u := f.unit;
        l := (f.gx + l) DIV u; t := (f.gy + t) DIV u;
        r := (f.gx + r) DIV u; b := (f.gy + b) DIV u;
        f.rider.SaveRect(l, t, r, b, res)
    END SaveRect;

    PROCEDURE (f: Frame) RestoreRect*
        (l, t, r, b: INTEGER; dispose: BOOLEAN), NEW;
        VAR u: INTEGER;
    BEGIN
        ASSERT((l <= r) & (t <= b), 20);
        u := f.unit;
        l := (f.gx + l) DIV u; t := (f.gy + t) DIV u;
        r := (f.gx + r) DIV u; b := (f.gy + b) DIV u;
        f.rider.RestoreRect(l, t, r, b, dispose)
    END RestoreRect;

    PROCEDURE (f: Frame) DrawRect*
        (l, t, r, b, s: INTEGER; col: Color), NEW;
        VAR u: INTEGER;
    BEGIN
        ASSERT((l <= r) & (t <= b), 20);
        ASSERT(s >= fill, 21);
        u := f.unit;
        l := (f.gx + l) DIV u; t := (f.gy + t) DIV u;
        r := (f.gx + r) DIV u; b := (f.gy + b) DIV u;
        s := s DIV u;
        f.rider.DrawRect(l, t, r, b, s, col)
    END DrawRect;

    PROCEDURE (f: Frame) DrawOval*
        (l, t, r, b, s: INTEGER; col: Color), NEW;
        VAR u: INTEGER;
    BEGIN
        ASSERT((l <= r) & (t <= b), 20);
        ASSERT(s >= fill, 21);
        u := f.unit;
        l := (f.gx + l) DIV u; t := (f.gy + t) DIV u;
        r := (f.gx + r) DIV u; b := (f.gy + b) DIV u;
        s := s DIV u;
        f.rider.DrawOval(l, t, r, b, s, col)
    END DrawOval;

    PROCEDURE (f: Frame) DrawLine*
        (x0, y0, x1, y1, s: INTEGER; col: Color), NEW;
        VAR u: INTEGER;
    BEGIN
        ASSERT(s >= fill, 20);
        u := f.unit;
        x0 := (f.gx + x0) DIV u; y0 := (f.gy + y0) DIV u;
        x1 := (f.gx + x1) DIV u; y1 := (f.gy + y1) DIV u;
        s := s DIV u;
        f.rider.DrawLine(x0, y0, x1, y1, s, col)
    END DrawLine;

    PROCEDURE (f: Frame) DrawPath*
        (IN p: ARRAY OF Point; n, s: INTEGER; col: Color; path: INTEGER), NEW;

        (* Inner copy: BlackBox-faithful idiom — the outer `p` is IN
           (read-only); we copy through this value-mode formal so
           Draw can mutate the local copy in place before forwarding
           to the rider. *)
        PROCEDURE Draw (p: ARRAY OF Point);
            VAR i, u: INTEGER;
        BEGIN
            u := f.unit;
            s := s DIV u;
            i := 0;
            WHILE i # n DO
                p[i].x := (f.gx + p[i].x) DIV u;
                p[i].y := (f.gy + p[i].y) DIV u;
                INC(i)
            END;
            f.rider.DrawPath(p, n, s, col, path)
        END Draw;

    BEGIN
        ASSERT(n >= 0, 20);
        ASSERT(n <= LEN(p), 21);
        ASSERT((s # fill) OR (path = closedPoly) OR (path = closedBezier), 22);
        ASSERT(s >= fill, 23);
        Draw(p)
    END DrawPath;

    PROCEDURE (f: Frame) MarkRect*
        (l, t, r, b, s: INTEGER; mode: INTEGER; show: BOOLEAN), NEW;
        VAR u: INTEGER;
    BEGIN
        ASSERT(s >= fill, 21);
        u := f.unit;
        l := (f.gx + l) DIV u; t := (f.gy + t) DIV u;
        r := (f.gx + r) DIV u; b := (f.gy + b) DIV u;
        s := s DIV u;
        f.rider.MarkRect(l, t, r, b, s, mode, show)
    END MarkRect;

    PROCEDURE (f: Frame) Scroll* (dx, dy: INTEGER), NEW;
        VAR u: INTEGER;
    BEGIN
        u := f.unit;
        ASSERT(dx MOD u = 0, 20);
        ASSERT(dy MOD u = 0, 20);
        f.rider.Scroll(dx DIV u, dy DIV u)
    END Scroll;

    PROCEDURE (f: Frame) SetCursor* (cursor: INTEGER), NEW;
    BEGIN
        f.rider.SetCursor(cursor)
    END SetCursor;

    PROCEDURE (f: Frame) Input*
        (OUT x, y: INTEGER; OUT modifiers: SET; OUT isDown: BOOLEAN), NEW;
        VAR u: INTEGER;
    BEGIN
        f.rider.Input(x, y, modifiers, isDown);
        u := f.unit;
        x := x * u - f.gx;
        y := y * u - f.gy
    END Input;

    PROCEDURE (f: Frame) DrawString*
        (x, y: INTEGER; col: Color; IN s: ARRAY OF CHAR; font: Fonts.Font), NEW;
        VAR u: INTEGER;
    BEGIN
        u := f.unit;
        x := (f.gx + x) DIV u; y := (f.gy + y) DIV u;
        f.rider.DrawString(x, y, col, s, font)
    END DrawString;

    PROCEDURE (f: Frame) DrawSpace*
        (x, y, w: INTEGER; col: Color; font: Fonts.Font), NEW;
        VAR right, u: INTEGER;
    BEGIN
        u := f.unit;
        right := f.gx + x + w;
        x := (f.gx + x) DIV u;
        y := (f.gy + y) DIV u;
        w := (right + u - 1) DIV u - x;     (* round up; x may be truncated *)
        f.rider.DrawSpace(x, y, w, col, font)
    END DrawSpace;

    PROCEDURE (f: Frame) CharIndex*
        (x, pos: INTEGER; IN s: ARRAY OF CHAR; font: Fonts.Font): INTEGER, NEW;
        VAR u: INTEGER;
    BEGIN
        u := f.unit;
        x := (f.gx + x) DIV u; pos := (f.gx + pos) DIV u;
        RETURN f.rider.CharIndex(x, pos, s, font)
    END CharIndex;

    PROCEDURE (f: Frame) CharPos*
        (x, index: INTEGER; IN s: ARRAY OF CHAR; font: Fonts.Font): INTEGER, NEW;
        VAR u: INTEGER;
    BEGIN
        u := f.unit;
        x := (f.gx + x) DIV u;
        RETURN f.rider.CharPos(x, index, s, font) * u - f.gx
    END CharPos;

    PROCEDURE (f: Frame) DrawSString*
        (x, y: INTEGER; col: Color; IN s: ARRAY OF SHORTCHAR; font: Fonts.Font), NEW;
        VAR u: INTEGER;
    BEGIN
        u := f.unit;
        x := (f.gx + x) DIV u; y := (f.gy + y) DIV u;
        f.rider.DrawSString(x, y, col, s, font)
    END DrawSString;

    PROCEDURE (f: Frame) SCharIndex*
        (x, pos: INTEGER; IN s: ARRAY OF SHORTCHAR; font: Fonts.Font): INTEGER, NEW;
        VAR u: INTEGER;
    BEGIN
        u := f.unit;
        x := (f.gx + x) DIV u; pos := (f.gx + pos) DIV u;
        RETURN f.rider.SCharIndex(x, pos, s, font)
    END SCharIndex;

    PROCEDURE (f: Frame) SCharPos*
        (x, index: INTEGER; IN s: ARRAY OF SHORTCHAR; font: Fonts.Font): INTEGER, NEW;
        VAR u: INTEGER;
    BEGIN
        u := f.unit;
        x := (f.gx + x) DIV u;
        RETURN f.rider.SCharPos(x, index, s, font) * u - f.gx
    END SCharPos;


    (* -- Module-level helpers --------------------------------------------- *)

    PROCEDURE RGBColor* (red, green, blue: INTEGER): Color;
    BEGIN
        ASSERT((red >= 0) & (red < 256), 20);
        ASSERT((green >= 0) & (green < 256), 21);
        ASSERT((blue >= 0) & (blue < 256), 22);
        RETURN (blue * 65536) + (green * 256) + red
    END RGBColor;

    PROCEDURE IsPrinterPort* (p: Port): BOOLEAN;
    BEGIN
        RETURN p.printerMode
    END IsPrinterPort;

BEGIN
    background       := white;
    dialogBackground := white
END Ports.
