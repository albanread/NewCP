MODULE Sequencers;
(*
   NewCP `Sequencers` port — abstract surface only.

   BlackBox `System/Mod/Sequencers.odc` defines the contract every
   document's per-Domain undo/redo + notification mechanism implements.
   The module itself is almost entirely abstract: it declares the
   `Sequencer`, `Notifier`, `Message`, `CloseMsg`, `RemoveMsg` and
   `Directory` types, exports a global `dir` and `SetDir` for installing
   a concrete factory, and provides the two book-keeping helpers
   (`Notify` broadcast loop and `InstallNotifier`) that subclasses
   share.  The actual undo/redo state machine lives in a concrete
   subclass — BlackBox uses `StdInterpreter`, which we'll port later.

   Until that lands, this module is enough to compile every framework
   module that imports `Sequencers` — `Models`, `Views`, `Documents`,
   etc. — provided their concrete `Sequencer` subclasses are also
   ported (or stubbed) before they're instantiated.

   Notifier list ownership is intentionally left non-exported: only
   `InstallNotifier` and `Notify` (the base method, not a subclass
   override) walk the chain, and the chain pointer field `next` is
   the receiver's responsibility to leave alone.
*)

    IMPORT Stores;

    CONST
        clean*       = 0;       (** modification class: harmless edits — keep undo intact *)
        notUndoable* = 1;       (** modification class: drops the undo stack *)
        invisible*   = 2;       (** modification class: ignored entirely *)

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

    VAR
        dir-: Directory;        (** the installed factory; NIL until SetDir runs *)


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
    dir := NIL
END Sequencers.
