MODULE ModelsDispatchProbe;
(* Verify the Models dispatch procedures (`Do`, `BeginScript`,
   `LastOp`, `Bunch`, etc.) actually forward to a typed Sequencer
   when one is installed.  The probe wires up:

   - A counting TestSequencer that tracks each Sequencer-method call.
   - A test Operation whose `Do()` method bumps a separate counter,
     so we can also verify the no-sequencer fallback path.
   - A TinyModel; we install the sequencer, exercise every dispatch
     proc, then read the counters back. *)

    IMPORT Models, Sequencers, Stores;

    TYPE
        TinyModelDesc* = RECORD (Models.ModelDesc) END;
        TinyModel*     = POINTER TO TinyModelDesc;

        TestSequencerDesc* = RECORD (Sequencers.SequencerDesc) END;
        TestSequencer*     = POINTER TO TestSequencerDesc;

        TestOpDesc* = RECORD (Stores.OperationDesc) END;
        TestOp*     = POINTER TO TestOpDesc;

    VAR
        doCount-:          INTEGER;
        beginScriptCount-: INTEGER;
        endScriptCount-:   INTEGER;
        bunchCount-:       INTEGER;
        opDoCount-:        INTEGER;


    (* -- TestOp: a real Operation override --------------------------------- *)

    PROCEDURE (op: TestOpDesc) Do*;
    BEGIN
        INC(opDoCount)
    END Do;


    (* -- TestSequencer: count dispatched calls ----------------------------- *)

    PROCEDURE (s: TestSequencerDesc) Dirty* (): BOOLEAN;
    BEGIN RETURN FALSE END Dirty;

    PROCEDURE (s: TestSequencerDesc) SetDirty* (dirty: BOOLEAN);
    BEGIN END SetDirty;

    PROCEDURE (s: TestSequencerDesc) BeginScript*
        (IN name: Stores.OpName; VAR script: Stores.Operation);
    BEGIN
        INC(beginScriptCount);
        script := NIL
    END BeginScript;

    PROCEDURE (s: TestSequencerDesc) Do*
        (st: Stores.Store; IN name: Stores.OpName; op: Stores.Operation);
    BEGIN
        INC(doCount)
    END Do;

    PROCEDURE (s: TestSequencerDesc) LastOp*
        (st: Stores.Store): Stores.Operation;
    BEGIN RETURN NIL END LastOp;

    PROCEDURE (s: TestSequencerDesc) Bunch* (st: Stores.Store);
    BEGIN
        INC(bunchCount)
    END Bunch;

    PROCEDURE (s: TestSequencerDesc) EndScript* (script: Stores.Operation);
    BEGIN
        INC(endScriptCount)
    END EndScript;

    PROCEDURE (s: TestSequencerDesc) StopBunching* ();
    BEGIN END StopBunching;

    PROCEDURE (s: TestSequencerDesc) BeginModification*
        (type: INTEGER; st: Stores.Store);
    BEGIN END BeginModification;

    PROCEDURE (s: TestSequencerDesc) EndModification*
        (type: INTEGER; st: Stores.Store);
    BEGIN END EndModification;

    PROCEDURE (s: TestSequencerDesc) CanUndo* (): BOOLEAN;
    BEGIN RETURN FALSE END CanUndo;

    PROCEDURE (s: TestSequencerDesc) CanRedo* (): BOOLEAN;
    BEGIN RETURN FALSE END CanRedo;

    PROCEDURE (s: TestSequencerDesc) GetUndoName* (VAR name: Stores.OpName);
    BEGIN name[0] := 0X END GetUndoName;

    PROCEDURE (s: TestSequencerDesc) GetRedoName* (VAR name: Stores.OpName);
    BEGIN name[0] := 0X END GetRedoName;

    PROCEDURE (s: TestSequencerDesc) Undo* ();
    BEGIN END Undo;

    PROCEDURE (s: TestSequencerDesc) Redo* ();
    BEGIN END Redo;


    (** Allocate a TinyModel + TestSequencer + TestOp.  Run a
        cross-section of Models's dispatch procs:
            Do(m, "edit", op)   -> sequencer.Do            (doCount = 1)
            BeginScript(...)    -> sequencer.BeginScript   (beginScriptCount = 1)
            EndScript(...)      -> sequencer.EndScript     (endScriptCount = 1)
            Bunch(m)            -> sequencer.Bunch         (bunchCount = 1)
        Then detach the sequencer (SetSequencer NIL) and call
        Do(m, "fallback", op) — the WITH ELSE branch should fire,
        invoking op.Do() directly (opDoCount = 1).

        Returns a packed verifier:
            doCount * 100000 + beginScriptCount * 10000
              + endScriptCount * 1000 + bunchCount * 100 + opDoCount
        Expect 111101 = 1×100k + 1×10k + 1×1k + 1×100 + 1. *)
    PROCEDURE Run* (): INTEGER;
        VAR
            m:    TinyModel;
            s:    TestSequencer;
            op:   TestOp;
            opOut: Stores.Operation;
            name: Stores.OpName;
    BEGIN
        doCount          := 0;
        beginScriptCount := 0;
        endScriptCount   := 0;
        bunchCount       := 0;
        opDoCount        := 0;

        NEW(m);
        NEW(s);
        NEW(op);
        Models.SetSequencer(m, s);

        name[0] := 65X; name[1] := 0X;       (* "A" *)
        Models.Do(m, name, op);
        Models.BeginScript(m, name, opOut);
        Models.EndScript(m, opOut);
        Models.Bunch(m);

        (* Detach sequencer, exercise fallback to op.Do(). *)
        Models.SetSequencer(m, NIL);
        Models.Do(m, name, op);

        RETURN (doCount * 100000)
             + (beginScriptCount * 10000)
             + (endScriptCount * 1000)
             + (bunchCount * 100)
             + opDoCount
    END Run;

BEGIN
    doCount          := 0;
    beginScriptCount := 0;
    endScriptCount   := 0;
    bunchCount       := 0;
    opDoCount        := 0
END ModelsDispatchProbe.
