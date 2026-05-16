MODULE HostPng;
(*
   Render a Documents.Document to a PNG file.

   Uses the real iGui + Direct2D + DirectWrite pipeline so text,
   fonts and layout are rendered identically to an on-screen window.
   A temporary MDI child is opened, the document is painted into
   its batch, and the batch is captured to PNG via PrintWindow
   before the child is closed.

   Entry point:
       HostPng.RenderDoc(doc, w, h, "output.png")

   w, h are in pixels (port unit = 1 DIP).  The child window is
   opened at that size, painted once, captured, and closed —
   it may briefly flash on screen.
*)

    IMPORT Documents, Views, Ports, HostPorts, iGui;

    TYPE
        PaintFrameDesc = RECORD (Views.FrameDesc) END;
        PaintFrame     = POINTER TO PaintFrameDesc;


    PROCEDURE RenderDoc* (doc: Documents.Document;
                          w, h: INTEGER;
                          IN path: ARRAY OF CHAR): BOOLEAN;
        VAR port:      HostPorts.HostPort;
            pf:        PaintFrame;
            childId:   INTEGER;
            shortPath: ARRAY 512 OF SHORTCHAR;
            shortTitle: ARRAY 32 OF SHORTCHAR;
            i: INTEGER;
            ok: INTSHORT;
    BEGIN
        IF doc = NIL THEN RETURN FALSE END;

        (* Open a real MDI child window for the render. *)
        shortTitle := "__png_render__";
        port := HostPorts.NewPort(shortTitle, childId);
        IF port = NIL THEN RETURN FALSE END;
        port.Init(1, FALSE);      (* 1 unit = 1 DIP pixel *)

        (* Wire a paint frame to the port. *)
        NEW(pf);
        pf.ConnectTo(port);
        pf.SetOffset(0, 0);

        (* Convert output path CHAR → SHORTCHAR. *)
        i := 0;
        WHILE (path[i] # 0X) & (i < 511) DO
            shortPath[i] := SHORT(path[i]);
            INC(i)
        END;
        shortPath[i] := 0X;

        (* Build the render batch then capture + save synchronously. *)
        iGui.BeginBatch(childId);
        doc.Restore(pf, 0, 0, w, h);
        ok := iGui.CaptureBatchToPng(childId, shortPath);

        (* Tear down the temporary child. *)
        HostPorts.Close(port);

        RETURN ok = 1
    END RenderDoc;

END HostPng.
