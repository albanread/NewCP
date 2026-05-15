MODULE Meta;
(*
   First slice of the BlackBox `Meta` port — minimal surface for
   Converters / Documents / Windows / StdCmds to compile against.

   BB's `Meta` is a 2300-line runtime-reflection registry that
   resolves names like "Documents.ImportDocument" to a callable
   procedure at run time.  This slice ships only the surface
   directly referenced by first-wave consumers; bodies fall back
   to "not found" so the dependency chain compiles and links.
   Real reflection lands in a follow-up once we wire in the
   Kernel-side module/type walkers.

   Deferred: every value-accessor method on `Item`
   (`GetVal`/`PutVal`/`IntVal`/`CharVal`/…), the `Scanner` type
   for module-table iteration, the `LookupFilter` hook chain,
   and `Call`/`ParamCall` for reflection-driven invocation.
*)

    IMPORT Kernel;

    CONST
        undef* = 0;

        (** object classes *)
        typObj*   = 2;
        varObj*   = 3;
        procObj*  = 4;
        fieldObj* = 5;
        modObj*   = 6;
        parObj*   = 7;

        (** type classes *)
        boolTyp*    = 1;
        sCharTyp*   = 2;
        charTyp*    = 3;
        byteTyp*    = 4;
        sIntTyp*    = 5;
        intTyp*     = 6;
        sRealTyp*   = 7;
        realTyp*    = 8;
        setTyp*     = 9;
        longTyp*    = 10;
        anyRecTyp*  = 11;
        anyPtrTyp*  = 12;
        procTyp*    = 16;
        recTyp*     = 17;
        arrTyp*     = 18;
        ptrTyp*     = 19;

        (** record attributes *)
        final*      = 0;
        extensible* = 1;
        limited*    = 2;
        abstract*   = 3;

        (** visibility *)
        hidden*   = 1;
        readOnly* = 2;
        exported* = 4;

        (** parameter kinds (also stored in Item.vis on parObj) *)
        value* = 10;
        in*    = 11;
        out*   = 12;
        var*   = 13;


    TYPE
        (** BB-faithful name buffer. *)
        Name* = ARRAY 256 OF CHAR;

        (** Abstract Value — callers extend with a single typed
            field (`Converters.ImpVal`, `ExpVal`) so reflected
            calls can route data through the typed slot. *)
        Value* = ABSTRACT RECORD END;

        (** Reflection cursor.  Fields BB-faithful so consumers
            can pattern-match on `obj` / `typ` / `vis`. *)
        Item* = RECORD (Value)
            obj-:  INTEGER;
            typ-:  INTEGER;
            vis-:  INTEGER;
            adr-:  INTEGER;
            mod:   Kernel.Module;
            desc:  Kernel.Type
        END;

        (** BB's filter-hook proc type — declared for surface
            compatibility; the hook chain isn't invoked in this
            slice. *)
        LookupFilter* = PROCEDURE (IN path: ARRAY OF CHAR; OUT i: Item; OUT done: BOOLEAN);


    (** Look up a module by name.  Deferred — always returns
        an undef Item until the Kernel-side `ThisMod` wire-up
        lands. *)
    PROCEDURE Lookup* (IN name: ARRAY OF CHAR; OUT mod: Item);
    BEGIN
        mod.obj := undef;
        mod.typ := undef;
        mod.vis := undef;
        mod.adr := undef;
        mod.mod := NIL;
        mod.desc := NIL
    END Lookup;

    (** Resolve a qualified path like `"Documents.ImportDocument"`.
        Deferred — always returns undef in this slice. *)
    PROCEDURE LookupPath* (IN path: ARRAY OF CHAR; OUT i: Item);
    BEGIN
        i.obj := undef;
        i.typ := undef;
        i.vis := undef;
        i.adr := undef;
        i.mod := NIL;
        i.desc := NIL
    END LookupPath;

    (** Wrap a heap-allocated record in an Item.  Deferred. *)
    PROCEDURE GetItem* (obj: ANYPTR; OUT i: Item);
    BEGIN
        i.obj := undef;
        i.typ := undef;
        i.vis := undef;
        i.adr := undef;
        i.mod := NIL;
        i.desc := NIL
    END GetItem;

    (** Install a filter hook.  Deferred. *)
    PROCEDURE InstallFilter* (filter: LookupFilter);
    BEGIN
    END InstallFilter;

    (** Drop a filter hook.  Deferred. *)
    PROCEDURE UninstallFilter* (filter: LookupFilter);
    BEGIN
    END UninstallFilter;


    (* -- Item methods ----------------------------------------------------- *)

    (** TRUE iff the Item refers to a successfully-resolved object. *)
    PROCEDURE (VAR i: Item) Valid* (): BOOLEAN, NEW;
    BEGIN
        RETURN i.obj # undef
    END Valid;

    (** Look up `name` within this Item's scope.  Deferred. *)
    PROCEDURE (VAR in: Item) Lookup* (IN name: ARRAY OF CHAR; VAR i: Item), NEW;
    BEGIN
        i.obj := undef; i.typ := undef; i.vis := undef; i.adr := undef;
        i.mod := NIL; i.desc := NIL
    END Lookup;

    (** Fill `OUT mod, type` with the qualified type name.  Deferred. *)
    PROCEDURE (VAR i: Item) GetTypeName* (OUT mod, type: Name), NEW;
    BEGIN
        mod[0] := 0X; type[0] := 0X
    END GetTypeName;

    (** Read the value of a varObj into a Value extension.
        Deferred — when Meta.LookupPath returns undef the
        caller's `ok` channel already gates the access. *)
    PROCEDURE (VAR var: Item) GetVal* (VAR x: Value; OUT ok: BOOLEAN), NEW;
    BEGIN
        ok := FALSE
    END GetVal;

    (** Write a Value extension into a varObj.  Deferred. *)
    PROCEDURE (VAR var: Item) PutVal* (IN x: Value; OUT ok: BOOLEAN), NEW;
    BEGIN
        ok := FALSE
    END PutVal;

    (** Read a string varObj into `x`.  Deferred. *)
    PROCEDURE (VAR var: Item) GetStringVal* (OUT x: ARRAY OF CHAR; OUT ok: BOOLEAN), NEW;
    BEGIN
        x[0] := 0X; ok := FALSE
    END GetStringVal;

    (** Write a string into a string varObj.  Deferred. *)
    PROCEDURE (VAR var: Item) PutStringVal* (IN x: ARRAY OF CHAR; OUT ok: BOOLEAN), NEW;
    BEGIN
        ok := FALSE
    END PutStringVal;


END Meta.
