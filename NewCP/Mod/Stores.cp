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

    IMPORT SYSTEM, Kernel, StoresSys;

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

    (** Type-name string.  BlackBox-faithful 64-char ARRAY used to
        carry a Store's qualified type name (e.g.
        "TextModels.StdModel") through PollOpsMsg / PollDropMsg /
        the cross-module type registry. *)
    TypeName* = ARRAY 64 OF CHAR;

    (** Opaque writer handle (mirrors `ReaderHandle`).  The runtime
        slots a `WriterState` per handle; 0 = invalid. *)
    WriterHandle* = INTEGER;

    (** Typed reader cursor.  Carries the integer handle plus the
        sticky-eof flag.  Direct field access and the typed
        `ReadByte` / `ReadInt` / … methods below let `Internalize`
        implementations consume primitive fields without touching
        `StoresSys` directly.

        `cancelled` is the BlackBox-faithful sticky-failure flag.
        Once any layer of an Internalize chain sets it (via
        `ReadVersion` finding a stamp out of range, or via
        `TurnIntoAlien`), every subsequent layer bails out without
        further consuming bytes. *)
    Reader* = RECORD
        handle*:    ReaderHandle;
        eof*:       BOOLEAN;
        cancelled*: BOOLEAN
    END;

    (** Typed writer.  Symmetric with Reader; backed by an in-memory
        buffer in the runtime.  `Externalize` implementations call
        the typed `WriteByte` / `WriteInt` / … methods to append
        primitive fields; `Stores.CopyOf` uses the buffer as the
        round-trip carrier between Externalize and Internalize. *)
    Writer* = RECORD
        handle*: WriterHandle
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


(* --- S2 writer cursor primitives -------------------------------------- *)
(* Symmetric with the reader trampolines.  Same thin facade pattern:
   each procedure forwards to the matching `StoresSys.Writer*` shim;
   the typed `Writer` record (and its method-style wrappers below)
   lets callers stay above the integer-handle layer. *)

PROCEDURE NewWriter* (): WriterHandle;
BEGIN RETURN StoresSys.NewWriter() END NewWriter;

PROCEDURE CloseWriter* (w: WriterHandle);
BEGIN StoresSys.CloseWriter(w) END CloseWriter;

PROCEDURE WriterPos* (w: WriterHandle): INTEGER;
BEGIN RETURN StoresSys.WriterPos(w) END WriterPos;

PROCEDURE WriterWriteByte* (w: WriterHandle; b: INTEGER);
BEGIN StoresSys.WriterWriteByte(w, b) END WriterWriteByte;

PROCEDURE WriterWriteInt* (w: WriterHandle; x: INTEGER);
BEGIN StoresSys.WriterWriteInt(w, x) END WriterWriteInt;

PROCEDURE WriterWriteXInt* (w: WriterHandle; x: INTEGER);
BEGIN StoresSys.WriterWriteXInt(w, x) END WriterWriteXInt;

PROCEDURE WriterWriteLong* (w: WriterHandle; x: INTEGER);
BEGIN StoresSys.WriterWriteLong(w, x) END WriterWriteLong;

PROCEDURE WriterWriteBool* (w: WriterHandle; x: INTEGER);
BEGIN StoresSys.WriterWriteBool(w, x) END WriterWriteBool;

PROCEDURE WriterWriteBytes*
    (w: WriterHandle; IN buf: ARRAY OF BYTE; len: INTEGER): INTEGER;
BEGIN RETURN StoresSys.WriterWriteBytes(w, buf, len) END WriterWriteBytes;

(** Consume the writer's accumulated bytes and return a Reader
    anchored at the resulting in-memory buffer.  The writer's own
    buffer is left empty afterwards; clients should still call
    `CloseWriter` to release the handle. *)
PROCEDURE OpenReaderFromWriter* (w: WriterHandle): ReaderHandle;
BEGIN RETURN StoresSys.OpenReaderFromWriter(w) END OpenReaderFromWriter;


(* --- Store-tree cloning ------------------------------------------------- *)

(** Allocate a fresh, zero-initialised Store of the same runtime
    type as `template`.  Mirrors BlackBox `Stores.NewExt`.  Used
    by `CopyOf` to materialise the destination of a clone before
    streaming the source's externalised bytes into it.  Returns
    NIL when `template` is NIL or its type is not registered. *)
PROCEDURE NewExt* (template: Store): Store;
    VAR t: Kernel.Type; s: Store;
BEGIN
    IF template = NIL THEN RETURN NIL END;
    t := Kernel.TypeOf(template);
    IF t = NIL THEN RETURN NIL END;
    Kernel.NewObj(s, t);
    RETURN s
END NewExt;

(** Deep-clone `s` by round-tripping through an in-memory buffer:
    allocate a fresh Store of the same dynamic type via `NewExt`,
    `Externalize` the source into a Writer, hand the buffer over
    to a Reader, and `Internalize` it into the new Store.  Returns
    the new Store (never aliasing `s`), or NIL if `s` is NIL or
    `NewExt` fails.

    This replaces BlackBox's `Stores.CopyOf` and is what
    `Models.CopyOf` (and any other Cut/Copy/Paste-style code)
    should sit on. *)
PROCEDURE CopyOf* (s: Store): Store;
    VAR copy: Store;
        wr:   Writer;
        rd:   Reader;
BEGIN
    IF s = NIL THEN RETURN NIL END;
    copy := NewExt(s);
    IF copy = NIL THEN RETURN NIL END;

    wr.handle := NewWriter();
    s.Externalize(wr);

    rd.handle := OpenReaderFromWriter(wr.handle);
    rd.eof    := FALSE;
    CloseWriter(wr.handle);

    copy.Internalize(rd);
    CloseReader(rd.handle);

    RETURN copy
END CopyOf;


(* --- Typed Reader / Writer methods ------------------------------------ *)
(* BlackBox-faithful method-style API on top of the trampolines.
   Concrete `Internalize` / `Externalize` implementations should
   call these so the integer handle stays an implementation detail.
   `eof` on the Reader is sticky: once a read crosses `body_end`
   the runtime returns 0 / NIL and the next `Eof()` call yields
   TRUE.  We mirror that here by polling the runtime after each
   primitive read so callers can branch on `rd.eof`. *)

PROCEDURE NewReader* (s: StoreHandle; VAR rd: Reader);
BEGIN
    rd.handle := OpenReader(s);
    rd.eof := FALSE;
    rd.cancelled := FALSE
END NewReader;

PROCEDURE (VAR rd: Reader) Close* (), NEW;
BEGIN
    IF rd.handle # 0 THEN
        CloseReader(rd.handle);
        rd.handle := 0
    END;
    rd.eof := TRUE;
    rd.cancelled := TRUE
END Close;

PROCEDURE (rd: Reader) Pos* (): INTEGER, NEW;
BEGIN RETURN ReaderPos(rd.handle) END Pos;

PROCEDURE (VAR rd: Reader) SetPos* (pos: INTEGER), NEW;
BEGIN
    ReaderSetPos(rd.handle, pos);
    rd.eof := ReaderEof(rd.handle) # 0
END SetPos;

PROCEDURE (VAR rd: Reader) ReadByte* (OUT b: BYTE), NEW;
    VAR x, posBefore, posAfter: INTEGER;
BEGIN
    IF rd.eof THEN b := 0X; RETURN END;
    posBefore := ReaderPos(rd.handle);
    x := StoresSys.ReaderReadByte(rd.handle);
    posAfter := ReaderPos(rd.handle);
    IF posAfter = posBefore THEN rd.eof := TRUE; b := 0X; RETURN END;
    b := SHORT(SHORT(SHORT(x)))
END ReadByte;

PROCEDURE (VAR rd: Reader) ReadInt* (OUT x: INTEGER), NEW;
    VAR posBefore, posAfter: INTEGER;
BEGIN
    IF rd.eof THEN x := 0; RETURN END;
    posBefore := ReaderPos(rd.handle);
    x := StoresSys.ReaderReadInt(rd.handle);
    posAfter := ReaderPos(rd.handle);
    IF posAfter = posBefore THEN rd.eof := TRUE; x := 0 END
END ReadInt;

PROCEDURE (VAR rd: Reader) ReadXInt* (OUT x: INTEGER), NEW;
    VAR posBefore, posAfter: INTEGER;
BEGIN
    IF rd.eof THEN x := 0; RETURN END;
    posBefore := ReaderPos(rd.handle);
    x := StoresSys.ReaderReadXInt(rd.handle);
    posAfter := ReaderPos(rd.handle);
    IF posAfter = posBefore THEN rd.eof := TRUE; x := 0 END
END ReadXInt;

PROCEDURE (VAR rd: Reader) ReadLong* (OUT x: INTEGER), NEW;
    VAR posBefore, posAfter: INTEGER;
BEGIN
    IF rd.eof THEN x := 0; RETURN END;
    posBefore := ReaderPos(rd.handle);
    x := StoresSys.ReaderReadLong(rd.handle);
    posAfter := ReaderPos(rd.handle);
    IF posAfter = posBefore THEN rd.eof := TRUE; x := 0 END
END ReadLong;

PROCEDURE (VAR rd: Reader) ReadBool* (OUT b: BOOLEAN), NEW;
    VAR posBefore, posAfter: INTEGER;
BEGIN
    IF rd.eof THEN b := FALSE; RETURN END;
    posBefore := ReaderPos(rd.handle);
    b := StoresSys.ReaderReadBool(rd.handle) # 0;
    posAfter := ReaderPos(rd.handle);
    IF posAfter = posBefore THEN rd.eof := TRUE; b := FALSE END
END ReadBool;

PROCEDURE (VAR rd: Reader) ReadBytes*
    (VAR buf: ARRAY OF BYTE; len: INTEGER), NEW;
    VAR got: INTEGER;
BEGIN
    IF rd.eof OR (len <= 0) THEN RETURN END;
    got := StoresSys.ReaderReadBytes(rd.handle, buf, len);
    IF got # len THEN rd.eof := TRUE END
END ReadBytes;

(** BB-faithful version-stamp reader.  Reads a single byte
    (the version stamp written by `WriteVersion`) and asserts
    it falls in `[min, max]`.  Sets the receiver's `cancelled`
    flag if out of range or already cancelled by an upstream
    layer; subsequent reads in the same chain are no-ops once
    `cancelled` is set.  Mirrors `Stores.Reader.ReadVersion` in
    `System/Mod/Stores.odc.md`. *)
PROCEDURE (VAR rd: Reader) ReadVersion*
    (min, max: INTEGER; OUT thisVersion: INTEGER), NEW;
    VAR b: BYTE;
BEGIN
    thisVersion := 0;
    IF rd.cancelled THEN RETURN END;
    rd.ReadByte(b);
    IF rd.eof THEN rd.cancelled := TRUE; RETURN END;
    thisVersion := b;
    IF (thisVersion < min) OR (thisVersion > max) THEN
        rd.cancelled := TRUE
    END
END ReadVersion;

(** Hard escape on a malformed inline store — set by the
    framework when an Internalize body sees a tag-byte mismatch
    or a wire-format header it can't recover from.  The `reason`
    argument is informational only in this slice; the BlackBox
    original logs it via `Stores.Report`. *)
PROCEDURE (VAR rd: Reader) TurnIntoAlien* (reason: INTEGER), NEW;
BEGIN
    rd.cancelled := TRUE
END TurnIntoAlien;

(** Read an inline child store off the wire.  Returns the
    integer handle of the materialised child (or 0 if the wire
    indicates "nil store" / the read failed).  Sets `cancelled`
    on failure.  Concrete typed-record materialization remains
    on the caller (using `Stores.NewStore` on the returned
    handle) until the runtime grows a direct Reader-to-object RTTI
    factory. *)
PROCEDURE (VAR rd: Reader) ReadStore* (OUT handle: ReaderHandle), NEW;
BEGIN
    handle := StoresSys.ReaderReadInlineStore(rd.handle);
    rd.eof := StoresSys.ReaderEof(rd.handle) # 0;
    IF handle = 0 THEN rd.cancelled := TRUE END
END ReadStore;

(** Skip an inline child store without materializing it.  Used
    when the parent's Internalize hasn't been wired to handle a
    given child slot yet (e.g. TextViews' controller / ruler /
    attributes children before those modules have ported). *)
PROCEDURE (VAR rd: Reader) SkipStore* (), NEW;
    VAR ok: INTEGER;
BEGIN
    ok := StoresSys.ReaderSkipInlineStore(rd.handle);
    IF ok = 0 THEN rd.cancelled := TRUE END;
    rd.eof := StoresSys.ReaderEof(rd.handle) # 0
END SkipStore;

PROCEDURE SplitQualifiedName* (IN q: ARRAY OF CHAR;
                                OUT modName, typeName: ARRAY OF CHAR): BOOLEAN;
    VAR i, j: INTEGER;
BEGIN
    i := 0;
    WHILE (q[i] # 0X) & (q[i] # ".") DO
        modName[i] := q[i];
        INC(i)
    END;
    IF q[i] # "." THEN RETURN FALSE END;
    modName[i] := 0X;
    INC(i);
    j := 0;
    WHILE q[i] # 0X DO
        typeName[j] := q[i];
        INC(i); INC(j)
    END;
    typeName[j] := 0X;
    RETURN (modName[0] # 0X) & (typeName[0] # 0X)
END SplitQualifiedName;

PROCEDURE NewStoreByName* (IN qualifiedName: ARRAY OF CHAR): Store;
    VAR mod: Kernel.Module; t: Kernel.Type; s: Store;
        modName, typeName: Kernel.Name;
BEGIN
    IF ~SplitQualifiedName(qualifiedName, modName, typeName) THEN RETURN NIL END;
    mod := Kernel.ThisMod(modName);
    IF mod = NIL THEN RETURN NIL END;
    t := Kernel.ThisType(mod, typeName);
    IF t = NIL THEN RETURN NIL END;
    Kernel.NewObj(s, t);
    RETURN s
END NewStoreByName;

PROCEDURE NewLikeOf* (template: Store): Store;
BEGIN
    RETURN NewExt(template)
END NewLikeOf;

PROCEDURE InternalizeFrom* (src: StoreHandle; dst: Store): BOOLEAN;
    VAR rd: Reader; eof: BOOLEAN;
BEGIN
    NewReader(src, rd);
    IF rd.handle = 0 THEN RETURN TRUE END;
    dst.Internalize(rd);
    eof := rd.eof;
    rd.Close();
    RETURN eof
END InternalizeFrom;

PROCEDURE NewStore* (src: StoreHandle): Store;
    VAR name: Kernel.Name; s: Store; eof: BOOLEAN;
BEGIN
    IF src = 0 THEN RETURN NIL END;
    GetTypeName(src, name);
    IF name[0] = 0X THEN RETURN NIL END;
    s := NewStoreByName(name);
    IF s = NIL THEN RETURN NIL END;
    eof := InternalizeFrom(src, s);
    IF eof THEN END;
    RETURN s
END NewStore;

(** BB-faithful version-stamp writer.  Symmetric with
    `ReadVersion`. *)
PROCEDURE (VAR wr: Writer) WriteVersion* (version: INTEGER), NEW;
BEGIN
    StoresSys.WriterWriteByte(wr.handle, version)
END WriteVersion;

(** Read / write SET — same i32 wire encoding as INTEGER but
    typed as SET for the caller.  Used by TextRulers and
    elsewhere to round-trip the `opts` masks. *)
PROCEDURE (VAR rd: Reader) ReadSet* (OUT s: SET), NEW;
    VAR x: INTEGER;
BEGIN
    rd.ReadInt(x);
    s := SYSTEM.VAL(SET, x);
END ReadSet;

PROCEDURE (VAR wr: Writer) WriteSet* (s: SET), NEW;
BEGIN
    StoresSys.WriterWriteInt(wr.handle, SYSTEM.VAL(INTEGER, s))
END WriteSet;

PROCEDURE (VAR wr: Writer) WriteByte* (b: BYTE), NEW;
BEGIN StoresSys.WriterWriteByte(wr.handle, b) END WriteByte;

PROCEDURE (VAR wr: Writer) WriteInt* (x: INTEGER), NEW;
BEGIN StoresSys.WriterWriteInt(wr.handle, x) END WriteInt;

PROCEDURE (VAR wr: Writer) WriteXInt* (x: INTEGER), NEW;
BEGIN StoresSys.WriterWriteXInt(wr.handle, x) END WriteXInt;

PROCEDURE (VAR wr: Writer) WriteLong* (x: INTEGER), NEW;
BEGIN StoresSys.WriterWriteLong(wr.handle, x) END WriteLong;

PROCEDURE (VAR wr: Writer) WriteBool* (b: BOOLEAN), NEW;
BEGIN
    IF b THEN StoresSys.WriterWriteBool(wr.handle, 1)
    ELSE StoresSys.WriterWriteBool(wr.handle, 0)
    END
END WriteBool;

PROCEDURE (VAR wr: Writer) WriteBytes*
    (IN buf: ARRAY OF BYTE; len: INTEGER), NEW;
    VAR ignore: INTEGER;
BEGIN ignore := StoresSys.WriterWriteBytes(wr.handle, buf, len) END WriteBytes;

END Stores.
