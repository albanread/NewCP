MODULE Converters;
(*
   Pass 1: BB's Converters uses Meta.LookupPath to resolve import/export
   procedure names (strings) at runtime.  Meta.cp is not yet ported, so
   this implementation stores procedure-type variables directly in the
   Converter record.  Functionally equivalent; switch to string-based
   Meta lookup when Meta.cp lands.
*)

    IMPORT Files, Stores;

    TYPE
        Importer* = PROCEDURE (f: Files.File; OUT s: Stores.Store);
        Exporter* = PROCEDURE (s: Stores.Store; f: Files.File);

        ConverterDesc* = RECORD
            next-:     Converter;
            imp-:      Importer;
            exp-:      Exporter;
            fileType-: Files.Type;
            viewType-: Stores.TypeName
        END;
        Converter* = POINTER TO ConverterDesc;

    VAR
        list-: Converter;


    PROCEDURE Register* (imp: Importer; exp: Exporter;
                         IN fileType: Files.Type;
                         IN viewType: Stores.TypeName);
        VAR c: Converter;
    BEGIN
        NEW(c);
        c.imp      := imp;
        c.exp      := exp;
        c.fileType := fileType;
        c.viewType := viewType;
        c.next     := list;
        list       := c
    END Register;


    PROCEDURE ThisType* (IN type: Files.Type): Converter;
        VAR c: Converter;
    BEGIN
        c := list;
        WHILE c # NIL DO
            IF c.fileType = type THEN RETURN c END;
            c := c.next
        END;
        RETURN NIL
    END ThisType;


    PROCEDURE Import* (f: Files.File; IN type: Files.Type;
                       OUT s: Stores.Store);
        VAR c: Converter;
    BEGIN
        s := NIL;
        c := ThisType(type);
        IF (c # NIL) & (c.imp # NIL) THEN
            c.imp(f, s)
        END
    END Import;


    PROCEDURE Export* (s: Stores.Store; f: Files.File; IN type: Files.Type);
        VAR c: Converter;
    BEGIN
        c := ThisType(type);
        IF (c # NIL) & (c.exp # NIL) THEN
            c.exp(s, f)
        END
    END Export;


BEGIN
    list := NIL
END Converters.
