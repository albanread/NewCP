MODULE Stores;
(*
   NewCP `Stores` — both the low-level handle facade AND the BlackBox-
   faithful OO surface.  The two coexist:

   - The integer-handle types (StoreHandle / ReaderHandle / Document)
     plus the flat `Stores.*` procedures that operate on them.  These
     trampoline through `StoresSys`, the Rust-hosted shim that walks
     the on-disk `.odc` format.
   - The OO surface (`StoreDesc / Store`, `Reader`, `Writer`,
     `DomainDesc / Domain`, `OperationDesc / Operation`) that every
     framework module above this layer (Models, Views, TextModels,
     etc.) builds on.  StoreDesc carries the abstract serialization
     hooks (`Internalize` / `Externalize` / `CopyFrom` / `Domain`)
     subclasses override and super-call into.

   This is a regular `MODULE` (not `DEFINITION MODULE`) so the JIT
   actually emits bodies for the StoreDesc method stubs — a super
   call from `Models.ModelDesc.Internalize^(rd)` resolves to a real
   `Stores.StoreDesc_Internalize` symbol the loader can publish.
*)

    IMPORT StoresSys;

CONST
    KindNil*     = 0;
    KindLink*    = 1;
    KindNewLink* = 2;
    KindStore*   = 3;
    KindElem*    = 4;

TYPE
    (* -- Low-level handle types ----------------------------------------- *)

    Document*     = INTEGER;
    StoreHandle*  = INTEGER;
    ReaderHandle* = INTEGER;


    (* -- BlackBox-faithful OO surface ----------------------------------- *)

    (** Operation-name buffer.  BlackBox uses a fixed-size 32-char
        ARRAY for undo/redo descriptions ("typing", "delete", "paste",
        ...).  Sequencers passes these into `Do` / `BeginScript` so
        the UI can render an Undo/Redo menu label. *)
    OpName* = ARRAY 32 OF CHAR;

    (** Typed reader cursor.  Carries the legacy integer handle plus
        the sticky-eof flag; richer methods (ReadInt, ReadStore, …)
        will land alongside as the Stores OO surface fills out.  For
        now consumers go through the flat `Stores.ReaderRead*`
        primitives keyed off `rd.handle`. *)
    Reader* = RECORD
        handle*: ReaderHandle;
        eof*:    BOOLEAN
    END;

    (** Typed writer.  Symmetric with Reader; today just a placeholder
        — the writer-side runtime path isn't ported yet, so the field
        list is empty. *)
    Writer* = RECORD
        dummy: INTEGER       (* placeholder so the record has non-zero size *)
    END;

    (** Abstract base for every persistable record.  Models, Views,
        TextModels.StdModelDesc, etc. extend this. *)
    StoreDesc* = ABSTRACT RECORD END;
    Store*     = POINTER TO StoreDesc;

    (** Abstract handle for the Domain that owns a Store tree. *)
    DomainDesc*    = ABSTRACT RECORD END;
    Domain*        = POINTER TO DomainDesc;

    (** Abstract Sequencer operation. *)
    OperationDesc* = ABSTRACT RECORD END;
    Operation*     = POINTER TO OperationDesc;


PROCEDURE (op: Operation) Do*, NEW, ABSTRACT;
    (** Apply (or revert, for undo) the operation.  Concrete
        subclasses encapsulate a single edit and use this method to
        replay it on the underlying store; the `Models.Do` /
        `Models.Undo` paths fall back to calling this directly when
        no Sequencer is installed. *)


(* -- StoreDesc abstract surface ----------------------------------------- *)

PROCEDURE (s: Store) Internalize* (VAR rd: Reader), NEW, EMPTY;
    (** Read this Store's body bytes off `rd` and populate the
        receiver's fields.  The base implementation is empty;
        concrete framework layers (`Models.ModelDesc`,
        `TextModels.StdModelDesc`, etc.) override and chain via
        super calls (`s.Internalize^(rd)`) to read their own
        version stamps before / after their fields. *)

PROCEDURE (s: Store) Externalize* (VAR wr: Writer), NEW, EMPTY;
    (** Symmetric with `Internalize`. *)

PROCEDURE (s: Store) CopyFrom* (source: Store), NEW, EMPTY;
    (** Deep-copy `source`'s fields into the receiver.  Used by
        `Stores.CopyOf` (not yet ported) to clone an entire store
        tree rooted at a model. *)

PROCEDURE (s: Store) Domain* (): Domain, NEW, ABSTRACT;
    (** Return the Domain that owns the receiver. *)


(* -- Low-level handle facade -------------------------------------------- *)
(* Trampolines through StoresSys (the Rust-hosted shim).  Identical
   semantics; the indirection lets sema/IR layer typed records on top
   without losing direct access to the integer handles. *)

PROCEDURE OpenDocument* (IN path: ARRAY OF CHAR): Document;
BEGIN RETURN StoresSys.OpenDocument(path) END OpenDocument;

PROCEDURE CloseDocument* (doc: Document);
BEGIN StoresSys.CloseDocument(doc) END CloseDocument;

PROCEDURE RootStore* (doc: Document): StoreHandle;
BEGIN RETURN StoresSys.RootStore(doc) END RootStore;

PROCEDURE FirstChild* (s: StoreHandle): StoreHandle;
BEGIN RETURN StoresSys.FirstChild(s) END FirstChild;

PROCEDURE NextSibling* (s: StoreHandle): StoreHandle;
BEGIN RETURN StoresSys.NextSibling(s) END NextSibling;

PROCEDURE GetTypeName* (s: StoreHandle; OUT name: ARRAY OF CHAR);
BEGIN StoresSys.GetTypeName(s, name) END GetTypeName;

PROCEDURE GetBodyLen* (s: StoreHandle): INTEGER;
BEGIN RETURN StoresSys.GetBodyLen(s) END GetBodyLen;

PROCEDURE GetKind* (s: StoreHandle): INTEGER;
BEGIN RETURN StoresSys.GetKind(s) END GetKind;


(* --- S2 reader cursor primitives -------------------------------------- *)

PROCEDURE OpenReader* (s: StoreHandle): ReaderHandle;
BEGIN RETURN StoresSys.OpenReader(s) END OpenReader;

PROCEDURE CloseReader* (r: ReaderHandle);
BEGIN StoresSys.CloseReader(r) END CloseReader;

PROCEDURE ReaderPos* (r: ReaderHandle): INTEGER;
BEGIN RETURN StoresSys.ReaderPos(r) END ReaderPos;

PROCEDURE ReaderSetPos* (r: ReaderHandle; pos: INTEGER);
BEGIN StoresSys.ReaderSetPos(r, pos) END ReaderSetPos;

PROCEDURE ReaderEof* (r: ReaderHandle): INTEGER;
BEGIN RETURN StoresSys.ReaderEof(r) END ReaderEof;

PROCEDURE ReaderReadByte* (r: ReaderHandle): INTEGER;
BEGIN RETURN StoresSys.ReaderReadByte(r) END ReaderReadByte;

PROCEDURE ReaderReadInt* (r: ReaderHandle): INTEGER;
BEGIN RETURN StoresSys.ReaderReadInt(r) END ReaderReadInt;

PROCEDURE ReaderReadXInt* (r: ReaderHandle): INTEGER;
BEGIN RETURN StoresSys.ReaderReadXInt(r) END ReaderReadXInt;

PROCEDURE ReaderReadLong* (r: ReaderHandle): INTEGER;
BEGIN RETURN StoresSys.ReaderReadLong(r) END ReaderReadLong;

PROCEDURE ReaderReadBool* (r: ReaderHandle): INTEGER;
BEGIN RETURN StoresSys.ReaderReadBool(r) END ReaderReadBool;

PROCEDURE ReaderReadBytes* (r: ReaderHandle; VAR buf: ARRAY OF BYTE; len: INTEGER): INTEGER;
BEGIN RETURN StoresSys.ReaderReadBytes(r, buf, len) END ReaderReadBytes;


(* --- Inline-child consumption ----------------------------------------- *)

PROCEDURE ReaderSkipInlineStore* (r: ReaderHandle): INTEGER;
BEGIN RETURN StoresSys.ReaderSkipInlineStore(r) END ReaderSkipInlineStore;

PROCEDURE ReaderReadInlineStore* (r: ReaderHandle): StoreHandle;
BEGIN RETURN StoresSys.ReaderReadInlineStore(r) END ReaderReadInlineStore;

END Stores.
