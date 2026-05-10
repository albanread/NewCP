MODULE HostStores;
(*
   Concrete typed surface over the flat handle-based `Stores`
   primitives.  S2 first slice: typed `Reader` so callers can
   write `r.ReadByte(b)` instead of threading integer handles.

   This sits on top of `Stores.cp` the same way `HostFiles` sits
   on `Files`: the abstract module is a runtime-backed DEFINITION
   surface; the Host module ships as compiled CP source and adds
   typed records + methods that wrap the flat handles.  Subsequent
   S2 slices (Store abstract base, NewStore, Internalize dispatch)
   layer onto this same Host module.
*)

IMPORT Kernel, Stores;

TYPE
    ReaderDesc* = RECORD
        handle*: INTEGER;     (** opaque Stores reader handle; 0 = closed/unset *)
        eof*:    BOOLEAN
    END;
    Reader* = POINTER TO ReaderDesc;

    (** Abstract base for every persistable typed store.  Concrete
        view/model records extend StoreDesc and override
        `Internalize`; later S2 slices add `Externalize` and a
        `NewStore` factory that allocates by qualified type name
        and dispatches into the override. *)
    StoreDesc* = ABSTRACT RECORD END;
    Store* = POINTER TO StoreDesc;

(** Allocate a new Reader bound to `s`'s body.  Returns NIL when
    the source store has no body (nil/link/newlink) or is invalid. *)
PROCEDURE NewReader* (s: Stores.StoreHandle): Reader;
    VAR r: Reader; h: INTEGER;
BEGIN
    h := Stores.OpenReader(s);
    IF h = 0 THEN RETURN NIL END;
    NEW(r);
    r.handle := h;
    r.eof := FALSE;
    RETURN r
END NewReader;

(** Release the underlying handle.  Idempotent.  After Close every
    further read sets `r.eof` and returns 0. *)
PROCEDURE (r: ReaderDesc) Close* (), NEW;
BEGIN
    IF r.handle # 0 THEN
        Stores.CloseReader(r.handle);
        r.handle := 0
    END;
    r.eof := TRUE
END Close;

(** Cursor offset within the body (0 = at the first body byte). *)
PROCEDURE (r: ReaderDesc) Pos* (): INTEGER, NEW;
BEGIN RETURN Stores.ReaderPos(r.handle) END Pos;

(** Reposition the cursor; clamped to the body's bounds.  Refreshes
    `r.eof` so callers can rely on the flag after a seek. *)
PROCEDURE (r: ReaderDesc) SetPos* (pos: INTEGER), NEW;
BEGIN
    Stores.ReaderSetPos(r.handle, pos);
    r.eof := Stores.ReaderEof(r.handle) # 0
END SetPos;

(** Reader.eof follows BlackBox semantics: it is sticky only on a
    *failed* read attempt, not on every read that lands at the end.
    A successful read that lands the cursor exactly on `body_end`
    leaves `eof = FALSE`; the next read attempt then fails (cursor
    can't advance further) and `eof` flips to TRUE.  We detect the
    failure by checking that the cursor moved forward — the runtime
    primitive returns 0 on either error or genuine zero data, so
    cursor delta is the unambiguous signal. *)

PROCEDURE (r: ReaderDesc) ReadByte* (OUT x: BYTE), NEW;
    VAR b, posBefore, posAfter: INTEGER;
BEGIN
    IF r.eof THEN x := 0; RETURN END;
    posBefore := Stores.ReaderPos(r.handle);
    b := Stores.ReaderReadByte(r.handle);
    posAfter := Stores.ReaderPos(r.handle);
    IF posAfter = posBefore THEN r.eof := TRUE; x := 0; RETURN END;
    (* INTEGER (i64) -> BYTE (u8): three SHORT() steps in NewCP's
       rank chain (Integer -> IntShort -> ShortInt -> Byte). *)
    x := SHORT(SHORT(SHORT(b)))
END ReadByte;

PROCEDURE (r: ReaderDesc) ReadInt* (OUT x: INTEGER), NEW;
    VAR posBefore, posAfter: INTEGER;
BEGIN
    IF r.eof THEN x := 0; RETURN END;
    posBefore := Stores.ReaderPos(r.handle);
    x := Stores.ReaderReadInt(r.handle);
    posAfter := Stores.ReaderPos(r.handle);
    IF posAfter = posBefore THEN r.eof := TRUE; x := 0 END
END ReadInt;

PROCEDURE (r: ReaderDesc) ReadXInt* (OUT x: INTEGER), NEW;
    VAR posBefore, posAfter: INTEGER;
BEGIN
    IF r.eof THEN x := 0; RETURN END;
    posBefore := Stores.ReaderPos(r.handle);
    x := Stores.ReaderReadXInt(r.handle);
    posAfter := Stores.ReaderPos(r.handle);
    IF posAfter = posBefore THEN r.eof := TRUE; x := 0 END
END ReadXInt;

PROCEDURE (r: ReaderDesc) ReadLong* (OUT x: INTEGER), NEW;
    VAR posBefore, posAfter: INTEGER;
BEGIN
    IF r.eof THEN x := 0; RETURN END;
    posBefore := Stores.ReaderPos(r.handle);
    x := Stores.ReaderReadLong(r.handle);
    posAfter := Stores.ReaderPos(r.handle);
    IF posAfter = posBefore THEN r.eof := TRUE; x := 0 END
END ReadLong;

PROCEDURE (r: ReaderDesc) ReadBool* (OUT x: BOOLEAN), NEW;
    VAR posBefore, posAfter: INTEGER;
BEGIN
    IF r.eof THEN x := FALSE; RETURN END;
    posBefore := Stores.ReaderPos(r.handle);
    x := Stores.ReaderReadBool(r.handle) # 0;
    posAfter := Stores.ReaderPos(r.handle);
    IF posAfter = posBefore THEN r.eof := TRUE; x := FALSE END
END ReadBool;

(** Read `len` bytes into `buf[0..len-1]`.  Sets `r.eof` if fewer
    than `len` bytes were available; the partially-filled prefix
    is preserved unchanged. *)
PROCEDURE (r: ReaderDesc) ReadBytes* (VAR buf: ARRAY OF BYTE; len: INTEGER), NEW;
    VAR got: INTEGER;
BEGIN
    IF r.eof OR (len <= 0) THEN RETURN END;
    got := Stores.ReaderReadBytes(r.handle, buf, len);
    IF got # len THEN r.eof := TRUE END
END ReadBytes;

(** Skip past an inline child store whose header sits at the
    current cursor.  Returns TRUE on success.  Used by Internalize
    methods that recognise an inline-store byte but don't yet
    materialize it (e.g. TextModels.StdModel skipping a NEW
    attribute when the typed `Attributes` record isn't ported
    yet). *)
PROCEDURE (r: ReaderDesc) SkipInlineStore* (): BOOLEAN, NEW;
BEGIN
    IF (r.handle = 0) OR r.eof THEN RETURN FALSE END;
    RETURN Stores.ReaderSkipInlineStore(r.handle) # 0
END SkipInlineStore;

(** Consume an inline child store and return its handle so the
    caller can materialize it via `NewStore`.  Returns 0 (not a
    valid `Stores.StoreHandle`) when the cursor is not on a child
    header — callers should check.  Pairs with `SkipInlineStore`
    when the typed instance isn't needed. *)
PROCEDURE (r: ReaderDesc) ReadInlineStore* (): Stores.StoreHandle, NEW;
BEGIN
    IF (r.handle = 0) OR r.eof THEN RETURN 0 END;
    RETURN Stores.ReaderReadInlineStore(r.handle)
END ReadInlineStore;

(* --- Abstract Store methods ------------------------------------------ *)

(** Read this store's body bytes through `rd` and populate the
    receiver's fields.  Concrete subclasses override; the
    framework calls this once per persisted instance. *)
PROCEDURE (s: StoreDesc) Internalize* (rd: Reader), NEW, ABSTRACT;

(** Split a qualified type name "Module.Type" into the parts.
    Returns FALSE when the input lacks a single dot or either
    side is empty.  Used by NewStoreByName to route into the
    Kernel module registry. *)
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

(** Allocate a new Store whose runtime type is the qualified
    record name `qualifiedName` (e.g. "HostStoresProbe.BytePeekDesc").
    Returns NIL when the name is malformed, the module isn't
    registered with Kernel, the type isn't registered in that
    module, or the allocation itself fails.

    For this lookup to succeed the named module must be visible to
    `Kernel.ThisMod` — i.e. either a runtime native module or a
    compiled CP module whose name has been published into the
    module registry.  Compiled-CP-module auto-registration is the
    deferred loader hook documented in docs/next_milestones.md. *)
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

(** Allocate a new Store with the same runtime type as `template`.
    Useful for `Stores.CopyOf`-style cloning where you have an
    existing instance but not its qualified name.  Returns NIL
    when `template` is NIL or its type is unknown. *)
PROCEDURE NewLikeOf* (template: Store): Store;
    VAR t: Kernel.Type; s: Store;
BEGIN
    IF template = NIL THEN RETURN NIL END;
    t := Kernel.TypeOf(template);
    IF t = NIL THEN RETURN NIL END;
    Kernel.NewObj(s, t);
    RETURN s
END NewLikeOf;

(** Open a Reader on `src`'s body, dispatch through `dst`'s
    Internalize override, and close the reader.  Returns the
    final eof flag — TRUE if reads ran past the body's end. *)
PROCEDURE InternalizeFrom* (src: Stores.StoreHandle; dst: Store): BOOLEAN;
    VAR rd: Reader; eof: BOOLEAN;
BEGIN
    rd := NewReader(src);
    IF rd = NIL THEN RETURN TRUE END;
    dst.Internalize(rd);
    eof := rd.eof;
    rd.Close();
    RETURN eof
END InternalizeFrom;

(** Allocate a typed Store matching `src`'s qualified type tag and
    invoke its `Internalize` override on `src`'s body bytes.
    Returns NIL when the source store's type isn't registered or
    allocation fails; on success the returned object has been
    populated by its own `Internalize` method. *)
PROCEDURE NewStore* (src: Stores.StoreHandle): Store;
    VAR name: Kernel.Name; s: Store; eof: BOOLEAN;
BEGIN
    IF src = 0 THEN RETURN NIL END;
    Stores.GetTypeName(src, name);
    IF name[0] = 0X THEN RETURN NIL END;
    s := NewStoreByName(name);
    IF s = NIL THEN RETURN NIL END;
    eof := InternalizeFrom(src, s);
    (* eof is informational at this layer — callers that care can
       re-read by opening their own Reader. *)
    IF eof THEN END;
    RETURN s
END NewStore;

END HostStores.
