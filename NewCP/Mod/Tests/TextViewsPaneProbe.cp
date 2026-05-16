MODULE TextViewsPaneProbe;

    IMPORT TextModels, TextViews;

    PROCEDURE Run* (): INTEGER;
        VAR sm:    TextModels.StdModel;
            sv:    TextViews.StdView;
            d:     TextModels.Doc;
            wr:    TextModels.Writer;
            p:     TextViews.Pane;
            d2:    TextModels.Doc;
            p2:    TextViews.Pane;
            pView: TextViews.View;
    BEGIN
        (* Test 1: StdModelToDoc with known text *)
        NEW(sm);
        sm.result  := TextModels.OkComplete;
        sm.text[0] := 'H'; sm.text[1] := 'e'; sm.text[2] := 'l';
        sm.text[3] := 'l'; sm.text[4] := 'o'; sm.text[5] := 0X;
        sm.textLen := 5;
        d := TextViews.StdModelToDoc(sm);
        IF d = NIL THEN RETURN -1 END;
        IF d.len # 5 THEN RETURN -2 END;
        IF d.buf[0] # 'H' THEN RETURN -3 END;
        IF d.buf[4] # 'o' THEN RETURN -4 END;

        (* Test 2: NewPane from a synthesised StdView *)
        NEW(sv);
        sv.result    := TextViews.OkComplete;
        NEW(sm);
        sm.text[0] := 'W'; sm.text[1] := 'o'; sm.text[2] := 'r';
        sm.text[3] := 'l'; sm.text[4] := 'd'; sm.text[5] := 0X;
        sm.textLen   := 5;
        sm.result    := TextModels.OkComplete;
        sv.model     := sm;
        sv.org       := 7;
        sv.dy        := 3;
        sv.hideMarks := TRUE;
        p := TextViews.NewPane(sv);
        IF p = NIL THEN RETURN -5 END;
        IF p.text = NIL THEN RETURN -6 END;
        IF p.text.Length() # 5 THEN RETURN -7 END;
        IF p.org # 7 THEN RETURN -8 END;
        IF p.dy # 3 THEN RETURN -9 END;
        IF p.hideMarks # TRUE THEN RETURN -10 END;

        (* Test 3: stdDir.New with a Doc *)
        NEW(d2);
        wr := d2.NewWriter(NIL);
        wr.WriteString("AB");
        pView := TextViews.stdDir.New(d2);
        IF pView = NIL THEN RETURN -11 END;
        IF ~(pView IS TextViews.Pane) THEN RETURN -12 END;
        p2 := pView(TextViews.Pane);
        IF p2.text.Length() # 2 THEN RETURN -13 END;

        RETURN d.len * 100 + p.text.Length() * 10 + p2.text.Length()
    END Run;

END TextViewsPaneProbe.
