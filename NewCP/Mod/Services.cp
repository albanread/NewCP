MODULE Services;
(*
   NewCP port of BlackBox `System/Mod/Services.odc`.

   Three responsibilities:

   1. Per-tick wall-clock helpers (`Ticks` returning millisecond
      monotonic time).

   2. A typed `Action` list — abstract records implementing a `Do`
      method are scheduled via `DoLater(action, when)` and run
      when their `notBefore` deadline elapses.

   3. An `ActionHook` indirection so the host's idle loop can
      `Step` (drain due actions) and `Loop` (run the loop until
      a quit condition).

   4. RTTI surface — `TypeLevel`, `TypeLevel`, `SameType`,
      `IsExtensionOf`, `Is`, `Extends`, `Level`, `AdrOf`,
      `GetTypeName`, `Collect` — wrappers around `Kernel`'s
      opaque type API.

   Divergences from BlackBox:
   - `ANYREC` parameters replaced by `ANYPTR` for the RTTI
     procedures; this matches all actual call sites in the
     framework (Properties.cp passes pointers, not record vars).
   - No module-unload guard in `Exec`; our runtime does not
     support dynamic unloading in this slice.
   - `Kernel.trapCount` field → `Kernel.TrapCount()` function.
*)

    IMPORT SYSTEM, Kernel;

    CONST
        (** `DoLater` notBefore sentinels. *)
        now*         = 0;
        immediately* = -1;

        (** Internal time unit — `Ticks` returns milliseconds. *)
        resolution* = 1000;

        scale = resolution DIV Kernel.timeResolution;
        corr  = resolution MOD Kernel.timeResolution;


    TYPE
        (** Abstract deferred-action record. *)
        ActionDesc* = ABSTRACT RECORD
            notBefore-: INTEGER;
            next-:      Action
        END;
        Action* = POINTER TO ActionDesc;

        (** Abstract idle-loop hook. *)
        ActionHookDesc* = ABSTRACT RECORD (Kernel.HookDesc) END;
        ActionHook*     = POINTER TO ActionHookDesc;

        (** Standard (default) hook — calls IterateOverActions. *)
        StdHookDesc = RECORD (ActionHookDesc) END;
        StdHook     = POINTER TO StdHookDesc;


    VAR
        (** Installed hook; `Step*` delegates to it. *)
        actionHook-:   ActionHook;

        (** Head of the pending-actions list. *)
        actions:       Action;

        (** Actions being iterated in the current `Step`.  NIL
            outside of IterateOverActions.  Used by Cleanup to
            recover from a trap inside `a.Do`. *)
        candidates:    Action;

        hasImmediates: BOOLEAN;

        (** Snapshot of the trap counter; updated by Cleanup. *)
        trapCnt-:      INTEGER;


    (* ---- Ticks ----------------------------------------------------------- *)

    (** Monotonic millisecond clock. *)
    PROCEDURE Ticks* (): INTEGER;
        VAR t: INTEGER;
    BEGIN
        t := Kernel.Time();
        RETURN t * scale + t * corr DIV Kernel.timeResolution
    END Ticks;


    (* ---- Action protocol ------------------------------------------------- *)

    (** Run the action. ABSTRACT — concrete actions implement work. *)
    PROCEDURE (a: ActionDesc) Do*, NEW, ABSTRACT;


    (* ---- Action-list bookkeeping ----------------------------------------- *)

    PROCEDURE In (head, a: Action): BOOLEAN;
    BEGIN
        WHILE (head # NIL) & (head # a) DO head := head.next END;
        RETURN head # NIL
    END In;

    PROCEDURE Incl (VAR list: Action; a: Action);
    BEGIN
        ASSERT(a # NIL, 20);
        ASSERT(~In(list, a), 21);
        IF list # NIL THEN a.next := list END;
        list := a
    END Incl;

    PROCEDURE Excl (VAR list: Action; a: Action);
        VAR cursor: Action;
    BEGIN
        IF list = NIL THEN RETURN END;
        IF list = a THEN
            list := a.next;
            a.next := NIL;
            RETURN
        END;
        cursor := list;
        WHILE (cursor.next # NIL) & (cursor.next # a) DO
            cursor := cursor.next
        END;
        IF cursor.next = a THEN
            cursor.next := a.next;
            a.next := NIL
        END
    END Excl;


    (* ---- Trap recovery --------------------------------------------------- *)

    (** Called at the start of IterateOverActions.  If a trap fired
        inside a prior `Do` call, `candidates` may still hold the
        unprocessed tail; prepend it back to `actions`. *)
    PROCEDURE Cleanup;
        VAR p: Action;
    BEGIN
        IF candidates # NIL THEN
            p := candidates;
            WHILE p.next # NIL DO p := p.next END;
            p.next := actions; actions := candidates; candidates := NIL
        END;
        trapCnt := Kernel.TrapCount()
    END Cleanup;


    (* ---- Core iteration -------------------------------------------------- *)

    PROCEDURE IterateOverActions (time: INTEGER);
        VAR p: Action;
    BEGIN
        Cleanup;
        (* Move all pending to candidates, then dispatch. *)
        candidates := actions; actions := NIL;
        WHILE candidates # NIL DO
            p := candidates; candidates := p.next;
            IF (0 <= p.notBefore) & (p.notBefore <= time) OR
               (p.notBefore = immediately) THEN
                p.next := NIL;
                p.Do
            ELSE
                p.next := actions; actions := p
            END
        END
    END IterateOverActions;


    (* ---- ActionHook abstract interface ----------------------------------- *)

    (** Step — run all due actions.  ABSTRACT. *)
    PROCEDURE (h: ActionHookDesc) Step*, NEW, ABSTRACT;

    (** Loop — drain immediate actions.  ABSTRACT. *)
    PROCEDURE (h: ActionHookDesc) Loop*, NEW, ABSTRACT;


    (* ---- StdHook concrete implementations -------------------------------- *)

    PROCEDURE (h: StdHookDesc) Step*;
    BEGIN
        IF (candidates = NIL) OR (trapCnt < Kernel.TrapCount()) THEN
            IterateOverActions(Ticks())
        END
    END Step;

    PROCEDURE (h: StdHookDesc) Loop*;
    BEGIN
        IF hasImmediates THEN
            ASSERT((candidates = NIL) OR (trapCnt < Kernel.TrapCount()), 100);
            IterateOverActions(immediately);
            hasImmediates := FALSE
        END
    END Loop;


    (* ---- Public scheduling API ------------------------------------------- *)

    (** Schedule `a` to run no earlier than `notBefore` ticks.
        Re-scheduling an already-pending action updates its deadline. *)
    PROCEDURE DoLater* (a: Action; notBefore: INTEGER);
    BEGIN
        ASSERT(a # NIL, 20);
        IF ~In(actions, a) & ~In(candidates, a) THEN
            Incl(actions, a)
        END;
        a.notBefore := notBefore;
        IF notBefore = immediately THEN hasImmediates := TRUE END
    END DoLater;

    (** Cancel a pending action.  Idempotent. *)
    PROCEDURE RemoveAction* (a: Action);
    BEGIN
        IF a # NIL THEN
            Excl(actions, a);
            Excl(candidates, a)
        END
    END RemoveAction;

    (** Run all due actions via the installed hook. *)
    PROCEDURE Step*;
    BEGIN
        IF actionHook # NIL THEN actionHook.Step() END
    END Step;

    (** True if any pending action has `immediately` deadline. *)
    PROCEDURE HasImmediates* (): BOOLEAN;
    BEGIN
        RETURN hasImmediates
    END HasImmediates;


    (* ---- RTTI helpers ---------------------------------------------------- *)

    (** Split "Module.TypeName" and look up the Kernel.Type.
        Returns NIL if either part is missing or the type cannot
        be resolved. *)
    PROCEDURE ThisDesc (IN type: ARRAY OF CHAR): Kernel.Type;
        VAR m: Kernel.Module; t: Kernel.Type;
            mod: ARRAY 256 OF CHAR;
            typName: Kernel.Name;
            i, j: INTEGER; ch: CHAR;
    BEGIN
        i := 0; ch := type[0];
        WHILE (ch # '.') & (ch # 0X) DO
            mod[i] := ch; INC(i); ch := type[i]
        END;
        IF ch # '.' THEN RETURN NIL END;
        mod[i] := 0X; INC(i);
        j := 0;
        REPEAT ch := type[i]; typName[j] := ch; INC(i); INC(j) UNTIL ch = 0X;
        m := Kernel.ThisMod(mod);
        IF m = NIL THEN RETURN NIL END;
        RETURN Kernel.ThisType(m, typName)
    END ThisDesc;


    (* ---- RTTI public API ------------------------------------------------- *)

    (** Qualified type name (e.g. "TextModels.Doc") for `obj`. *)
    PROCEDURE GetTypeName* (obj: ANYPTR; OUT type: ARRAY OF CHAR);
    BEGIN
        Kernel.GetQualifiedTypeName(Kernel.TypeOf(obj), type)
    END GetTypeName;

    (** TRUE iff `a` and `b` are the same dynamic type. *)
    PROCEDURE SameType* (a, b: ANYPTR): BOOLEAN;
    BEGIN
        RETURN Kernel.TypeOf(a) = Kernel.TypeOf(b)
    END SameType;

    (** TRUE iff the dynamic type of `a` extends the dynamic type
        of `b` (reflexive: `a` and `b` of same type returns TRUE). *)
    PROCEDURE IsExtensionOf* (a, b: ANYPTR): BOOLEAN;
        VAR ta, tb, tx: Kernel.Type;
    BEGIN
        ta := Kernel.TypeOf(a); tb := Kernel.TypeOf(b);
        tx := ta;
        WHILE (tx # NIL) & (tx # tb) DO tx := Kernel.BaseOf(tx) END;
        RETURN tx = tb
    END IsExtensionOf;

    (** TRUE iff `obj`'s dynamic type is an extension of the named
        type (format "Module.RecordName"). *)
    PROCEDURE Is* (obj: ANYPTR; IN type: ARRAY OF CHAR): BOOLEAN;
        VAR ta, tb, tx: Kernel.Type;
    BEGIN
        ta := Kernel.TypeOf(obj);
        tb := ThisDesc(type);
        IF tb = NIL THEN RETURN FALSE END;
        tx := ta;
        WHILE (tx # NIL) & (tx # tb) DO tx := Kernel.BaseOf(tx) END;
        RETURN tx = tb
    END Is;

    (** TRUE iff the named type `type` extends named type `base`. *)
    PROCEDURE Extends* (IN type, base: ARRAY OF CHAR): BOOLEAN;
        VAR ta, tb, tx: Kernel.Type;
    BEGIN
        ASSERT((type[0] # 0X) & (base[0] # 0X), 20);
        ta := ThisDesc(type);
        tb := ThisDesc(base);
        IF (ta = NIL) OR (tb = NIL) THEN RETURN FALSE END;
        tx := ta;
        WHILE (tx # NIL) & (tx # tb) DO tx := Kernel.BaseOf(tx) END;
        RETURN tx = tb
    END Extends;

    (** Inheritance depth of the named type (0 = root). *)
    PROCEDURE Level* (IN type: ARRAY OF CHAR): INTEGER;
        VAR t: Kernel.Type;
    BEGIN
        t := ThisDesc(type);
        IF t = NIL THEN RETURN -1 END;
        RETURN Kernel.LevelOf(t)
    END Level;

    (** Inheritance depth of `obj`'s dynamic type.
        Note: signature uses ANYPTR (not ANYREC as in BlackBox)
        because all call sites in the framework pass pointers. *)
    PROCEDURE TypeLevel* (obj: ANYPTR): INTEGER;
        VAR t: Kernel.Type;
    BEGIN
        t := Kernel.TypeOf(obj);
        IF t = NIL THEN RETURN -1
        ELSE RETURN Kernel.LevelOf(t)
        END
    END TypeLevel;

    (** Address of `obj`'s type descriptor, usable as a stable
        sort key.  (BlackBox: SYSTEM.ADR of ANYREC variable.) *)
    PROCEDURE AdrOf* (obj: ANYPTR): INTEGER;
        VAR t: Kernel.Type;
    BEGIN
        t := Kernel.TypeOf(obj);
        IF t = NIL THEN RETURN 0 END;
        RETURN SYSTEM.VAL(INTEGER, t)
    END AdrOf;

    (** Trigger a GC cycle. *)
    PROCEDURE Collect*;
    BEGIN
        Kernel.Collect()
    END Collect;


    (* ---- Init ------------------------------------------------------------ *)

    PROCEDURE Init;
        VAR h: StdHook;
    BEGIN
        NEW(h); actionHook := h;
        actions := NIL; candidates := NIL;
        hasImmediates := FALSE; trapCnt := 0
    END Init;

BEGIN
    Init
END Services.
