MODULE SequencersProbe;
(* Verify the abstract-Sequencers contract round-trips end-to-end:
   a concrete `Directory` produces a concrete `Sequencer` via `dir.New()`,
   notifiers chain through `InstallNotifier`, and `Sequencer.Notify`
   broadcasts in registration order to every notifier's overridden
   `Notify` method.

   Stubs every ABSTRACT method on `Sequencer` with a benign default
   so the concrete type is instantiable; only `Notify` (the base
   broadcast helper) and the notifier dispatch path are actually
   exercised here. *)

    IMPORT Sequencers, Stores;

    TYPE
        TestSequencerDesc* = RECORD (Sequencers.SequencerDesc) END;
        TestSequencer*     = POINTER TO TestSequencerDesc;

        TestDirectoryDesc* = RECORD (Sequencers.DirectoryDesc) END;
        TestDirectory*     = POINTER TO TestDirectoryDesc;

        TestNotifierDesc* = RECORD (Sequencers.NotifierDesc)
            id*: INTEGER
        END;
        TestNotifier*     = POINTER TO TestNotifierDesc;

        PingMsg* = RECORD (Sequencers.Message)
            tag*: INTEGER
        END;

    VAR
        notifyTrace-: ARRAY 16 OF INTEGER;     (* one slot per notifier fired *)
        notifyCount-: INTEGER;                  (* next free slot in trace *)
        lastTag-:     INTEGER;                  (* last broadcast PingMsg.tag seen *)


    (* -- Concrete Sequencer: stub every abstract method ------------------- *)

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


    (* -- Concrete Directory: hand back a fresh TestSequencer -------------- *)

    PROCEDURE (d: TestDirectoryDesc) New* (): Sequencers.Sequencer;
        VAR s: TestSequencer;
    BEGIN
        NEW(s);
        RETURN s
    END New;


    (* -- Concrete Notifier: record the message tag in `notifyTrace` ------- *)

    PROCEDURE (n: TestNotifierDesc) Notify* (VAR msg: Sequencers.Message);
    BEGIN
        IF msg IS PingMsg THEN
            lastTag := msg(PingMsg).tag
        END;
        IF notifyCount < LEN(notifyTrace) THEN
            notifyTrace[notifyCount] := n.id;
            INC(notifyCount)
        END
    END Notify;


    (* -- Test entry point ------------------------------------------------- *)

    (** Install a TestDirectory, ask it for a fresh Sequencer, hook two
        notifiers (ids 1 and 2), broadcast a `PingMsg` with tag = 99,
        and return the trace packed as:
            (notifyCount * 1_000_000) + (lastTag * 1_000)
              + (notifyTrace[0] * 10) + notifyTrace[1]
        `InstallNotifier` pushes onto the head of the chain, so the
        firing order is the reverse of registration: trace = [2, 1].
        Expect 2_099_021 — 2 fires, lastTag = 99, trace = [2, 1]. *)
    PROCEDURE Run*(): INTEGER;
        VAR
            d:   TestDirectory;
            s:   Sequencers.Sequencer;
            n1:  TestNotifier;
            n2:  TestNotifier;
            msg: PingMsg;
    BEGIN
        notifyCount := 0;
        lastTag     := 0;

        NEW(d);
        Sequencers.SetDir(d);

        s := Sequencers.dir.New();

        NEW(n1); n1.id := 1;
        NEW(n2); n2.id := 2;
        s.InstallNotifier(n1);
        s.InstallNotifier(n2);

        msg.tag := 99;
        s.Notify(msg);

        RETURN (notifyCount * 1000000) + (lastTag * 1000)
             + (notifyTrace[0] * 10) + notifyTrace[1]
    END Run;

BEGIN
    notifyCount := 0;
    lastTag     := 0
END SequencersProbe.
