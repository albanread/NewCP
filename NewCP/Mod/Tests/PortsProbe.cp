MODULE PortsProbe;
(* Verify the Ports coordinate-translation contract end-to-end:
   a Frame connected to a recording Port should forward DrawRect /
   DrawLine calls to its Rider after dividing every coordinate by
   the Port's `unit`.  The probe sets unit = 100 so the arithmetic
   `(gx + l) DIV u` is easy to read in the trace.

   No real drawing happens — the test Port hands out a test Rider
   that just records the most recent call's translated coordinates
   in module-level variables.  The integration test reads those
   back through Run() to confirm both the dispatch path (Frame →
   Rider) and the math (user units → device dots).
*)

    IMPORT Ports, Fonts;

    TYPE
        TestPortDesc* = RECORD (Ports.PortDesc) END;
        TestPort*     = POINTER TO TestPortDesc;

        TestRiderDesc* = RECORD (Ports.RiderDesc)
            base*: TestPort
        END;
        TestRider*     = POINTER TO TestRiderDesc;

        TestFrameDesc* = RECORD (Ports.FrameDesc) END;
        TestFrame*     = POINTER TO TestFrameDesc;

    VAR
        (* Last-rect captured by the rider's DrawRect.  Kept exported
           with read-only modifier (`-`) so the integration test can
           inspect them via direct global reads if needed. *)
        rectL-, rectT-, rectR-, rectB-, rectS-: INTEGER;
        rectColor-:                              INTEGER;

        (* Last-line captured by the rider's DrawLine. *)
        lineX0-, lineY0-, lineX1-, lineY1-, lineS-: INTEGER;
        lineColor-:                                  INTEGER;

        (* Number of calls observed — sanity check that the dispatch
           actually reached the rider. *)
        rectCallCount-: INTEGER;
        lineCallCount-: INTEGER;


    (* -- TestPort: minimal concrete Port ---------------------------------- *)

    PROCEDURE (p: TestPortDesc) GetSize* (OUT w, h: INTEGER);
    BEGIN
        w := 800;
        h := 600
    END GetSize;

    PROCEDURE (p: TestPortDesc) SetSize* (w, h: INTEGER);
    BEGIN END SetSize;

    PROCEDURE (p: TestPortDesc) NewRider* (): Ports.Rider;
        VAR r: TestRider;
    BEGIN
        NEW(r);
        RETURN r
    END NewRider;

    PROCEDURE (p: TestPortDesc) OpenBuffer* (l, t, r, b: INTEGER);
    BEGIN END OpenBuffer;

    PROCEDURE (p: TestPortDesc) CloseBuffer* ();
    BEGIN END CloseBuffer;


    (* -- TestRider: stub every abstract method, record DrawRect / Line ---- *)

    PROCEDURE (rd: TestRiderDesc) SetRect* (l, t, r, b: INTEGER);
    BEGIN END SetRect;

    PROCEDURE (rd: TestRiderDesc) GetRect* (OUT l, t, r, b: INTEGER);
    BEGIN
        l := 0; t := 0; r := 0; b := 0
    END GetRect;

    PROCEDURE (rd: TestRiderDesc) Base* (): Ports.Port;
    BEGIN
        RETURN rd.base
    END Base;

    PROCEDURE (rd: TestRiderDesc) Move* (dx, dy: INTEGER);
    BEGIN END Move;

    PROCEDURE (rd: TestRiderDesc) SaveRect*
        (l, t, r, b: INTEGER; VAR res: INTEGER);
    BEGIN res := 0 END SaveRect;

    PROCEDURE (rd: TestRiderDesc) RestoreRect*
        (l, t, r, b: INTEGER; dispose: BOOLEAN);
    BEGIN END RestoreRect;

    PROCEDURE (rd: TestRiderDesc) DrawRect*
        (l, t, r, b, s: INTEGER; col: Ports.Color);
    BEGIN
        rectL := l; rectT := t; rectR := r; rectB := b;
        rectS := s; rectColor := col;
        INC(rectCallCount)
    END DrawRect;

    PROCEDURE (rd: TestRiderDesc) DrawOval*
        (l, t, r, b, s: INTEGER; col: Ports.Color);
    BEGIN END DrawOval;

    PROCEDURE (rd: TestRiderDesc) DrawLine*
        (x0, y0, x1, y1, s: INTEGER; col: Ports.Color);
    BEGIN
        lineX0 := x0; lineY0 := y0; lineX1 := x1; lineY1 := y1;
        lineS := s; lineColor := col;
        INC(lineCallCount)
    END DrawLine;

    PROCEDURE (rd: TestRiderDesc) DrawPath*
        (IN p: ARRAY OF Ports.Point; n, s: INTEGER; col: Ports.Color;
         path: INTEGER);
    BEGIN END DrawPath;

    PROCEDURE (rd: TestRiderDesc) MarkRect*
        (l, t, r, b, s, mode: INTEGER; show: BOOLEAN);
    BEGIN END MarkRect;

    PROCEDURE (rd: TestRiderDesc) Scroll* (dx, dy: INTEGER);
    BEGIN END Scroll;

    PROCEDURE (rd: TestRiderDesc) SetCursor* (cursor: INTEGER);
    BEGIN END SetCursor;

    PROCEDURE (rd: TestRiderDesc) Input*
        (OUT x, y: INTEGER; OUT modifiers: SET; OUT isDown: BOOLEAN);
    BEGIN
        x := 0; y := 0; modifiers := {}; isDown := FALSE
    END Input;

    (* Text dispatch: stubbed.  We don't import Fonts here either —
       only declare the parameter types Ports demands so the compiler
       can build vtable slots.  The integration test never invokes
       these; a dedicated text-on-frame probe lives elsewhere. *)
    PROCEDURE (rd: TestRiderDesc) DrawString*
        (x, y: INTEGER; col: Ports.Color; IN s: ARRAY OF CHAR;
         font: Fonts.Font);
    BEGIN END DrawString;

    PROCEDURE (rd: TestRiderDesc) DrawSpace*
        (x, y, w: INTEGER; col: Ports.Color; font: Fonts.Font);
    BEGIN END DrawSpace;

    PROCEDURE (rd: TestRiderDesc) CharIndex*
        (x, pos: INTEGER; IN s: ARRAY OF CHAR;
         font: Fonts.Font): INTEGER;
    BEGIN RETURN 0 END CharIndex;

    PROCEDURE (rd: TestRiderDesc) CharPos*
        (x, index: INTEGER; IN s: ARRAY OF CHAR;
         font: Fonts.Font): INTEGER;
    BEGIN RETURN 0 END CharPos;

    PROCEDURE (rd: TestRiderDesc) DrawSString*
        (x, y: INTEGER; col: Ports.Color; IN s: ARRAY OF SHORTCHAR;
         font: Fonts.Font);
    BEGIN END DrawSString;

    PROCEDURE (rd: TestRiderDesc) SCharIndex*
        (x, pos: INTEGER; IN s: ARRAY OF SHORTCHAR;
         font: Fonts.Font): INTEGER;
    BEGIN RETURN 0 END SCharIndex;

    PROCEDURE (rd: TestRiderDesc) SCharPos*
        (x, index: INTEGER; IN s: ARRAY OF SHORTCHAR;
         font: Fonts.Font): INTEGER;
    BEGIN RETURN 0 END SCharPos;


    (* -- Run -------------------------------------------------------------- *)

    (** Set up a Frame bound to a TestPort with unit = 100, offset
        gx = 50 / gy = 70, then call Frame.DrawRect with user-space
        coords (l=0, t=0, r=200, b=300, s=fill, col=red).  Expect the
        rider to receive translated coords:
            l = (50 + 0)  DIV 100 = 0
            t = (70 + 0)  DIV 100 = 0
            r = (50 + 200) DIV 100 = 2
            b = (70 + 300) DIV 100 = 3
            s = -1 DIV 100 = -1   (* fill stays fill: DIV truncates toward -∞ in CP *)
        Returns a packed integer:
            (rectCallCount * 1000000) + (rectR * 1000) + (rectB * 100)
              + (rectColor MOD 1000)
        Expect 1_002_300 + 255 = 1_002_555 — one call, r=2, b=3, color
        red = 0xFF = 255. *)
    PROCEDURE Run* (): INTEGER;
        VAR
            p: TestPort;
            f: TestFrame;
    BEGIN
        rectCallCount := 0;
        lineCallCount := 0;

        NEW(p);
        p.Init(100, FALSE);

        NEW(f);
        f.ConnectTo(p);
        f.SetOffset(50, 70);

        f.DrawRect(0, 0, 200, 300, Ports.fill, Ports.red);

        RETURN (rectCallCount * 1000000) + (rectR * 1000) + (rectB * 100) + (rectColor MOD 1000)
    END Run;


BEGIN
    rectCallCount := 0;
    lineCallCount := 0
END PortsProbe.
