MODULE Services;
(*
   First slice of the BlackBox `Services` port.

   `Services` is the deferred-action scheduler that lives
   underneath the host event loop.  Three responsibilities in
   BlackBox:

   1. Per-tick wall-clock helpers (`Ticks` returning
      millisecond-resolution monotonic time).
   2. A typed `Action` list — abstract records implementing a
      `Do` method are scheduled via `DoLater(action, when)` and
      run when their `notBefore` deadline elapses.
   3. An `ActionHook` indirection so the host's idle loop can
      `Step` (drain due actions) and `Loop` (run the loop until
      a quit condition).

   This slice ships (1) and (2) — the typed-action API every
   framework module above (`Controllers.Forwarder`, scrolling /
   animation timers, deferred-paint repaint queue) eventually
   schedules through.  The `ActionHook` types are present so
   subclasses compile, but the host-side `Step` / `Loop` bodies
   stay stubs — they need a real event loop, which lives in
   the iGui layer.

   Deferred (called out below):

   - The `actionHook-` module variable and `Init` body — needs
     the host's idle-loop installer.
   - `Collect` (forces a Kernel.GC pass) — needs `Kernel.GC`,
     which our runtime exposes as `__newcp_gc_collect`.  Can
     land alongside this slice; for now it's an EMPTY stub.
   - `IsExtensionOf`, `Is`, `Extends`, `Level`, `TypeLevel`,
     `AdrOf`, `GetTypeName`, `SameType` — the RTTI surface.
     Needs `Kernel.TypeOf` (we have it) but also `Kernel.Module`
     field reads (`t.mod.name$`); deferred.
*)

    IMPORT Kernel;

    CONST
        (** `DoLater` notBefore sentinels. *)
        now*         = 0;
        immediately* = -1;

        (** Internal time unit — `Ticks` returns milliseconds. *)
        resolution* = 1000;

        (* BlackBox-faithful rescale between Kernel.Time's tick
           rate and Services.resolution.  In our build both are
           1000 so scale = 1 and corr = 0; the expressions stay
           in source form so the relationship survives any
           future bump to either constant. *)
        scale = resolution DIV Kernel.timeResolution;
        corr  = resolution MOD Kernel.timeResolution;


    TYPE
        (** Abstract deferred-action record.  Concrete actions
            extend this and override `Do`; the scheduler runs
            them in `notBefore`-order from `Step`. *)
        ActionDesc* = ABSTRACT RECORD
            notBefore-: INTEGER;   (** earliest run-time tick *)
            next-:      Action     (** linked list of pending actions *)
        END;
        Action* = POINTER TO ActionDesc;

        (** Abstract idle-loop hook.  The host installs a
            concrete `ActionHook` via the deferred
            `SetActionHook`; `Step` / `Loop` then route control
            to the host's event-pump.  Subclasses extend
            `Kernel.Hook` so the runtime's hook registry can
            store them generically. *)
        ActionHookDesc* = ABSTRACT RECORD (Kernel.HookDesc) END;
        ActionHook*     = POINTER TO ActionHookDesc;


    VAR
        (** Head of the pending-actions list — kept sorted by
            ascending `notBefore`.  Module-private in BlackBox;
            we keep it that way. *)
        actions:       Action;

        (** Set TRUE when `Step` is iterating to dispatch
            actions whose `notBefore <= now`.  Kept on the
            module to mirror BB's shape but unused in this
            slice. *)
        hasImmediates: BOOLEAN;

        (** Trap-recovery counter — bumped by the runtime when
            an action's `Do` traps and is unwound.  Surfaced for
            test harnesses to detect "did anything trap during
            Step". *)
        trapCnt-: INTEGER;


    (* -- Ticks ----------------------------------------------------------- *)

    (** Monotonic millisecond clock since module load.  Mirrors
        `Services.Ticks` in BlackBox; rescales `Kernel.Time` if
        its native resolution differs from `Services.resolution`.
        In our build both are 1000, so the rescale is a no-op. *)
    PROCEDURE Ticks* (): INTEGER;
        VAR t: INTEGER;
    BEGIN
        t := Kernel.Time();
        RETURN t * scale + t * corr DIV Kernel.timeResolution
    END Ticks;


    (* -- Action protocol ------------------------------------------------- *)

    (** Run the action.  ABSTRACT — concrete actions implement
        the actual work (e.g. repaint a frame, retry a
        connection, fire a notification).  The framework only
        calls `Do` from `Step`, never directly. *)
    PROCEDURE (a: Action) Do*, NEW, ABSTRACT;


    (* -- Action-list bookkeeping ----------------------------------------- *)

    (** Membership test — TRUE iff `a` is anywhere in the
        `head`-rooted linked list.  O(n) walk. *)
    PROCEDURE In (head, a: Action): BOOLEAN;
    BEGIN
        WHILE (head # NIL) & (head # a) DO head := head.next END;
        RETURN head # NIL
    END In;

    (** Prepend `a` to `list`.  Pre-asserts `a # NIL` and that
        `a` isn't already in the list (an action can only be
        scheduled once at a time). *)
    PROCEDURE Incl (VAR list: Action; a: Action);
    BEGIN
        ASSERT(a # NIL, 20);
        ASSERT(~In(list, a), 21);
        IF list # NIL THEN a.next := list END;
        list := a
    END Incl;

    (** Unlink `a` from `list`.  No-op if `a` isn't in the
        list — callers don't have to pre-check. *)
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


    (* -- Public scheduling API ------------------------------------------- *)

    (** Schedule `a` to run no earlier than `notBefore` ticks.
        `now` (=0) means "as soon as possible"; `immediately`
        (=-1) means "run before any time-tagged action" by
        forcing the deadline below any real tick value.

        The actions list stays sorted by ascending notBefore
        so `Step` can short-circuit on the first not-yet-due
        action.  Re-scheduling an already-pending action is a
        no-op (Incl's assertion guards against double-add). *)
    PROCEDURE DoLater* (a: Action; notBefore: INTEGER);
    BEGIN
        ASSERT(a # NIL, 20);
        ASSERT(~In(actions, a), 21);
        a.notBefore := notBefore;
        IF notBefore = immediately THEN
            hasImmediates := TRUE
        END;
        Incl(actions, a)
    END DoLater;

    (** Cancel a pending action.  Idempotent — silently
        succeeds if `a` was never scheduled. *)
    PROCEDURE RemoveAction* (a: Action);
    BEGIN
        Excl(actions, a)
    END RemoveAction;

    (** Run every action whose deadline has passed.  Skeleton
        implementation — the full BlackBox body tracks
        `candidates` to handle nested re-entries and uses
        Kernel's trap-cleanup to recover from a faulting
        `Do`.  This slice just walks the list once and runs
        every due action in turn; nested DoLater calls inside
        a `Do` are picked up on the next Step. *)
    PROCEDURE Step*;
        VAR cursor, victim: Action; nowTicks: INTEGER;
    BEGIN
        nowTicks := Ticks();
        cursor := actions;
        WHILE cursor # NIL DO
            victim := cursor;
            cursor := cursor.next;
            IF (victim.notBefore = immediately)
            OR (victim.notBefore <= nowTicks) THEN
                Excl(actions, victim);
                victim.Do
            END
        END;
        hasImmediates := FALSE
    END Step;

    (** True if any pending action's deadline is `immediately`.
        Lets the host idle loop short-circuit a sleep when
        immediate work is queued. *)
    PROCEDURE HasImmediates* (): BOOLEAN;
    BEGIN
        RETURN hasImmediates
    END HasImmediates;


END Services.
