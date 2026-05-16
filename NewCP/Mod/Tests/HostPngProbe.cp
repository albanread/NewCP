MODULE HostPngProbe;
(*
   Smoke-test for HostPng.RenderDoc.

   Opens a TextModels.Doc with "Hello from NewCP!", wraps it in a
   Documents.Document, and renders it to "test_render.png" in the
   current working directory.

   Run with:
       newcp-driver run-igui HostPngProbe.Run

   Expected: test_render.png written next to the executable, showing
   the text rendered via DirectWrite.
*)

    IMPORT Documents, TextModels, TextViews, HostPng, Console;

    PROCEDURE Run*;
        VAR d:   TextModels.Doc;
            wr:  TextModels.Writer;
            v:   TextViews.View;
            doc: Documents.Document;
            ok:  BOOLEAN;
    BEGIN
        Console.WriteShortString("HostPngProbe: building document"); Console.WriteLn;

        NEW(d);
        wr := d.NewWriter(NIL);
        wr.WriteString("Hello from NewCP!");

        v := TextViews.dir.New(d);
        IF v = NIL THEN
            Console.WriteShortString("HostPngProbe: TextViews.dir.New = NIL"); Console.WriteLn;
            RETURN
        END;

        doc := Documents.dir.New(v, 800, 200);
        IF doc = NIL THEN
            Console.WriteShortString("HostPngProbe: Documents.dir.New = NIL"); Console.WriteLn;
            RETURN
        END;

        Console.WriteShortString("HostPngProbe: rendering to test_render.png"); Console.WriteLn;
        ok := HostPng.RenderDoc(doc, 800, 200, "test_render.png");

        IF ok THEN
            Console.WriteShortString("HostPngProbe: OK — test_render.png written"); Console.WriteLn
        ELSE
            Console.WriteShortString("HostPngProbe: FAIL — RenderDoc returned FALSE"); Console.WriteLn
        END
    END Run;

END HostPngProbe.
