MODULE TextViewsPanePixels;
(* First-pixels probe: drive a concrete Pane through its Restore
   method onto a recording Rider, then verify the right paint
   calls landed.

   This proves the full pipeline:
       Pane.Restore
         → Views.Frame.DrawRect
         → coord translation
         → Rider.DrawRect
         → recorded into module-level state

   Two paint calls expected from Pane.Restore:
     1. background fill   (white,    full dirty rect)
     2. indicator bar     (black,    top-edge band if model bound)

   With unit=1 and gx=gy=0 the translation is identity; with
   model=NIL only the background fills (one call). *)

    IMPORT Ports, Fonts, Views, Containers, TextModels, TextViews;

    TYPE
        TestPortDesc* = RECORD (Ports.PortDesc) END;
        TestPort*     = POINTER TO TestPortDesc;

        TestRiderDesc* = RECORD (Ports.RiderDesc) END;
        TestRider*     = POINTER TO TestRiderDesc;

        (** Minimum-viable concrete TextModels.Model so we can
            test Pane.Restore's "is the pane bound to a model"
            branch.  The Reader/Writer factories return NIL since
            this slice doesn't read text — just paints scaffolding. *)
        StubModelDesc* = RECORD (TextModels.ModelDesc) END;
        StubModel*     = POINTER TO StubModelDesc;

        (** Concrete Views.Frame.  Views.FrameDesc adds bounds /
            view / front / mark / state / coord fields, all
            read-only-exported — we can't set them from this
            probe module.  Pane.Restore never reads them (it
            uses the (l, t, r, b) it receives as parameters and
            forwards via Ports.FrameDesc inherited methods), so
            leaving them at zero-init is safe. *)
        TestFrameDesc* = RECORD (Views.FrameDesc) END;
        TestFrame*     = POINTER TO TestFrameDesc;

    VAR
        rectCallCount-: INTEGER;
        (* Per-call recorded args: index 0 = first DrawRect, etc.
           Sized for at most 4 calls so a later slice can verify
           more shapes without re-sizing. *)
        rectL-, rectT-, rectR-, rectB-: ARRAY 4 OF INTEGER;
        rectS-, rectColor-:              ARRAY 4 OF INTEGER;


    (* -- StubModel: minimal concrete TextModels.Model --------------------- *)

    PROCEDURE (m: StubModelDesc) NewReader* (old: TextModels.Reader): TextModels.Reader;
    BEGIN RETURN NIL END NewReader;
    PROCEDURE (m: StubModelDesc) NewWriter* (old: TextModels.Writer): TextModels.Writer;
    BEGIN RETURN NIL END NewWriter;
    PROCEDURE (m: StubModelDesc) Length* (): INTEGER;
    BEGIN RETURN 0 END Length;
    PROCEDURE (m: StubModelDesc) GetEmbeddingLimits*
        (OUT minW, maxW, minH, maxH: INTEGER);
    BEGIN minW := 0; maxW := 1000; minH := 0; maxH := 1000 END GetEmbeddingLimits;
    PROCEDURE (m: StubModelDesc) ReplaceView* (old, new: Views.View);
    BEGIN END ReplaceView;


    (* -- TestPort: minimal concrete Port ---------------------------------- *)

    PROCEDURE (p: TestPortDesc) GetSize* (OUT w, h: INTEGER);
    BEGIN
        w := 800; h := 600
    END GetSize;
    PROCEDURE (p: TestPortDesc) SetSize* (w, h: INTEGER);
    BEGIN END SetSize;
    PROCEDURE (p: TestPortDesc) NewRider* (): Ports.Rider;
        VAR r: TestRider;
    BEGIN
        NEW(r); RETURN r
    END NewRider;
    PROCEDURE (p: TestPortDesc) OpenBuffer* (l, t, r, b: INTEGER);
    BEGIN END OpenBuffer;
    PROCEDURE (p: TestPortDesc) CloseBuffer* ();
    BEGIN END CloseBuffer;


    (* -- TestRider: stub abstracts, record DrawRect ----------------------- *)

    PROCEDURE (rd: TestRiderDesc) SetRect* (l, t, r, b: INTEGER);
    BEGIN END SetRect;
    PROCEDURE (rd: TestRiderDesc) GetRect* (OUT l, t, r, b: INTEGER);
    BEGIN l := 0; t := 0; r := 0; b := 0 END GetRect;
    PROCEDURE (rd: TestRiderDesc) Base* (): Ports.Port;
    BEGIN RETURN NIL END Base;
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
        VAR i: INTEGER;
    BEGIN
        i := rectCallCount;
        IF i < 4 THEN
            rectL[i] := l; rectT[i] := t;
            rectR[i] := r; rectB[i] := b;
            rectS[i] := s; rectColor[i] := col
        END;
        INC(rectCallCount)
    END DrawRect;

    PROCEDURE (rd: TestRiderDesc) DrawOval*
        (l, t, r, b, s: INTEGER; col: Ports.Color);
    BEGIN END DrawOval;
    PROCEDURE (rd: TestRiderDesc) DrawLine*
        (x0, y0, x1, y1, s: INTEGER; col: Ports.Color);
    BEGIN END DrawLine;
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
    BEGIN x := 0; y := 0; modifiers := {}; isDown := FALSE END Input;
    PROCEDURE (rd: TestRiderDesc) DrawString*
        (x, y: INTEGER; col: Ports.Color; IN s: ARRAY OF CHAR;
         font: Fonts.Font);
    BEGIN END DrawString;
    PROCEDURE (rd: TestRiderDesc) DrawSpace*
        (x, y, w: INTEGER; col: Ports.Color; font: Fonts.Font);
    BEGIN END DrawSpace;
    PROCEDURE (rd: TestRiderDesc) CharIndex*
        (x, pos: INTEGER; IN s: ARRAY OF CHAR; font: Fonts.Font): INTEGER;
    BEGIN RETURN 0 END CharIndex;
    PROCEDURE (rd: TestRiderDesc) CharPos*
        (x, index: INTEGER; IN s: ARRAY OF CHAR; font: Fonts.Font): INTEGER;
    BEGIN RETURN 0 END CharPos;
    PROCEDURE (rd: TestRiderDesc) DrawSString*
        (x, y: INTEGER; col: Ports.Color; IN s: ARRAY OF SHORTCHAR;
         font: Fonts.Font);
    BEGIN END DrawSString;
    PROCEDURE (rd: TestRiderDesc) SCharIndex*
        (x, pos: INTEGER; IN s: ARRAY OF SHORTCHAR; font: Fonts.Font): INTEGER;
    BEGIN RETURN 0 END SCharIndex;
    PROCEDURE (rd: TestRiderDesc) SCharPos*
        (x, index: INTEGER; IN s: ARRAY OF SHORTCHAR; font: Fonts.Font): INTEGER;
    BEGIN RETURN 0 END SCharPos;


    (* -- helpers ---------------------------------------------------------- *)

    PROCEDURE ResetCapture;
        VAR i: INTEGER;
    BEGIN
        rectCallCount := 0;
        i := 0;
        WHILE i < 4 DO
            rectL[i] := 0; rectT[i] := 0;
            rectR[i] := 0; rectB[i] := 0;
            rectS[i] := 0; rectColor[i] := 0;
            INC(i)
        END
    END ResetCapture;

    PROCEDURE BuildFrame (): TestFrame;
        VAR p: TestPort; f: TestFrame;
    BEGIN
        NEW(p);
        p.Init(1, FALSE);
        NEW(f);
        f.ConnectTo(p);
        f.SetOffset(0, 0);
        RETURN f
    END BuildFrame;


    (* -- Probes ----------------------------------------------------------- *)

    (** Pane with no bound model: Restore emits only the background
        fill — exactly one DrawRect call with the white color. *)
    PROCEDURE RestoreUnboundEmitsBackground* (): INTEGER;
        VAR v: TextViews.View; f: TestFrame; result: INTEGER;
    BEGIN
        ResetCapture;
        v := TextViews.dir.New(NIL);
        IF v = NIL THEN RETURN -1 END;
        f := BuildFrame();

        v.Restore(f, 0, 0, 800, 600);

        result := 0;
        IF rectCallCount = 1 THEN INC(result, 1) END;
        IF rectColor[0] = Ports.white THEN INC(result, 2) END;
        IF (rectL[0] = 0) & (rectT[0] = 0)
         & (rectR[0] = 800) & (rectB[0] = 600) THEN INC(result, 4) END;
        IF rectS[0] = Ports.fill THEN INC(result, 8) END;
        RETURN result  (* expect 15 if every check passes *)
    END RestoreUnboundEmitsBackground;

    (** Pane bound to a real model: Restore emits background fill
        PLUS the top-edge indicator bar — two DrawRect calls in
        order (white background first, black bar second). *)
    PROCEDURE RestoreBoundEmitsBackgroundAndBar* (): INTEGER;
        VAR m: StubModel;
            v: TextViews.View; f: TestFrame;
            result: INTEGER;
    BEGIN
        ResetCapture;
        NEW(m);
        v := TextViews.dir.New(m);
        IF v = NIL THEN RETURN -1 END;
        f := BuildFrame();

        v.Restore(f, 0, 0, 800, 600);

        result := 0;
        IF rectCallCount = 2 THEN INC(result, 1) END;
        IF rectColor[0] = Ports.white THEN INC(result, 2) END;
        IF rectColor[1] = Ports.black THEN INC(result, 4) END;
        IF (rectL[1] = 0) & (rectT[1] = 0)
         & (rectR[1] = 800) & (rectB[1] = 50) THEN INC(result, 8) END;
        RETURN result  (* expect 15 if every check passes *)
    END RestoreBoundEmitsBackgroundAndBar;


BEGIN
    rectCallCount := 0
END TextViewsPanePixels.
