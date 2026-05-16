MODULE TextModelsDocCopyOfProbe;

    IMPORT TextModels, Stores;

    PROCEDURE Run* (): INTEGER;
        VAR d:  TextModels.Doc;
            wr: TextModels.Writer;
            clone: Stores.Store;
            d2: TextModels.Doc;
    BEGIN
        NEW(d);
        wr := d.NewWriter(NIL);
        wr.WriteString("Hello");   (* 5 chars *)

        IF d.len # 5 THEN RETURN -1 END;
        IF d.buf[0] # "H" THEN RETURN -2 END;

        clone := Stores.CopyOf(d);
        IF clone = NIL THEN RETURN -3 END;
        IF ~(clone IS TextModels.Doc) THEN RETURN -4 END;
        d2 := clone(TextModels.Doc);
        IF d2 = d THEN RETURN -5 END;   (* must be distinct *)

        IF d2.len # 5 THEN RETURN -6 END;
        IF d2.buf[0] # "H" THEN RETURN -7 END;
        IF d2.buf[4] # "o" THEN RETURN -8 END;
        IF d2.buf[5] # 0X THEN RETURN -9 END;   (* sentinel *)

        RETURN d2.len * 100 + ORD(d2.buf[0])
        (* expect 5 * 100 + 72 = 572 *)
    END Run;

END TextModelsDocCopyOfProbe.
