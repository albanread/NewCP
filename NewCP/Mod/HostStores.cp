MODULE HostStores;
(*
   Transitional compatibility surface over `Stores`.

   Earlier bring-up work parked the typed reader/materialization path
   here so framework modules could start deserializing before
   `Stores.cp` itself owned the full OO seam.  That ownership has now
   moved back into `Stores`; `HostStores` remains only so older test
   fixtures and temporary callers can import one module name while the
   last call sites are migrated.
*)

IMPORT Stores;

TYPE
    Reader* = Stores.Reader;
    StoreDesc* = Stores.StoreDesc;
    Store* = Stores.Store;

PROCEDURE NewReader* (s: Stores.StoreHandle; VAR rd: Reader);
BEGIN
    Stores.NewReader(s, rd)
END NewReader;

PROCEDURE SplitQualifiedName* (IN q: ARRAY OF CHAR;
                                OUT modName, typeName: ARRAY OF CHAR): BOOLEAN;
BEGIN
    RETURN Stores.SplitQualifiedName(q, modName, typeName)
END SplitQualifiedName;

PROCEDURE NewStoreByName* (IN qualifiedName: ARRAY OF CHAR): Store;
BEGIN
    RETURN Stores.NewStoreByName(qualifiedName)
END NewStoreByName;

PROCEDURE NewLikeOf* (template: Store): Store;
BEGIN
    RETURN Stores.NewLikeOf(template)
END NewLikeOf;

PROCEDURE InternalizeFrom* (src: Stores.StoreHandle; dst: Store): BOOLEAN;
BEGIN
    RETURN Stores.InternalizeFrom(src, dst)
END InternalizeFrom;

PROCEDURE NewStore* (src: Stores.StoreHandle): Store;
BEGIN
    RETURN Stores.NewStore(src)
END NewStore;

END HostStores.
