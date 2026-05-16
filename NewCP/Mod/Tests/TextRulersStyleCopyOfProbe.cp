MODULE TextRulersStyleCopyOfProbe;

    IMPORT TextRulers, Stores;

    PROCEDURE Run* (): INTEGER;
        VAR origAttr: TextRulers.Attributes;
            origStyle: TextRulers.Style;
            clone: Stores.Store;
            cloneStyle: TextRulers.Style;
    BEGIN
        (* Guard: need an installed directory with a default attr. *)
        IF TextRulers.dir = NIL THEN RETURN 0 END;
        origAttr := TextRulers.dir.attr;
        IF origAttr = NIL THEN RETURN 0 END;
        IF ~origAttr.init THEN RETURN 0 END;

        (* Build a StdStyle wrapping the default attr. *)
        origStyle := TextRulers.dir.NewStyle(origAttr);
        IF origStyle = NIL THEN RETURN -1 END;
        IF origStyle.attr # origAttr THEN RETURN -2 END;

        (* Round-trip through CopyOf — exercises WriteStore + ReadStore. *)
        clone := Stores.CopyOf(origStyle);
        IF clone = NIL THEN RETURN -3 END;
        IF ~(clone IS TextRulers.Style) THEN RETURN -4 END;
        cloneStyle := clone(TextRulers.Style);
        IF cloneStyle = origStyle THEN RETURN -5 END;  (* must be distinct *)

        (* The cloned style's attr must have been round-tripped. *)
        IF cloneStyle.attr = NIL THEN RETURN -6 END;
        IF ~cloneStyle.attr.init THEN RETURN -7 END;

        (* Scalar field parity — exercises the full Attributes Internalize. *)
        IF cloneStyle.attr.first # origAttr.first THEN RETURN -8 END;
        IF cloneStyle.attr.left  # origAttr.left  THEN RETURN -9 END;
        IF cloneStyle.attr.opts  # origAttr.opts  THEN RETURN -10 END;
        IF cloneStyle.attr.tabs.len # origAttr.tabs.len THEN RETURN -11 END;

        (* cloneStyle.attr may be the SAME object as origAttr (if
           Internalize re-uses the interned attrs) or a distinct clone —
           both are valid.  Only field values matter. *)

        RETURN cloneStyle.attr.tabs.len * 100 + 77
    END Run;

END TextRulersStyleCopyOfProbe.
