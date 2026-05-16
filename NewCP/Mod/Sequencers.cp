MODULE Sequencers;
(*
   NewCP `Sequencers` port.

   Declares the abstract contract (`Sequencer`, `Notifier`, `Message`,
   `CloseMsg`, `RemoveMsg`, `Directory`) plus the concrete `StdSequencer`
   that implements the undo/redo state machine (Track 12 A12).

   `StdSequencer` maintains separate undo and redo stacks (depth
   `maxStack = 64`) and supports script grouping via `CompoundOp` (depth
   `maxScript = 256`).  `Stores.Operation.Do` is a toggle: calling it the
   first time applies the operation; calling it again reverts it.
   CompoundOp tracks which direction is next via its `done` flag.

   Notifier list ownership is intentionally left non-exported: only
   `InstallNotifier` and `Notify` (the base method) walk the chain.
*)

    IMPORT Stores;

    CONST
        clean*       = 0;       (** modification class: harmless edits — keep undo intact *)
        notUndoable* = 1;       (** modification class: drops the undo stack *)
        invisible*   = 2;       (** modification class: ignored entirely *)

        maxStack  = 64;         (* max undo/redo depth *)
        maxScript = 256;        (* max ops per script *)

    TYPE
        (** Abstract base for every Sequencer message.  Subclasses
            (CloseMsg, RemoveMsg, plus subsystem-specific extensions)
            extend this via `RECORD (Message)`. *)
        Message* = ABSTRACT RECORD END;

        (** Notifier — a chained subscriber to a Sequencer's broadcast
            stream.  Concrete subclasses override `Notify` (default
            EMPTY) to react to dispatched messages.  `next` is private
            chain bookkeeping; clients must not read or write it. *)
        NotifierDesc* = ABSTRACT RECORD
            next: Notifier
        END;
        Notifier* = POINTER TO NotifierDesc;

        (** Sequencer — abstract per-Domain undo/redo + notifier hub.
            Concrete impls (StdInterpreter et al.) override every
            ABSTRACT method below; the two non-abstract helpers
            (`Notify`, `InstallNotifier`) and the EMPTY `Handle`
            ride along on the base. *)
        SequencerDesc* = ABSTRACT RECORD
            notifiers: Notifier
        END;
        Sequencer* = POINTER TO SequencerDesc;

        (** Sent when a document is about to close.  Handlers may set
            `sticky := TRUE` to veto the close (e.g. unsaved changes
            with the user choosing Cancel). *)
        CloseMsg* = RECORD (Message)
            sticky*: BOOLEAN    (** OUT, preset to FALSE *)
        END;

        (** Sent when a Sequencer is being torn down — typically just
            before the owning Domain dies.  Carries no payload; pure
            lifecycle ping. *)
        RemoveMsg* = RECORD (Message) END;

        (** Concrete-Sequencer factory.  The runtime registers exactly
            one `Directory` instance (typically at framework init) via
            `SetDir`, and `dir.New()` is then how every other module
            obtains a fresh Sequencer. *)
        DirectoryDesc* = ABSTRACT RECORD END;
        Directory*     = POINTER TO DirectoryDesc;

        OpEntry = RECORD
            op:   Stores.Operation;
            name: Stores.OpName
        END;

        CompoundOpDesc = RECORD (Stores.OperationDesc)
            ops:  ARRAY maxScript OF OpEntry;
            len:  INTEGER;
            done: BOOLEAN
        END;
        CompoundOp = POINTER TO CompoundOpDesc;

        StdSequencerDesc* = RECORD (SequencerDesc)
            undo:       ARRAY maxStack OF OpEntry;
            ulen:       INTEGER;
            redo:       ARRAY maxStack OF OpEntry;
            rlen:       INTEGER;
            dirty:      BOOLEAN;
            bunching:   BOOLEAN;
            scriptDepth: INTEGER;
            scriptName: Stores.OpName;
            script:     CompoundOp;
            modDepth:   INTEGER
        END;
        StdSequencer* = POINTER TO StdSequencerDesc;

        StdDirectoryDesc = RECORD (DirectoryDesc) END;
        StdDirectory     = POINTER TO StdDirectoryDesc;

    VAR
        dir-:    Directory;     (** the installed factory; NIL until SetDir runs *)
        stdDir:  StdDirectory;


    (* -- Directory --------------------------------------------------------- *)

    PROCEDURE (d: Directory) New* (): Sequencer, NEW, ABSTRACT;

    PROCEDURE SetDir* (d: Directory);
    BEGIN
        ASSERT(d # NIL, 20);
        dir := d
    END SetDir;


    (* -- Notifier ---------------------------------------------------------- *)

    PROCEDURE (f: Notifier) Notify* (VAR msg: Message), NEW, EMPTY;


    (* -- Sequencer abstract methods --------------------------------------- *)

    PROCEDURE (s: Sequencer) Dirty* (): BOOLEAN, NEW, ABSTRACT;
    PROCEDURE (s: Sequencer) SetDirty* (dirty: BOOLEAN), NEW, ABSTRACT;

    PROCEDURE (s: Sequencer) BeginScript*
        (IN name: Stores.OpName; VAR script: Stores.Operation), NEW, ABSTRACT;

    PROCEDURE (s: Sequencer) Do*
        (st: Stores.Store; IN name: Stores.OpName; op: Stores.Operation),
        NEW, ABSTRACT;

    PROCEDURE (s: Sequencer) LastOp*
        (st: Stores.Store): Stores.Operation, NEW, ABSTRACT;

    PROCEDURE (s: Sequencer) Bunch* (st: Stores.Store), NEW, ABSTRACT;

    PROCEDURE (s: Sequencer) EndScript* (script: Stores.Operation), NEW, ABSTRACT;

    PROCEDURE (s: Sequencer) StopBunching* (), NEW, ABSTRACT;

    PROCEDURE (s: Sequencer) BeginModification*
        (type: INTEGER; st: Stores.Store), NEW, ABSTRACT;

    PROCEDURE (s: Sequencer) EndModification*
        (type: INTEGER; st: Stores.Store), NEW, ABSTRACT;

    PROCEDURE (s: Sequencer) CanUndo* (): BOOLEAN, NEW, ABSTRACT;
    PROCEDURE (s: Sequencer) CanRedo* (): BOOLEAN, NEW, ABSTRACT;

    PROCEDURE (s: Sequencer) GetUndoName* (VAR name: Stores.OpName), NEW, ABSTRACT;
    PROCEDURE (s: Sequencer) GetRedoName* (VAR name: Stores.OpName), NEW, ABSTRACT;

    PROCEDURE (s: Sequencer) Undo* (), NEW, ABSTRACT;
    PROCEDURE (s: Sequencer) Redo* (), NEW, ABSTRACT;


    (* -- CompoundOp ------------------------------------------------------- *)

    PROCEDURE (c: CompoundOp) Do*;
        VAR i: INTEGER;
    BEGIN
        IF ~c.done THEN
            i := 0;
            WHILE i < c.len DO
                c.ops[i].op.Do();
                INC(i)
            END
        ELSE
            i := c.len - 1;
            WHILE i >= 0 DO
                c.ops[i].op.Do();
                DEC(i)
            END
        END;
        c.done := ~c.done
    END Do;


    (* -- StdDirectory ----------------------------------------------------- *)

    PROCEDURE (d: StdDirectory) New* (): Sequencer;
        VAR s: StdSequencer;
    BEGIN
        NEW(s);
        s.ulen        := 0;
        s.rlen        := 0;
        s.dirty       := FALSE;
        s.bunching    := FALSE;
        s.scriptDepth := 0;
        s.script      := NIL;
        s.modDepth    := 0;
        RETURN s
    END New;


    (* -- StdSequencer ----------------------------------------------------- *)

    PROCEDURE (s: StdSequencer) Dirty* (): BOOLEAN;
    BEGIN
        RETURN s.dirty
    END Dirty;

    PROCEDURE (s: StdSequencer) SetDirty* (dirty: BOOLEAN);
    BEGIN
        s.dirty := dirty
    END SetDirty;

    PROCEDURE (s: StdSequencer) CanUndo* (): BOOLEAN;
    BEGIN
        RETURN s.ulen > 0
    END CanUndo;

    PROCEDURE (s: StdSequencer) CanRedo* (): BOOLEAN;
    BEGIN
        RETURN s.rlen > 0
    END CanRedo;

    PROCEDURE (s: StdSequencer) GetUndoName* (VAR name: Stores.OpName);
    BEGIN
        IF s.ulen > 0 THEN name := s.undo[s.ulen - 1].name
        ELSE name := ""
        END
    END GetUndoName;

    PROCEDURE (s: StdSequencer) GetRedoName* (VAR name: Stores.OpName);
    BEGIN
        IF s.rlen > 0 THEN name := s.redo[s.rlen - 1].name
        ELSE name := ""
        END
    END GetRedoName;

    PROCEDURE (s: StdSequencer) LastOp* (st: Stores.Store): Stores.Operation;
    BEGIN
        IF s.ulen > 0 THEN RETURN s.undo[s.ulen - 1].op
        ELSE RETURN NIL
        END
    END LastOp;

    PROCEDURE (s: StdSequencer) Do*
            (st: Stores.Store; IN name: Stores.OpName; op: Stores.Operation);
    BEGIN
        op.Do();
        IF s.scriptDepth > 0 THEN
            (* buffer into the current compound script *)
            IF s.script.len < maxScript THEN
                s.script.ops[s.script.len].op   := op;
                s.script.ops[s.script.len].name := name;
                INC(s.script.len)
            END
        ELSIF s.bunching & (s.ulen > 0) THEN
            (* merge with the top undo entry — they undo together *)
            s.undo[s.ulen - 1].op   := op;
            s.undo[s.ulen - 1].name := name;
            s.bunching := FALSE;
            s.rlen     := 0;
            s.dirty    := TRUE
        ELSE
            IF s.ulen < maxStack THEN
                s.undo[s.ulen].op   := op;
                s.undo[s.ulen].name := name;
                INC(s.ulen)
            END;
            s.rlen  := 0;
            s.dirty := TRUE
        END
    END Do;

    PROCEDURE (s: StdSequencer) Bunch* (st: Stores.Store);
    BEGIN
        s.bunching := TRUE
    END Bunch;

    PROCEDURE (s: StdSequencer) StopBunching* ();
    BEGIN
        s.bunching := FALSE
    END StopBunching;

    PROCEDURE (s: StdSequencer) BeginScript*
            (IN name: Stores.OpName; VAR script: Stores.Operation);
        VAR c: CompoundOp;
    BEGIN
        INC(s.scriptDepth);
        IF s.scriptDepth = 1 THEN
            NEW(c);
            c.len  := 0;
            c.done := FALSE;
            s.script     := c;
            s.scriptName := name
        END;
        script := s.script
    END BeginScript;

    PROCEDURE (s: StdSequencer) EndScript* (script: Stores.Operation);
        VAR name: Stores.OpName;
    BEGIN
        IF s.scriptDepth > 0 THEN
            DEC(s.scriptDepth);
            IF s.scriptDepth = 0 THEN
                IF s.script.len > 0 THEN
                    (* sub-ops were Do()'d as buffered, so compound is already
                       in applied state — mark done so first Undo call reverses *)
                    s.script.done := TRUE;
                    name := s.scriptName;
                    IF s.ulen < maxStack THEN
                        s.undo[s.ulen].op   := s.script;
                        s.undo[s.ulen].name := name;
                        INC(s.ulen)
                    END;
                    s.rlen  := 0;
                    s.dirty := TRUE
                END;
                s.script := NIL
            END
        END
    END EndScript;

    PROCEDURE (s: StdSequencer) Undo* ();
        VAR e: OpEntry;
    BEGIN
        IF s.ulen > 0 THEN
            DEC(s.ulen);
            e := s.undo[s.ulen];
            e.op.Do();
            IF s.rlen < maxStack THEN
                s.redo[s.rlen] := e;
                INC(s.rlen)
            END
        END
    END Undo;

    PROCEDURE (s: StdSequencer) Redo* ();
        VAR e: OpEntry;
    BEGIN
        IF s.rlen > 0 THEN
            DEC(s.rlen);
            e := s.redo[s.rlen];
            e.op.Do();
            IF s.ulen < maxStack THEN
                s.undo[s.ulen] := e;
                INC(s.ulen)
            END;
            s.dirty := TRUE
        END
    END Redo;

    PROCEDURE (s: StdSequencer) BeginModification*
            (type: INTEGER; st: Stores.Store);
    BEGIN
        INC(s.modDepth);
        IF type = notUndoable THEN
            s.ulen := 0
        END
    END BeginModification;

    PROCEDURE (s: StdSequencer) EndModification*
            (type: INTEGER; st: Stores.Store);
    BEGIN
        IF s.modDepth > 0 THEN
            DEC(s.modDepth)
        END
    END EndModification;


    (* -- Sequencer concrete helpers --------------------------------------- *)

    (** Optional generic message handler.  Default EMPTY; subclasses
        override to hook their own message types (paint, focus, etc.)
        without polluting the abstract list above. *)
    PROCEDURE (s: Sequencer) Handle* (VAR msg: ANYREC), NEW, EMPTY;

    (** Broadcast `msg` to every installed notifier in registration
        order.  The base impl just walks the chain; subclasses rarely
        need to override. *)
    PROCEDURE (s: Sequencer) Notify* (VAR msg: Message), NEW;
        VAR n: Notifier;
    BEGIN
        n := s.notifiers;
        WHILE n # NIL DO
            n.Notify(msg);
            n := n.next
        END
    END Notify;

    (** Push a notifier onto the head of the subscriber chain.  No
        deduplication: registering the same notifier twice causes its
        `Notify` to be called twice per broadcast. *)
    PROCEDURE (s: Sequencer) InstallNotifier* (n: Notifier), NEW;
    BEGIN
        n.next := s.notifiers;
        s.notifiers := n
    END InstallNotifier;


BEGIN
    NEW(stdDir); dir := stdDir
END Sequencers.
