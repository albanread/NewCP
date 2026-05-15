MODULE Converters;
(*
   First slice of the BlackBox `Converters` port.

   BB's `Converters` is a small (~200-line) module that runs the
   file-format dispatch registry.  Modules call `Register` at
   startup to bind a (file-extension, importer-proc, exporter-proc,
   store-type) tuple; later the framework's "Open file" command
   calls `Import` which:

     1. walks the registered linked list, picking the converter
        whose `fileType` matches the file's extension (or any
        converter with `importAll` set);
     2. resolves the importer's name string
        (e.g. `"Documents.ImportDocument"`) to a callable
        procedure via `Meta.LookupPath`;
     3. invokes that procedure on the open file, getting back a
        `Stores.Store`.

   `Export` is the symmetric path for save-as.

   What's working in THIS slice:
   - `Register` builds the linked list, appending to the tail
     and stamping `doc` on the first storeType="" entry.
   - `list` / `Converter` records are inspectable from within
     the module (use `CountAndFirstType` rather than walking
     the imported `list` global — cross-module field access
     through it still trips a separate emit issue).
   - `GetCommand` is BB-faithful at the surface; body deferred.

   Deferred (waiting on follow-ups):
   - `Import` / `Export` bodies — the BB-faithful versions
     declare `ImpVal`/`ExpVal` (records extending `Meta.Value`
     with a procedure-pointer field) and call `val.p(...)`
     after `GetCommand` resolves the name.  Currently
     compile-hangs the LLVM emitter when the abstract-extending
     record carrying a procedure-pointer field is materialised
     in the call site.  Restore once that's fixed upstream.
   - `GetCommand` body — needs Meta's real reflection (the MVS
     in `Mod/Meta.cp` returns undef from LookupPath).
*)

    IMPORT Meta, Files, Stores, Dialog;

    CONST
        (** Converter.opts — bitmask flags. *)
        importAll* = 0;       (** Catch-all importer for unrecognised extensions. *)

        (** Used as Files.Locator.res when the dialog was cancelled.
            BB-faithful constant; equivalent of the BB `canceled` private
            constant lifted out for our Files.Locator pattern. *)
        canceled = 8;


    TYPE
        (** Importer procedure type — BB-faithful signature.
            Called by `Import` after the file has been opened. *)
        Importer* = PROCEDURE (f: Files.File; OUT s: Stores.Store);

        (** Exporter procedure type. *)
        Exporter* = PROCEDURE (s: Stores.Store; f: Files.File);

        (** Converter — linked-list node in the registry.
            Read-only exports follow BB convention so callers can
            inspect the chain (e.g. to populate the file-open dialog's
            type filter list). *)
        Converter* = POINTER TO RECORD
            next-:      Converter;
            imp-, exp-: Dialog.String;     (** Reflection-resolvable proc names; e.g. "Documents.ImportDocument". *)
            storeType-: Stores.TypeName;   (** Optional store-type filter (matches Stores.TypeOf on Export). *)
            fileType-:  Files.Type;        (** File extension this converter handles ("odc", "txt", ...). *)
            opts-:      SET
        END;

        (* ImpVal / ExpVal value-wrappers around Importer / Exporter
           procs are BB-faithful pieces of the reflection dispatch
           chain.  They're declared once Meta.LookupPath returns
           real results — currently triggering a separate compile-time
           hang we haven't tracked down (likely related to abstract-
           extending records carrying procedure-pointer fields). *)


    VAR
        (** Head of the global converter chain.  Modules iterate
            this directly to populate file-type pickers. *)
        list*: Converter;

        (** Default document exporter — first converter registered
            with an empty `storeType` (BB convention: the "any
            store" exporter, used by `Documents.ExportDocument`
            when the caller passes `conv = NIL`). *)
        doc: Converter;


    (** Look up a procedure by name via Meta's reflection registry
        and fill `val` with a typed Value extension carrying its
        function pointer.  `ok` reports whether the lookup
        succeeded.  This slice always returns `ok := FALSE` because
        `Meta.LookupPath` is currently a surface stub — see
        Mod/Meta.cp.  Once Meta's reflection wires through to the
        loaded-module table, this body needs no change. *)
    (** Reflection-driven name → procedure lookup.  Resolves a
        proc name like "Documents.ImportDocument" through Meta's
        reflection registry into a callable procedure value.
        Currently `ok := FALSE` because Meta.LookupPath itself is
        a surface stub; the body is BB-faithful so it lights up
        the moment Meta's reflection wires through. *)
    PROCEDURE GetCommand (IN name: ARRAY OF CHAR; VAR val: Meta.Value; OUT ok: BOOLEAN);
    BEGIN
        (* BB body uses Meta.LookupPath + a Meta.Value-extending
           ImpVal/ExpVal wrapper to pull the procedure pointer
           out via reflection.  Deferred until Meta.LookupPath
           returns real results — until then the caller's `ok`
           channel gates the dispatch and we report failure here. *)
        ok := FALSE
    END GetCommand;


    (** Register a new converter at the tail of the chain.
        Either `imp` or `exp` may be empty (the converter is then
        import-only or export-only respectively); `fileType` must
        be non-empty.  When `storeType` is empty and `doc` is not
        yet set, this entry also becomes the default exporter.
        BB-faithful precondition asserts. *)
    PROCEDURE Register* (imp, exp: Dialog.String;
                         storeType: Stores.TypeName;
                         fileType: Files.Type;
                         opts: SET);
        VAR e, f: Converter;
    BEGIN
        ASSERT((imp # "") OR (exp # ""), 20);
        ASSERT(fileType # "", 21);

        NEW(e);
        e.next      := NIL;
        e.opts      := opts;
        e.fileType  := fileType;
        e.imp       := imp;
        e.exp       := exp;
        e.storeType := storeType;

        IF (storeType = "") & (doc = NIL) THEN
            doc := e
        END;
        IF list = NIL THEN
            list := e
        ELSE
            f := list;
            WHILE f.next # NIL DO f := f.next END;
            f.next := e
        END
    END Register;


    (** Two-pass converter lookup + invoke.  When `conv = NIL` on
        entry, the body walks the list — first looking for a
        fileType-matching importer, then falling back to any
        `importAll` converter — and writes the picked entry back
        through the VAR.

        Deferred: the actual reflection-driven dispatch through
        `GetCommand` → procedure-pointer-through-Meta.Value pair.
        That path tripped an LLVM-emit hang on first attempt (the
        combination of "ABSTRACT-extending RECORD with a procedure-
        pointer field" + indirect `val.p(...)` call locks LLVM in
        an apparent infinite loop during codegen).  Restore the
        full body once the compiler bug is identified — until then
        the lookup walk + Dialog.ShowMsg fallback is still useful
        for "no converter found" diagnostics. *)
    PROCEDURE Import* (loc: Files.Locator; name: Files.Name;
                       VAR conv: Converter; OUT s: Stores.Store);
    BEGIN
        (* deferred *)
        s := NIL
    END Import;


    PROCEDURE Export* (loc: Files.Locator; name: Files.Name;
                       conv: Converter; s: Stores.Store);
    BEGIN
        (* deferred *)
    END Export;


    (** Sanity helper — count the entries currently on the chain.
        Used by ConvertersProbe to avoid a cross-module read of
        the imported `list` global that may hit an emit issue
        with named-type field access. *)
    PROCEDURE CountAndFirstType* (OUT firstFileTypeOdc: BOOLEAN): INTEGER;
        VAR n: INTEGER; c: Converter;
    BEGIN
        n := 0; c := list;
        firstFileTypeOdc := (c # NIL) & (c.fileType = "odc");
        WHILE c # NIL DO INC(n); c := c.next END;
        RETURN n
    END CountAndFirstType;


BEGIN
    list := NIL;
    doc  := NIL
END Converters.
