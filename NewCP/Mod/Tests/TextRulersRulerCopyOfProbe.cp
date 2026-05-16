MODULE TextRulersRulerCopyOfProbe;

    IMPORT TextRulers, Stores;

    PROCEDURE Run* (): INTEGER;
        VAR origAttr:  TextRulers.Attributes;
            origStyle: TextRulers.Style;
            origRuler: TextRulers.Ruler;
            clone:     Stores.Store;
            cloneRuler: TextRulers.Ruler;
    BEGIN
        IF TextRulers.dir = NIL THEN RETURN 0 END;
        origAttr := TextRulers.dir.attr;
        IF (origAttr = NIL) OR ~origAttr.init THEN RETURN 0 END;

        origStyle := TextRulers.dir.NewStyle(origAttr);
        IF origStyle = NIL THEN RETURN -1 END;

        origRuler := TextRulers.dir.New(origStyle);
        IF origRuler = NIL THEN RETURN -2 END;
        IF origRuler.style # origStyle THEN RETURN -3 END;

        clone := Stores.CopyOf(origRuler);
        IF clone = NIL THEN RETURN -4 END;
        IF ~(clone IS TextRulers.Ruler) THEN RETURN -5 END;
        cloneRuler := clone(TextRulers.Ruler);
        IF cloneRuler = origRuler THEN RETURN -6 END;    (* must be distinct *)

        IF cloneRuler.style = NIL THEN RETURN -7 END;
        IF cloneRuler.style.attr = NIL THEN RETURN -8 END;
        IF ~cloneRuler.style.attr.init THEN RETURN -9 END;

        IF cloneRuler.style.attr.first # origAttr.first THEN RETURN -10 END;
        IF cloneRuler.style.attr.opts  # origAttr.opts  THEN RETURN -11 END;
        IF cloneRuler.style.attr.tabs.len # origAttr.tabs.len THEN RETURN -12 END;

        RETURN cloneRuler.style.attr.tabs.len * 1000 + 99
    END Run;

END TextRulersRulerCopyOfProbe.
