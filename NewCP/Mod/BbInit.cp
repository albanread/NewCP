MODULE BbInit;
(*
   NewCP application entry point.

   Creates a text document, wraps it in a Documents.Document, opens
   it in a HostWindows-backed window, then pumps iGui events until
   the frame closes.

   Run with:
       newcp-driver run-igui BbInit.Run

   Module initialisation order (via import graph):
     Documents  -> sets dir/stdDir via its BEGIN block (StdDirectory)
     HostWindows -> calls Windows.SetDir(stdHostDir) in its BEGIN block

   Event dispatch:
     EvPaint      -> HostWindows.PaintChild(childId)
     EvResize     -> HostWindows.ResizeChild(childId, w, h)
     EvClose      -> HostWindows.CloseChild(childId)
     EvFrameClose -> exit loop
*)

    IMPORT Documents, Windows, TextModels, TextViews, HostWindows, iGui, Console;

    CONST
        winW = 800;
        winH = 600;


    PROCEDURE Run*;
        VAR d:   TextModels.Doc;
            wr:  TextModels.Writer;
            v:   TextViews.View;
            doc: Documents.Document;
            win: Windows.Window;
            ok:  INTSHORT;
            kind, childId, timeMs, p1, p2, p3, p4: INTEGER;
    BEGIN
        Console.WriteShortString("BbInit: starting"); Console.WriteLn;

        (* Build a text model with welcome content. *)
        NEW(d);
        wr := d.NewWriter(NIL);
        wr.WriteString("Welcome to NewCP!");

        (* Wrap in a TextViews view. *)
        v := TextViews.dir.New(d);
        IF v = NIL THEN
            Console.WriteShortString("BbInit: TextViews.dir.New returned NIL");
            Console.WriteLn;
            RETURN
        END;

        (* Create a document wrapping the view.
           Documents.dir is non-NIL because Documents self-initialises
           its StdDirectory in its module BEGIN block. *)
        doc := Documents.dir.New(v, winW, winH);
        IF doc = NIL THEN
            Console.WriteShortString("BbInit: Documents.dir.New returned NIL");
            Console.WriteLn;
            RETURN
        END;

        (* Open an MDI child window.
           Windows.dir is non-NIL because HostWindows sets it in its BEGIN block.
           Windows.Open prepends the window to Windows.first. *)
        win := Windows.Open(doc, "Welcome to NewCP", winW, winH);
        IF win = NIL THEN
            Console.WriteShortString("BbInit: Windows.Open returned NIL");
            Console.WriteLn;
            RETURN
        END;

        Console.WriteShortString("BbInit: window open — close frame to exit");
        Console.WriteLn;

        (* Event loop: dispatch iGui events to HostWindows handlers. *)
        REPEAT
            ok := iGui.NextEvent(kind, childId, timeMs, p1, p2, p3, p4, -1);
            IF ok # 0 THEN
                IF    kind = iGui.EvPaint  THEN HostWindows.PaintChild(childId)
                ELSIF kind = iGui.EvResize THEN HostWindows.ResizeChild(childId, p1, p2)
                ELSIF kind = iGui.EvClose  THEN HostWindows.CloseChild(childId)
                ELSIF kind = iGui.EvFrameClose THEN EXIT
                END
            END
        UNTIL FALSE;

        Console.WriteShortString("BbInit: done"); Console.WriteLn
    END Run;

END BbInit.
