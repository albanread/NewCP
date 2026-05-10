MODULE TextViewsProbe;
(* End-to-end recursive typed load.  Walks a real `.odc`'s store
   tree, finds the first TextViews.StdViewDesc, materializes it
   via NewStore — which dispatches Internalize, which in turn
   recursively materializes the embedded TextModels.StdModel. *)

IMPORT Stores, HostStores, TextModels, TextViews;

(** DFS through `s` looking for a `TextViews.StdViewDesc`. *)
PROCEDURE FindStdViewIn (s: Stores.StoreHandle): Stores.StoreHandle;
    VAR child, found: Stores.StoreHandle; name: ARRAY 64 OF CHAR;
BEGIN
    IF s = 0 THEN RETURN 0 END;
    Stores.GetTypeName(s, name);
    IF name = "TextViews.StdViewDesc" THEN RETURN s END;
    child := Stores.FirstChild(s);
    WHILE child # 0 DO
        found := FindStdViewIn(child);
        IF found # 0 THEN RETURN found END;
        child := Stores.NextSibling(child)
    END;
    RETURN 0
END FindStdViewIn;

(** Type-resolution sanity. *)
PROCEDURE TypeResolves* (): INTEGER;
    VAR sv: TextViews.StdView;
BEGIN
    NEW(sv);
    IF sv = NIL THEN RETURN 0 END;
    RETURN 1
END TypeResolves;

(** Load a real BlackBox `.odc`, walk to its first
    TextViews.StdViewDesc, materialize it, and verify the typed
    view round-trips:
      - Internalize ran to OkComplete
      - the embedded model is non-NIL
      - the model's own Internalize ran to OkComplete *)
PROCEDURE LoadStdViewFromTourOdc* (): INTEGER;
    VAR doc, viewStore: INTEGER;
        s: HostStores.Store;
        sv: TextViews.StdView;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Tour.odc");
    IF doc = 0 THEN RETURN -1 END;
    viewStore := FindStdViewIn(Stores.RootStore(doc));
    IF viewStore = 0 THEN Stores.CloseDocument(doc); RETURN -2 END;

    s := HostStores.NewStore(viewStore);
    Stores.CloseDocument(doc);
    IF s = NIL THEN RETURN -3 END;

    sv := s(TextViews.StdView);
    IF sv.result # TextViews.OkComplete THEN RETURN -100 - sv.result END;
    IF sv.model = NIL THEN RETURN -50 END;
    IF sv.model.result # TextModels.OkComplete THEN RETURN -200 - sv.model.result END;
    RETURN 1
END LoadStdViewFromTourOdc;

(** Surface a few decoded fields for spot-checking.  Encodes the
    model's text length, the view's org, and dy as a single
    INTEGER:

      textLen * 1_000_000 + (org+1) * 1_000 + (dy+1)

    A non-trivial Tour.odc StdView should have textLen >= 200,
    org >= 0, dy >= 0. *)
PROCEDURE TourStdViewSummary* (): INTEGER;
    VAR doc, viewStore: INTEGER;
        s: HostStores.Store;
        sv: TextViews.StdView;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Tour.odc");
    IF doc = 0 THEN RETURN -1 END;
    viewStore := FindStdViewIn(Stores.RootStore(doc));
    IF viewStore = 0 THEN Stores.CloseDocument(doc); RETURN -2 END;
    s := HostStores.NewStore(viewStore);
    Stores.CloseDocument(doc);
    IF s = NIL THEN RETURN -3 END;
    sv := s(TextViews.StdView);
    IF sv.result # TextViews.OkComplete THEN RETURN -100 - sv.result END;
    IF sv.model = NIL THEN RETURN -50 END;
    RETURN sv.model.textLen * 1000000 + (sv.org + 1) * 1000 + (sv.dy + 1)
END TourStdViewSummary;

END TextViewsProbe.
