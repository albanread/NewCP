MODULE PaneDemo;

(* End-to-end "see pixels" demo: opens a real iGui child window,
   builds a TextViews.Pane bound to a Doc containing some text,
   then drives Pane.Restore through a HostRider so the actual
   iGui surface receives Fill / Stroke / DrawText commands.  The
   child window stays open until the user closes the frame.

   Run with:
       newcp-driver run-igui PaneDemo.Run

   Expected on-screen result:
     - white window background
     - black bar across the top edge (the "view exists here"
       indicator from Pane.Restore phase 1)
     - "Hello, pixels!" rendered just below the bar in the
       default Segoe UI font

   Paint pipeline this exercises end-to-end:

       Pane.Restore
         → Views.Frame.DrawRect / DrawString
         → Ports coord translation
         → HostRider.DrawRect / DrawString
         → HostPortsSys.FillRect / DrawTextRun
         → iGui.EmitFillRect / EmitDrawTextRun
         → iGui surface batch
         → Direct2D paint on WM_PAINT

   This is the first slice where all the abstract dispatch has a
   concrete substrate on every layer.  The same CP code that
   drove the recording-rider unit tests now drives real pixels. *)

IMPORT iGui, Ports, Views, Fonts, HostFonts, TextModels, TextViews, HostPorts, HostPortsSys, Console;

TYPE
    (** Concrete Views.Frame.  Views.FrameDesc inherits the bounds
        / view / state fields with `-` read-only export, so we can't
        write them from this module — but Pane.Restore only reads
        the (l, t, r, b) passed to it as parameters and the
        Ports.FrameDesc inherited bits (unit, rider, gx, gy) that
        Ports' setters update.  Zero-init on the Views.FrameDesc
        fields is harmless for this slice. *)
    PaintFrameDesc* = RECORD (Views.FrameDesc) END;
    PaintFrame*     = POINTER TO PaintFrameDesc;

PROCEDURE Run*;
    VAR p: HostPorts.HostPort;
        childId: INTEGER;
        port: HostPorts.HostPort;
        d:   TextModels.Doc;
        wr:  TextModels.Writer;
        v:   TextViews.View;
        pf:  PaintFrame;
        ok:  INTSHORT;
        kind, evChild, timeMs, p1, p2, p3, p4: INTEGER;
BEGIN
    Console.WriteShortString("PaneDemo: opening iGui child window"); Console.WriteLn;

    (* Open a real iGui child window. *)
    port := HostPorts.NewPort("Pane demo", childId);
    IF port = NIL THEN
        Console.WriteShortString("PaneDemo: HostPorts.NewPort failed"); Console.WriteLn;
        RETURN
    END;
    port.Init(1, FALSE);                  (* 1 unit = 1 DIP *)
    Console.WriteShortString("PaneDemo: opened child id "); Console.WriteInt(childId); Console.WriteLn;

    (* Build a Doc with the message we want to render. *)
    NEW(d);
    wr := d.NewWriter(NIL);
    wr.WriteString("Hello, pixels!");
    Console.WriteShortString("PaneDemo: doc length = "); Console.WriteInt(d.Length()); Console.WriteLn;

    (* Build a Pane bound to the Doc. *)
    v := TextViews.dir.New(d);
    IF v = NIL THEN
        Console.WriteShortString("PaneDemo: TextViews.dir.New returned NIL"); Console.WriteLn;
        RETURN
    END;

    (* Wire a paint frame to the HostPort.  ConnectTo (inherited
       from Ports.FrameDesc) hooks the frame's rider to the port
       and stamps the unit. *)
    NEW(pf);
    pf.ConnectTo(port);
    pf.SetOffset(0, 0);

    (* Bracket the paint with an iGui batch, drive Pane.Restore,
       submit. *)
    HostPorts.BeginPaint(port);
    v.Restore(pf, 0, 0, 800, 600);
    ok := HostPorts.SubmitPaint();
    Console.WriteShortString("PaneDemo: SubmitPaint ok = "); Console.WriteInt(ok); Console.WriteLn;

    (* Pump events until the user closes the frame. *)
    Console.WriteShortString("PaneDemo: close the frame to exit"); Console.WriteLn;
    REPEAT
        ok := iGui.NextEvent(kind, evChild, timeMs, p1, p2, p3, p4, -1);
        IF (ok # 0) & (kind = iGui.EvFrameClose) THEN EXIT END
    UNTIL FALSE;

    p := port      (* silence unused-warning *)
END Run;

END PaneDemo.
