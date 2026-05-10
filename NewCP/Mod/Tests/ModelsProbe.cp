MODULE ModelsProbe;
(* Verify the Models surface end-to-end:

   - A concrete `Model` subclass instantiates without diagnostics.
   - `Models.Era(m)` reads the era field through the read-only export.
   - `Models.Broadcast(m, msg)` increments era and stamps `msg.model`
     and `msg.era`.
   - When a Sequencer is installed via `Models.SetSequencer`,
     `Broadcast` dispatches to the Sequencer's `Handle(VAR msg: ANYREC)`
     method via the WITH narrowing on the model's ANYPTR `seq` field.
   - `Models.NeutralizeMsg` extends `Models.Message` and inherits the
     `model` / `era` envelope fields. *)

    IMPORT Models, Sequencers, Stores;

    TYPE
        TinyModelDesc* = RECORD (Models.ModelDesc) END;
        TinyModel*     = POINTER TO TinyModelDesc;

        (* A barely-concrete Sequencer that records the era of the
           last message its `Handle` saw.  Every abstract method is
           stubbed; `Handle` is the only one Models.Broadcast routes
           through. *)
        TestSequencerDesc* = RECORD (Sequencers.SequencerDesc) END;
        TestSequencer*     = POINTER TO TestSequencerDesc;

    VAR
        handleCount-:  INTEGER;
        handleEra-:    INTEGER;


    (* -- TestSequencer: stub every abstract method, intercept Handle ----- *)

    PROCEDURE (s: TestSequencerDesc) Dirty* (): BOOLEAN;
    BEGIN RETURN FALSE END Dirty;

    PROCEDURE (s: TestSequencerDesc) SetDirty* (dirty: BOOLEAN);
    BEGIN END SetDirty;

    PROCEDURE (s: TestSequencerDesc) BeginScript*
        (IN name: Stores.OpName; VAR script: Stores.Operation);
    BEGIN script := NIL END BeginScript;

    PROCEDURE (s: TestSequencerDesc) Do*
        (st: Stores.Store; IN name: Stores.OpName; op: Stores.Operation);
    BEGIN END Do;

    PROCEDURE (s: TestSequencerDesc) LastOp*
        (st: Stores.Store): Stores.Operation;
    BEGIN RETURN NIL END LastOp;

    PROCEDURE (s: TestSequencerDesc) Bunch* (st: Stores.Store);
    BEGIN END Bunch;

    PROCEDURE (s: TestSequencerDesc) EndScript* (script: Stores.Operation);
    BEGIN END EndScript;

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

    (* The override.  When Models.Broadcast WITHs the model's `seq`
       ANYPTR to Sequencers.Sequencer and calls `s.Handle(msg)`, this
       runs.  Now that stack records carry a shadow-header RTTI tag,
       we can type-guard `msg` from ANYREC back to Models.Message and
       read its envelope. *)
    PROCEDURE (s: TestSequencerDesc) Handle* (VAR msg: ANYREC);
    BEGIN
        WITH msg: Models.Message DO
            handleEra := msg.era
        ELSE
            handleEra := -1
        END;
        INC(handleCount)
    END Handle;


    (** Allocate a TinyModel, install a TestSequencer, broadcast a
        NeutralizeMsg twice, and return a packed verifier:
            (Era(m) * 1_000_000) + (handleEra * 1000) + handleCount
        Each Broadcast bumps era, stamps the message, and dispatches
        to TestSequencer.Handle which reads `msg.era` via WITH.
        After two broadcasts: Era=2, handleEra=2 (last seen),
        handleCount=2.  Expect 2_002_002 = 2×1m + 2×1k + 2. *)
    PROCEDURE Run* (): INTEGER;
        VAR
            m:   TinyModel;
            s:   TestSequencer;
            msg: Models.NeutralizeMsg;
    BEGIN
        handleCount := 0;
        handleEra   := 0;

        NEW(m);
        NEW(s);
        Models.SetSequencer(m, s);

        Models.Broadcast(m, msg);
        Models.Broadcast(m, msg);

        RETURN (Models.Era(m) * 1000000) + (handleEra * 1000) + handleCount
    END Run;

BEGIN
    handleCount := 0;
    handleEra   := 0
END ModelsProbe.
