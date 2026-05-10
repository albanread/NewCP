DEFINITION MODULE Stores;
(*
   NewCP Stores definition module — Stage S1 (read-only walker).

   Stage S1 surface only: open a `.odc` file, walk its store tree
   by handle, read each store's type name, body length, and wire
   kind. The full BlackBox `Stores` surface (Reader/Writer/Domain/
   Internalize/Externalize/CopyOf/Aliens) lands incrementally in
   later stages — see docs/stores_module_design.md §11.

   Compatibility scope: the legacy `System/Mod/Stores.odc` exposes
   typed `Store`, `Reader`, `Writer`, `Domain` records and abstract
   `Internalize`/`Externalize` methods. NewCP Stage S1 keeps those
   declarations out of the picture entirely — the typed graph is
   S2 work. S1 is here to validate the FFI shape end-to-end before
   we commit to the typed-record design.

   Like the rest of the BlackBox-equivalent surface, the
   implementation lives in the runtime: every procedure declared
   here links at JIT time to a `__newcp_stores_*` Rust function
   (or, equivalently, to the `StoresSys` flat shim under the same
   name). The CP source declares signatures only.

   Handle semantics:
   - `Document` — opaque handle to an open `.odc` file. NIL means
     not open / failed to open. Close with `CloseDocument`.
   - `Store` — opaque handle to a node in a document's store tree.
     NIL means the absent / no-such-node case. Closing a document
     invalidates every Store handle from it.
*)

CONST
    (* Wire-format kind tags returned by GetKind. Match the
       constants in StoresSys; re-exported here so consumers don't
       have to import the lower layer. *)
    KindNil*     = 0;
    KindLink*    = 1;
    KindNewLink* = 2;
    KindStore*   = 3;
    KindElem*    = 4;

TYPE
    Document* = INTEGER;
    Store*    = INTEGER;

PROCEDURE OpenDocument*  (IN path: ARRAY OF CHAR): Document;
    (** Open an `.odc` file. Returns 0 (NIL) on failure: file
        missing, bad CDOo magic, parse error, etc. The caller
        owns the returned handle and must close it. *)

PROCEDURE CloseDocument* (doc: Document);
    (** Release a document and all its store handles. *)

PROCEDURE RootStore*     (doc: Document): Store;
    (** Returns the root store handle, or 0 if the document is
        empty / closed / invalid. The root is conventionally a
        `Documents.StdDocument`. *)

PROCEDURE FirstChild*    (s: Store): Store;
    (** First child store, or 0 if the store has no children
        (text/bytes-only stores like `nil` and `link` always
        return 0). *)

PROCEDURE NextSibling*   (s: Store): Store;
    (** Next sibling at the same level, or 0 if there is no next
        sibling. Combined with `FirstChild` this is enough to
        traverse the entire store tree in DFS order. *)

PROCEDURE GetTypeName*   (s: Store; OUT name: ARRAY OF CHAR);
    (** Writes the qualified type name (e.g. "TextModels.StdModel")
        into `name`, zero-terminated. Empty string for
        nil / link / newlink stores, which carry no type. *)

PROCEDURE GetBodyLen*    (s: Store): INTEGER;
    (** Number of body bytes the store carries. 0 for stores that
        have no body (nil/link/newlink). *)

PROCEDURE GetKind*       (s: Store): INTEGER;
    (** One of the `Kind*` constants above. *)

END Stores.
