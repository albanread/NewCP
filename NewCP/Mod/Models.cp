MODULE Models;
(*
   NewCP `Models` port — abstract document-model surface.

   Direct port of BlackBox `System/Mod/Models.odc` with one shape change:
   the Model carries an opaque sequencer field directly (ANYPTR `seq`)
   instead of fetching it via `m.Domain().GetSequencer()`.  The reason
   is that NewCP's Stores port is still in handle-facade form — there's
   no `Stores.Store` record yet and therefore no `Domain()` method on
   Model.  Once Stores ports its OO surface we'll restore the BlackBox
   shape and have every dispatch procedure call
   `m.Domain().GetSequencer()` instead of reading `m.seq` directly.
   Behaviourally the two are equivalent when the model holds the
   sequencer the framework would have produced.

   The dispatch pattern follows BlackBox: pull the sequencer into a
   local `ANYPTR`, narrow it with `WITH s: Sequencers.Sequencer DO …`
   to get a typed receiver, and forward the message.  The narrow uses
   the runtime IS test (`__newcp_type_test`) the WITH backfill ships
   on top of.  Stack-allocated record subjects work via the shadow-
   header RTTI mechanism; cross-module bases get patched at module
   `__init_types` time.

   Divergences still in place:

   - `Internalize` / `Externalize` overrides on `Model` are skipped —
     they need Stores.Reader / Writer as records.  Super calls already
     work (see SuperProbe).
   - `CopyOf` is an identity stub.  BlackBox dispatches through
     `Stores.CopyOf(m)(Model)` which clones the entire store tree;
     we need Domain semantics for that.
   - The Sequencer's `Do` / `BeginScript` / `LastOp` / etc. take a
     `Stores.Store` — currently NIL since concrete Store instances
     aren't allocated yet (the OO surface is empty).  Hook the proper
     receiver in once Stores ports its data fields and runtime
     allocator.
   - `m.seq` is held directly on `ModelDesc` rather than fetched via
     `m.Domain().GetSequencer()` — the latter needs a populated
     `Stores.Domain` with a registered sequencer.
*)

    IMPORT Kernel, Sequencers, Stores;

    CONST
        minVersion = 0; maxVersion = 0;

        clean*       = Sequencers.clean;
        notUndoable* = Sequencers.notUndoable;
        invisible*   = Sequencers.invisible;

    TYPE
        (** Abstract document model.  Extends `Stores.Store` so every
            Model is also persistable (Internalize / Externalize will
            land via super calls once Reader / Writer become records).
            `era` is incremented on every `Broadcast`; `guard` traps
            reentrant broadcasts; `seq` is the optional sequencer this
            model dispatches messages through (ANYPTR so we don't drag
            every model concrete-type into a Sequencers dependency). *)
        ModelDesc* = ABSTRACT RECORD (Stores.StoreDesc)
            era-:   INTEGER;
            guard-: INTEGER;
            seq-:   ANYPTR
        END;
        Model* = POINTER TO ModelDesc;

        ContextDesc* = ABSTRACT RECORD END;
        Context*     = POINTER TO ContextDesc;

        Proposal* = ABSTRACT RECORD END;

        Message* = ABSTRACT RECORD
            model-: Model;
            era-:   INTEGER
        END;

        NeutralizeMsg* = RECORD (Message) END;

        UpdateMsg* = EXTENSIBLE RECORD (Message) END;

    VAR
        domainGuard: INTEGER;       (* = Kernel.TrapCount() + 1 if a
                                       Domaincast is in flight *)


    (* -- Model methods (extending Stores.StoreDesc) ----------------------- *)

    (** EXTENSIBLE Internalize chain.  Concrete model subclasses
        (`TextModels.StdModelDesc`, etc.) override this to read their
        own fields, calling `m.Internalize^(rd)` first to chain into
        the inherited behaviour.  BlackBox reads a Model version stamp
        at this layer; we'll add that once `Reader.ReadVersion` lands. *)
    PROCEDURE (m: Model) Internalize* (VAR rd: Stores.Reader), EXTENSIBLE;
    BEGIN
        m.Internalize^(rd)
    END Internalize;

    (** Symmetric to `Internalize`. *)
    PROCEDURE (m: Model) Externalize* (VAR wr: Stores.Writer), EXTENSIBLE;
    BEGIN
        m.Externalize^(wr)
    END Externalize;

    (** Models don't carry a Domain link yet — return NIL for now.
        BlackBox reads `m.dlink` here; we'll restore that once Stores
        grows the domain bookkeeping. *)
    PROCEDURE (m: Model) Domain* (): Stores.Domain;
    BEGIN
        RETURN NIL
    END Domain;


    (* -- Context abstract methods ----------------------------------------- *)

    PROCEDURE (c: Context) ThisModel* (): Model, NEW, ABSTRACT;
    PROCEDURE (c: Context) Normalize* (): BOOLEAN, NEW, ABSTRACT;
    PROCEDURE (c: Context) GetSize*   (OUT w, h: INTEGER), NEW, ABSTRACT;

    PROCEDURE (c: Context) SetSize*    (w, h: INTEGER), NEW, EMPTY;
    PROCEDURE (c: Context) MakeVisible* (l, t, r, b: INTEGER), NEW, EMPTY;
    PROCEDURE (c: Context) Consider*   (VAR p: Proposal), NEW, EMPTY;


    (* -- Miscellaneous ---------------------------------------------------- *)

    PROCEDURE Era* (m: Model): INTEGER;
    BEGIN
        ASSERT(m # NIL, 20);
        RETURN m.era
    END Era;

    (** Identity stub — see Divergences above. *)
    PROCEDURE CopyOf* (m: Model): Model;
    BEGIN
        ASSERT(m # NIL, 20);
        RETURN m
    END CopyOf;

    (** Install / replace the sequencer this model dispatches through.
        Pass NIL to detach.  Bypass for the BlackBox `Domain.SetSequencer`
        path that we don't have a Stores.Domain for. *)
    PROCEDURE SetSequencer* (m: Model; s: Sequencers.Sequencer);
    BEGIN
        ASSERT(m # NIL, 20);
        m.seq := s
    END SetSequencer;


    (* -- Sequencer-driven dispatch --------------------------------------- *)
    (* All of these chase the same shape: pull `m.seq` into an ANYPTR,
       narrow with `WITH s: Sequencers.Sequencer DO ... ELSE ... END`,
       and forward.  The Stores.Store handle we pass to the Sequencer
       is currently 0 (placeholder); switch to `m` once Models extends
       Stores.Store. *)

    PROCEDURE BeginScript* (m: Model;
                            IN name: Stores.OpName;
                            VAR script: Stores.Operation);
        VAR s: ANYPTR;
    BEGIN
        ASSERT(m # NIL, 20);
        s := m.seq;
        IF s # NIL THEN
            WITH s: Sequencers.Sequencer DO
                s.BeginScript(name, script)
            ELSE
                script := NIL
            END
        ELSE
            script := NIL
        END
    END BeginScript;

    PROCEDURE Do* (m: Model; IN name: Stores.OpName; op: Stores.Operation);
        VAR s: ANYPTR;
    BEGIN
        ASSERT(m # NIL, 20);
        ASSERT(op # NIL, 21);
        s := m.seq;
        IF s # NIL THEN
            WITH s: Sequencers.Sequencer DO
                s.Do(NIL, name, op)
            ELSE
                op.Do()
            END
        ELSE
            op.Do()
        END
    END Do;

    PROCEDURE LastOp* (m: Model): Stores.Operation;
        VAR s: ANYPTR;
    BEGIN
        ASSERT(m # NIL, 20);
        s := m.seq;
        IF s # NIL THEN
            WITH s: Sequencers.Sequencer DO
                RETURN s.LastOp(NIL)
            ELSE
                RETURN NIL
            END
        ELSE
            RETURN NIL
        END
    END LastOp;

    PROCEDURE Bunch* (m: Model);
        VAR s: ANYPTR;
    BEGIN
        ASSERT(m # NIL, 20);
        s := m.seq;
        IF s # NIL THEN
            WITH s: Sequencers.Sequencer DO
                s.Bunch(NIL)
            ELSE
            END
        END
    END Bunch;

    PROCEDURE StopBunching* (m: Model);
        VAR s: ANYPTR;
    BEGIN
        ASSERT(m # NIL, 20);
        s := m.seq;
        IF s # NIL THEN
            WITH s: Sequencers.Sequencer DO
                s.StopBunching()
            ELSE
            END
        END
    END StopBunching;

    PROCEDURE EndScript* (m: Model; script: Stores.Operation);
        VAR s: ANYPTR;
    BEGIN
        ASSERT(m # NIL, 20);
        s := m.seq;
        IF s # NIL THEN
            WITH s: Sequencers.Sequencer DO
                s.EndScript(script)
            ELSE
            END
        END
    END EndScript;

    PROCEDURE BeginModification* (type: INTEGER; m: Model);
        VAR s: ANYPTR;
    BEGIN
        ASSERT(m # NIL, 20);
        s := m.seq;
        IF s # NIL THEN
            WITH s: Sequencers.Sequencer DO
                s.BeginModification(type, NIL)
            ELSE
            END
        END
    END BeginModification;

    PROCEDURE EndModification* (type: INTEGER; m: Model);
        VAR s: ANYPTR;
    BEGIN
        ASSERT(m # NIL, 20);
        s := m.seq;
        IF s # NIL THEN
            WITH s: Sequencers.Sequencer DO
                s.EndModification(type, NIL)
            ELSE
            END
        END
    END EndModification;

    PROCEDURE SetDirty* (m: Model);
        VAR s: ANYPTR;
    BEGIN
        ASSERT(m # NIL, 20);
        s := m.seq;
        IF s # NIL THEN
            WITH s: Sequencers.Sequencer DO
                s.SetDirty(TRUE)
            ELSE
            END
        END
    END SetDirty;


    (** Domain-wide message broadcast.  We don't have `Stores.Domain`
        with its own sequencer hookup yet, so this is a no-op in the
        slim port — the `domainGuard` reentry check stays in place
        and is exercised once concrete domains land. *)
    PROCEDURE Domaincast* (VAR msg: Message);
        VAR g: INTEGER;
    BEGIN
        msg.model := NIL;
        msg.era   := -1;
        g := Kernel.TrapCount() + 1;
        IF domainGuard > 0 THEN ASSERT(domainGuard # g, 20) END;
        domainGuard := g;
        (* No real domain dispatch yet — once Stores ports its OO
           surface, fetch the domain's sequencer and call its Handle. *)
        domainGuard := 0
    END Domaincast;

    (** Bump `m.era`, stamp the message envelope, and forward to the
        installed Sequencer's `Handle` if present.  WITH narrows the
        ANYPTR `seq` field to the typed Sequencer at runtime.  The
        per-model `guard` field implements BlackBox's reentry trap
        for nested broadcasts on the same model. *)
    PROCEDURE Broadcast* (m: Model; VAR msg: Message);
        VAR s: ANYPTR; g: INTEGER;
    BEGIN
        ASSERT(m # NIL, 20);
        msg.model := m;
        s := m.seq;
        IF s # NIL THEN
            WITH s: Sequencers.Sequencer DO
                INC(m.era);
                msg.era := m.era;
                g := Kernel.TrapCount() + 1;
                IF m.guard > 0 THEN ASSERT(m.guard # g, 21) END;
                m.guard := g;
                s.Handle(msg);
                m.guard := 0
            ELSE
            END
        END
    END Broadcast;

BEGIN
    domainGuard := 0
END Models.
