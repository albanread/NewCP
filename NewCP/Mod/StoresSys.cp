DEFINITION MODULE StoresSys;
(**
   Flat C-ABI .odc envelope walker, backed by Rust's `newcp-odc`
   wire codec (see src/newcp-runtime/src/stores_sys.rs). This is
   Stores Stage S1: read-only navigation of the store tree, no
   typed `Stores.Store` instances yet.

   Conventions:
   - Document and store handles are opaque INTEGER values; 0 means
     "invalid / not open / NIL".
   - Store handles encode (document, node-index); they're meaningful
     only against the document that produced them. Closing the
     document invalidates every store handle from it.
   - Strings flow through `OUT name: ARRAY OF CHAR` (UTF-32, zero-
     terminated, capped at the array's element count).
   - `GetKind` returns one of the constants below.

   Direct CP clients should normally use `Stores.cp` instead;
   `StoresSys` is the low-level layer.
*)

CONST
    (* Wire-format kind tags returned by GetKind. *)
    KindNil*     = 0;
    KindLink*    = 1;
    KindNewLink* = 2;
    KindStore*   = 3;
    KindElem*    = 4;

PROCEDURE OpenDocument*  (IN path: ARRAY OF CHAR): INTEGER;
PROCEDURE CloseDocument* (handle: INTEGER);

PROCEDURE RootStore*     (handle: INTEGER): INTEGER;
PROCEDURE FirstChild*    (store: INTEGER): INTEGER;
PROCEDURE NextSibling*   (store: INTEGER): INTEGER;

PROCEDURE GetTypeName*   (store: INTEGER; OUT name: ARRAY OF CHAR);
PROCEDURE GetBodyLen*    (store: INTEGER): INTEGER;
PROCEDURE GetKind*       (store: INTEGER): INTEGER;

END StoresSys.
