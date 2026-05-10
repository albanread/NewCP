MODULE TextViews;
(*
   First slice of the BlackBox `TextViews` port.

   `StdView` is the on-disk representation of a visible text
   editor pane. Its body interleaves version bytes and inline
   child stores in a fixed sequence:

     1 byte    Stores.Store version
     1 byte    Views.View version
     1 byte    Containers.View version
     store     inline Model  (a TextModels.StdModel)
     store     inline Controller  (may be a nil store)
     1 byte    TextViews.View Internalize2 version
     1 byte    TextViews.StdView Internalize2 version
     1 byte    hideMarks BOOLEAN
     store     inline default Ruler  (may be nil)
     store     inline default Attributes  (may be nil)
     4 bytes   org  INTEGER  (top-of-view character offset)
     4 bytes   dy   INTEGER  (sub-line scroll offset, in pixels)

   This slice materializes the embedded `TextModels.StdModel`
   typed instance via `Reader.ReadInlineStore` followed by
   `HostStores.NewStore`, and skips past the controller / ruler /
   attributes children (which need their own typed records,
   slated for later slices).

   See newcp-odc/src/text_views.rs for the canonical wire-format
   specification.
*)

IMPORT HostStores, TextModels;

CONST
    OkComplete*           = 0;
    OkSuperVersionsTrunc* = 1;
    OkModelMissing*       = 2;
    OkModelLoadFailed*    = 3;
    OkControllerSkipFail* = 4;
    OkInternalize2Trunc*  = 5;
    OkRulerSkipFail*      = 6;
    OkAttrsSkipFail*      = 7;
    OkTrailerTrunc*       = 8;

TYPE
    StdViewDesc* = RECORD (HostStores.StoreDesc)
        (** Stage-1 super-class version chain (Stores.Store +
            Views.View + Containers.View). *)
        v1*: ARRAY 3 OF BYTE;
        (** Stage-2 (`Internalize2`) super-class version bytes
            (TextViews.View + TextViews.StdView). *)
        v2*: ARRAY 2 OF BYTE;
        hideMarks*: BOOLEAN;
        (** Top-of-view character offset; sub-line dy in pixels. *)
        org*: INTEGER;
        dy*:  INTEGER;
        (** Materialized child model. NIL when the on-disk view
            doesn't carry a model (rare) or when materialization
            failed; in the latter case `result` reflects the
            specific failure. *)
        model*: TextModels.StdModel;

        result*: INTEGER
    END;
    StdView* = POINTER TO StdViewDesc;

PROCEDURE (v: StdViewDesc) Internalize* (rd: HostStores.Reader);
    VAR i: INTEGER;
        b: BYTE;
        modelStoreHandle: INTEGER;
        modelStore: HostStores.Store;
        bool: BOOLEAN;
BEGIN
    v.model := NIL;
    v.org := 0;
    v.dy := 0;
    v.hideMarks := FALSE;
    v.result := OkSuperVersionsTrunc;

    (* Stage-1 super-class version chain. *)
    i := 0;
    WHILE i < 3 DO
        rd.ReadByte(b);
        IF rd.eof THEN RETURN END;
        v.v1[i] := b;
        INC(i)
    END;

    (* Inline Model store. *)
    v.result := OkModelMissing;
    modelStoreHandle := rd.ReadInlineStore();
    IF modelStoreHandle = 0 THEN RETURN END;

    v.result := OkModelLoadFailed;
    modelStore := HostStores.NewStore(modelStoreHandle);
    IF modelStore = NIL THEN RETURN END;
    (* Type-guard down to a concrete StdModel — anything else is
       a wire-format mismatch we don't handle yet. *)
    v.model := modelStore(TextModels.StdModel);

    (* Inline Controller store — skip without materializing. *)
    v.result := OkControllerSkipFail;
    IF ~rd.SkipInlineStore() THEN RETURN END;

    (* Stage-2 version bytes. *)
    v.result := OkInternalize2Trunc;
    i := 0;
    WHILE i < 2 DO
        rd.ReadByte(b);
        IF rd.eof THEN RETURN END;
        v.v2[i] := b;
        INC(i)
    END;

    (* hideMarks BOOLEAN (1 byte on the wire). *)
    rd.ReadBool(bool);
    IF rd.eof THEN RETURN END;
    v.hideMarks := bool;

    (* Default Ruler — skip. *)
    v.result := OkRulerSkipFail;
    IF ~rd.SkipInlineStore() THEN RETURN END;

    (* Default Attributes — skip. *)
    v.result := OkAttrsSkipFail;
    IF ~rd.SkipInlineStore() THEN RETURN END;

    (* org / dy trailer. *)
    v.result := OkTrailerTrunc;
    rd.ReadInt(v.org);
    IF rd.eof THEN RETURN END;
    rd.ReadInt(v.dy);
    IF rd.eof THEN RETURN END;

    v.result := OkComplete
END Internalize;

END TextViews.
