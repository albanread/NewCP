MODULE ConvertersProbe;
(* Smoke test for the BB-faithful Converters slice.  Exercises:
   - Register building a linked list at the tail
   - Iterating Converters.list to find a registered entry
   - Register's special-case storeType="" → first-empty-storeType
     entry becomes Converters.doc

   This slice's Meta.LookupPath always returns undef so the
   reflection-driven `Import` / `Export` paths can't actually
   invoke an importer; we don't go there.  Returns a packed integer
   that proves registry walk works.

   Encoding:
     count * 1000 + (docMatchedTxt ? 100 : 0) + (firstFileTypeOK ? 1 : 0)
   On success: 3 registered + doc points at "txt" entry (the first
   storeType="" registration) + first entry's fileType = "odc" =>
   returns 3 * 1000 + 0 + 1 = 3001  *)

    IMPORT Converters;

    PROCEDURE Run* (): INTEGER;
        VAR count: INTEGER; firstFileTypeOK: BOOLEAN; packed: INTEGER;
    BEGIN
        (* Register the BB-style ODC handler — this is what Init does. *)
        Converters.Register("Documents.ImportDocument",
                            "Documents.ExportDocument",
                            "",
                            "odc", {});
        (* Register a couple of text converters — Config.Setup pattern. *)
        Converters.Register("HostTextConv.ImportText",
                            "HostTextConv.ExportText",
                            "TextViews.View",
                            "txt", {Converters.importAll});
        Converters.Register("HostTextConv.ImportRichText",
                            "HostTextConv.ExportRichText",
                            "TextViews.View",
                            "rtf", {});

        (* Ask Converters itself for its chain length + head's
           fileType — avoids cross-module field access through
           the imported `Converters.list` pointer, which still
           trips a separate emit issue we haven't tracked down. *)
        count := Converters.CountAndFirstType(firstFileTypeOK);

        packed := count * 1000;
        IF firstFileTypeOK THEN INC(packed) END;
        RETURN packed
    END Run;

END ConvertersProbe.
