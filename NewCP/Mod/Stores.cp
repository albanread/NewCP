DEFINITION MODULE Stores;
(*
   NewCP Stores definition module — Stage S1 (read-only walker)
   plus S2 reader cursor primitives. The full typed surface
   (Reader/Writer/Domain records, Internalize/Externalize) is
   still incremental — see docs/stores_module_design.md §11.

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
   - `Reader` — opaque handle to a body cursor (S2). NIL means
     not connected; closing the source document invalidates every
     reader from it.
*)

CONST
    KindNil*     = 0;
    KindLink*    = 1;
    KindNewLink* = 2;
    KindStore*   = 3;
    KindElem*    = 4;

TYPE
    Document* = INTEGER;
    Store*    = INTEGER;
    Reader*   = INTEGER;

PROCEDURE OpenDocument*  (IN path: ARRAY OF CHAR): Document;
PROCEDURE CloseDocument* (doc: Document);

PROCEDURE RootStore*     (doc: Document): Store;
PROCEDURE FirstChild*    (s: Store): Store;
PROCEDURE NextSibling*   (s: Store): Store;

PROCEDURE GetTypeName*   (s: Store; OUT name: ARRAY OF CHAR);
PROCEDURE GetBodyLen*    (s: Store): INTEGER;
PROCEDURE GetKind*       (s: Store): INTEGER;

(* --- S2 reader cursor primitives -------------------------------------- *)

PROCEDURE OpenReader*    (s: Store): Reader;
PROCEDURE CloseReader*   (r: Reader);

PROCEDURE ReaderPos*     (r: Reader): INTEGER;
PROCEDURE ReaderSetPos*  (r: Reader; pos: INTEGER);
PROCEDURE ReaderEof*     (r: Reader): INTEGER;

PROCEDURE ReaderReadByte* (r: Reader): INTEGER;
PROCEDURE ReaderReadInt*  (r: Reader): INTEGER;
PROCEDURE ReaderReadXInt* (r: Reader): INTEGER;
PROCEDURE ReaderReadLong* (r: Reader): INTEGER;
PROCEDURE ReaderReadBool* (r: Reader): INTEGER;
PROCEDURE ReaderReadBytes* (r: Reader; VAR buf: ARRAY OF BYTE; len: INTEGER): INTEGER;

(* --- Inline-child consumption (S2 cont.) ------------------------------
   When the reader's cursor sits exactly at an inline child store's
   first byte (its kind tag), these helpers consume that child:

   - `ReaderSkipInlineStore` advances past it without materializing.
   - `ReaderReadInlineStore` advances past it and returns its
     `Store` handle so callers can pass it to the typed factory. *)

PROCEDURE ReaderSkipInlineStore* (r: Reader): INTEGER;
PROCEDURE ReaderReadInlineStore* (r: Reader): Store;

END Stores.
