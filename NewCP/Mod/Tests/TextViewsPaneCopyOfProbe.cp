MODULE TextViewsPaneCopyOfProbe;

    IMPORT TextModels, TextViews, Stores;

    PROCEDURE Run* (): INTEGER;
        VAR d:   TextModels.Doc;
            wr:  TextModels.Writer;
            v:   TextViews.View;
            p:   TextViews.Pane;
            clone: Stores.Store;
            copy:  TextViews.Pane;
    BEGIN
        (* Build a Doc with 3 chars. *)
        NEW(d);
        wr := d.NewWriter(NIL);
        wr.WriteString("Hi!");     (* 3 chars *)
        IF d.len # 3 THEN RETURN -1 END;

        (* Wrap in a Pane and set display state. *)
        v := TextViews.dir.New(d);
        IF v = NIL THEN RETURN -2 END;
        p := v(TextViews.Pane);
        v.SetOrigin(5, 2);
        v.DisplayMarks(TRUE);     (* hideMarks := TRUE *)

        (* Round-trip via CopyOf. *)
        clone := Stores.CopyOf(p);
        IF clone = NIL THEN RETURN -3 END;
        IF ~(clone IS TextViews.Pane) THEN RETURN -4 END;
        copy := clone(TextViews.Pane);
        IF copy = p THEN RETURN -5 END;   (* must be distinct *)

        (* Verify display state round-tripped. *)
        IF copy.org # 5 THEN RETURN -6 END;
        IF copy.dy  # 2 THEN RETURN -7 END;
        IF ~copy.hideMarks THEN RETURN -8 END;

        (* Verify model round-tripped. *)
        IF copy.text = NIL THEN RETURN -9 END;
        IF copy.text.Length() # 3 THEN RETURN -10 END;

        RETURN copy.text.Length() * 10000 + copy.org * 100 + copy.dy
        (* expect 3 * 10000 + 5 * 100 + 2 = 30502 *)
    END Run;

END TextViewsPaneCopyOfProbe.
