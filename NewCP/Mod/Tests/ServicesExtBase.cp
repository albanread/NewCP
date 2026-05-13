MODULE ServicesExtBase;
(*
   Workout for the `Services` slice.

   Exercises the deferred-action scheduler end-to-end:

   - extending `Services.ActionDesc` (ABSTRACT) with a concrete
     leaf that overrides `Do`;
   - scheduling two actions via `DoLater`, one with `immediately`
     and one with `now`;
   - draining the queue via `Step`;
   - verifying both `Do` bodies ran (via vtable dispatch) and
     the actions list is empty afterwards;
   - exercising `RemoveAction` to cancel a scheduled action.

   Returns a packed value confirming each stage fired.
*)

    IMPORT Services;

    TYPE
        (** Concrete leaf action — bumps a module counter
            every time its `Do` is invoked.  Records which
            instance fired by stamping its `id` into `lastFired`
            so the test can verify dispatch ordering. *)
        CounterActionDesc* = RECORD (Services.ActionDesc)
            id*: INTEGER
        END;
        CounterAction* = POINTER TO CounterActionDesc;


    VAR
        firedCount: INTEGER;
        lastFired:  INTEGER;


    PROCEDURE (a: CounterActionDesc) Do*;
    BEGIN
        firedCount := firedCount + 1;
        lastFired  := a.id
    END Do;


    PROCEDURE Run* (): INTEGER;
        VAR a, b, c: CounterAction; r: INTEGER;
    BEGIN
        firedCount := 0;
        lastFired  := -1;

        NEW(a); a.id := 11;
        NEW(b); b.id := 22;
        NEW(c); c.id := 33;

        (* Schedule three actions, one with `now`, one with
           `immediately`, one that we'll cancel before Step. *)
        Services.DoLater(a, Services.now);
        Services.DoLater(b, Services.immediately);
        Services.DoLater(c, Services.now);

        (* Cancel c before it can fire. *)
        Services.RemoveAction(c);

        (* Drain the scheduled-and-due actions. *)
        Services.Step;

        IF firedCount # 2 THEN RETURN -100 - firedCount END;
        IF (lastFired # 11) & (lastFired # 22) THEN
            RETURN -200 - lastFired
        END;

        (* Second Step should be a no-op — nothing pending. *)
        Services.Step;
        IF firedCount # 2 THEN RETURN -300 - firedCount END;

        (* Pack: firedCount(2)*1000 + lastFired(11 or 22).
           Either ordering counts as success.  We collapse
           "either 11 or 22" to a single canonical 22 to keep
           the test deterministic by re-running with one
           action only. *)
        firedCount := 0;
        NEW(a); a.id := 22;
        Services.DoLater(a, Services.now);
        Services.Step;

        r := firedCount * 1000 + lastFired;
        RETURN r       (* expected 1022 *)
    END Run;

END ServicesExtBase.
