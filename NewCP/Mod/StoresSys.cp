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

(* --- S2 reader cursor primitives -------------------------------------- *)

PROCEDURE OpenReader*      (store: INTEGER): INTEGER;
PROCEDURE CloseReader*     (reader: INTEGER);

PROCEDURE ReaderPos*       (reader: INTEGER): INTEGER;
PROCEDURE ReaderSetPos*    (reader: INTEGER; pos: INTEGER);
PROCEDURE ReaderEof*       (reader: INTEGER): INTEGER;

PROCEDURE ReaderReadByte*  (reader: INTEGER): INTEGER;
PROCEDURE ReaderReadInt*   (reader: INTEGER): INTEGER;   (* 4-byte LE *)
PROCEDURE ReaderReadXInt*  (reader: INTEGER): INTEGER;   (* 2-byte LE *)
PROCEDURE ReaderReadLong*  (reader: INTEGER): INTEGER;   (* 8-byte LE *)
PROCEDURE ReaderReadBool*  (reader: INTEGER): INTEGER;   (* 0/1 *)
PROCEDURE ReaderReadBytes* (reader: INTEGER; VAR buf: ARRAY OF BYTE; len: INTEGER): INTEGER;

(* --- Inline-child-store helpers (S2 cont.) ----------------------------
   When the reader's cursor sits exactly at the start of an inline
   child store of the parent the reader was opened on, these
   helpers consume that child.  They return 0 / NIL when the cursor
   is not on a child header. *)

PROCEDURE ReaderSkipInlineStore* (reader: INTEGER): INTEGER;
    (** Advance past an inline child without materializing it.
        Returns 1 on success, 0 if cursor is not at a child. *)

PROCEDURE ReaderReadInlineStore* (reader: INTEGER): INTEGER;
    (** Consume an inline child and return its `Store` handle.
        Returns 0 if cursor is not at a child. *)


(* --- S2 in-memory writer (round-trip backing for Stores.CopyOf) ------
   Symmetric with the reader primitives.  All writes are append-only;
   the wire format is little-endian for multi-byte integers, matching
   what the matching `ReaderRead*` consumes.  `OpenReaderFromWriter`
   moves the writer's accumulated bytes into a fresh in-memory buffer
   and returns a Reader anchored at that buffer's first byte; the
   buffer is released when the Reader is closed. *)

PROCEDURE NewWriter*        (): INTEGER;
PROCEDURE CloseWriter*      (writer: INTEGER);
PROCEDURE WriterPos*        (writer: INTEGER): INTEGER;
PROCEDURE WriterWriteByte*  (writer: INTEGER; b: INTEGER);
PROCEDURE WriterWriteInt*   (writer: INTEGER; x: INTEGER);   (* 4-byte LE *)
PROCEDURE WriterWriteXInt*  (writer: INTEGER; x: INTEGER);   (* 2-byte LE *)
PROCEDURE WriterWriteLong*  (writer: INTEGER; x: INTEGER);   (* 8-byte LE *)
PROCEDURE WriterWriteBool*  (writer: INTEGER; x: INTEGER);   (* 0/1 *)
PROCEDURE WriterWriteBytes* (writer: INTEGER; IN buf: ARRAY OF BYTE; len: INTEGER): INTEGER;

PROCEDURE OpenReaderFromWriter* (writer: INTEGER): INTEGER;

END StoresSys.
