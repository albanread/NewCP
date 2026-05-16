MODULE TextRulersAttrCopyOfProbe;

    IMPORT TextRulers, Stores;

    PROCEDURE Run* (): INTEGER;
        VAR orig: TextRulers.Attributes;
            clone: Stores.Store;
            copy:  TextRulers.Attributes;
    BEGIN
        IF TextRulers.dir = NIL THEN RETURN 0 END;
        orig := TextRulers.dir.attr;
        IF orig = NIL THEN RETURN 0 END;

        clone := Stores.CopyOf(orig);
        IF clone = NIL THEN RETURN -1 END;
        IF ~(clone IS TextRulers.Attributes) THEN RETURN -2 END;
        copy := clone(TextRulers.Attributes);
        IF copy = orig THEN RETURN -3 END;

        IF copy.init # orig.init THEN RETURN -4 END;

        IF orig.init THEN
            IF copy.first # orig.first THEN RETURN -5 END;
            IF copy.left  # orig.left  THEN RETURN -6 END;
            IF copy.right # orig.right THEN RETURN -7 END;
            IF copy.lead  # orig.lead  THEN RETURN -8 END;
            IF copy.dsc   # orig.dsc   THEN RETURN -9 END;
            IF copy.grid  # orig.grid  THEN RETURN -10 END;
            IF copy.opts  # orig.opts  THEN RETURN -11 END;
            IF copy.tabs.len # orig.tabs.len THEN RETURN -12 END
        END;

        RETURN copy.tabs.len * 100 + 42
    END Run;

END TextRulersAttrCopyOfProbe.
