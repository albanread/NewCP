MODULE MVCNotifyProbe;

    IMPORT TextModels, TextViews, Models;

    PROCEDURE Run* (): INTEGER;
        VAR d:   TextModels.Doc;
            wr:  TextModels.Writer;
            v:   TextViews.View;
            wr2: TextModels.Writer;
    BEGIN
        NEW(d);

        (* NewWriter auto-installs a sequencer *)
        wr := d.NewWriter(NIL);
        IF d.seq = NIL THEN RETURN -1 END;

        (* 3 writes before observer installed — era should reach 3 *)
        wr.WriteChar('A');
        wr.WriteChar('B');
        wr.WriteChar('C');
        IF d.era # 3 THEN RETURN -2 END;
        IF d.len # 3 THEN RETURN -3 END;

        (* Bind a Pane — InitModel2 installs ViewObserver on d *)
        v := TextViews.dir.New(d);
        IF v = NIL THEN RETURN -4 END;
        (* observer installed implicitly — can't check private field directly *)

        (* 2 more writes — observer fires HandleModelMsg each time *)
        wr2 := d.NewWriter(NIL);
        wr2.WriteChar('D');
        wr2.WriteChar('E');
        IF d.era # 5 THEN RETURN -6 END;
        IF d.len # 5 THEN RETURN -7 END;

        RETURN d.era * 1000 + d.len   (* expect 5005 *)
    END Run;

END MVCNotifyProbe.
