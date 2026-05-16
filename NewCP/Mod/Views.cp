MODULE Views;
(*
   First slice of the BlackBox `Views` port.

   The full BlackBox `Views.Mod` is ~2380 lines of UI / framework
   code; this slice ships ONLY the surface needed by the framework
   modules above it to extend `Views.View` and chain through its
   abstract Store protocol.  Concretely that means:

   - `ViewDesc` extending `Stores.StoreDesc` with the four BlackBox
     instance fields (`context`, `era`, `guard`, `bad`).  `View` is
     ABSTRACT — subclasses (`Containers.View`, `TextViews.View`,
     `TextViews.StdView`, …) override the protocol below.

   - The complete Model / Store / Frame / Handler method shape on
     `View` so a subclass can use the super-call chain without
     hitting "method not overridden" at link time.  Methods that are
     EMPTY in BlackBox stay EMPTY here; `Internalize`/`Externalize`
     keep their version-stamp read/write so wire-format compat
     survives.

   - The message records (`Message`, `NotifyMsg`, `UpdateCachesMsg`,
     `ScrollClassMsg`, `PropMessage`, `CtrlMessage`) for Handler
     code to declare its message types against.

   - The `Frame` record extending `Ports.Frame` so subclass Restore
     methods can accept the right type.

   Deferred (and explicitly NOT in this slice):

   - Module-level procedures (`Open`, `OldView`, `RegisterView`,
     `Update`, `Restore`, `Background`, …) — depend on
     Files/Converters/Dialog and the View-producer queue.
   - The `Region` / `Rect` / `RootFrame` / `StdFrame` / `QueueElem`
     internal types — only the View dispatch chain needs to compile.
   - The `GetSpecHook` / `ViewHook` / `MsgHook` / `NotifyHook`
     types — depend on Kernel.Hook, Dialog, Converters which are
     not all ported.
   - The `Overwritten` helper — uses `SYSTEM.TYP(View) - 4*(mno+1)`
     pointer arithmetic against the typedesc layout; defer until
     the typedesc ABI is documented.
*)

    IMPORT Stores, Models, Ports, Sequencers;

    CONST
        (** View.Background color *)
        transparent* = 0FF000000H;

        (** Views.CopyModel / Views.CopyOf shallow flag. *)
        deep*    = FALSE;
        shallow* = TRUE;

        (** Update / UpdateIn rebuild flag. *)
        keepFrames*    = FALSE;
        rebuildFrames* = TRUE;

        (** Deposit / QualifiedDeposit / Fetch sentinels. *)
        undefined* = 0;

        (** OldView / RegisterView "ask?" flag. *)
        dontAsk* = FALSE;
        ask*     = TRUE;

        (* method numbers used by `Overwritten` (deferred). *)
        copyFromModelView  = 7;
        copyFromSimpleView = 8;

        (* Frame.state *)
        new    = 0;
        open   = 1;
        closed = 2;

        maxN = 30;                 (* max rects approximating a region *)

        minVersion = 0;
        maxVersion = 0;

        (* actOp *)
        handler        = 1;
        restore        = 2;
        externalize    = 3;
        markBorderSize = 2;

        clean*       = Sequencers.clean;
        notUndoable* = Sequencers.notUndoable;
        invisible*   = Sequencers.invisible;


    TYPE
        (** Abstract base for every visible view.  Concrete views
            (`Containers.View`, `TextViews.View`, `TextViews.StdView`)
            extend this; the framework dispatches the message handler
            and Frame protocols below through their vtables. *)
        ViewDesc* = ABSTRACT RECORD (Stores.StoreDesc)
            context-: Models.Context;   (** stable context # NIL **)
            era-:     INTEGER;
            guard-:   INTEGER;          (* TrapCount()+1 if broadcasting *)
            bad-:     SET
        END;
        View* = POINTER TO ViewDesc;

        (** Title buffer for `Open` / `RegisterView` (fixed-size). *)
        Title* = ARRAY 64 OF CHAR;

        (** Abstract base for view-side frames — the per-render
            window into a `View`.  Real frames extend this with
            device-specific state; the field set here matches
            `BlackBox`'s `Views.Frame`. *)
        FrameDesc* = ABSTRACT RECORD (Ports.FrameDesc)
            l-, t-, r-, b-: INTEGER;    (** l < r, t < b **)
            view-:          View;       (** opened => view # NIL **)
            front-, mark-:  BOOLEAN;
            state-:         BYTE;
            x-, y-:         INTEGER;    (* origin in env coords *)
            gx0-, gy0-:     INTEGER;    (* global origin w/o scroll *)
            sx-, sy-:       INTEGER;    (* sub-pixel scroll comp *)
            level-:         INTEGER     (* partial z-ordering *)
        END;
        Frame* = POINTER TO FrameDesc;

        (** Abstract Message base.  Every handler-dispatched
            message extends this and is broadcast through the
            view's frame tree. *)
        Message* = ABSTRACT RECORD
            view-: View                (** view # NIL **)
        END;

        (** Notification message — passed to interested
            consumers when a view's state changes. *)
        NotifyMsg* = EXTENSIBLE RECORD (Message)
            id0*, id1*: INTEGER;
            opts*:      SET
        END;

        (** Caches-rebuild message — broadcast when a view's
            content has invalidated derived caches. *)
        UpdateCachesMsg* = EXTENSIBLE RECORD (Message) END;

        (** Scroll-class probe — receiver sets
            `allowBitmapScrolling` to opt out of bitmap blit
            during scroll. *)
        ScrollClassMsg* = RECORD (Message)
            allowBitmapScrolling*: BOOLEAN   (** OUT, preset to FALSE **)
        END;

        (** Abstract Property message base.  Property setters
            extend this to round-trip key/value updates. *)
        PropMessage* = ABSTRACT RECORD END;

        (** Abstract Controller message base.  Controllers
            extend this to forward input / focus / selection
            events into the view. *)
        CtrlMessage* = ABSTRACT RECORD END;


    (* -- View methods (Model protocol) ----------------------------------- *)

    (** "Source" was constructed by the same OO machinery — its
        model state is meaningful to the receiver. *)
    PROCEDURE (v: View) CopyFromSimpleView* (source: View), NEW, EMPTY;

    (** "Source" carries an independent model the receiver should
        wrap.  Default is EMPTY — concrete views override one of
        the two CopyFrom variants. *)
    PROCEDURE (v: View) CopyFromModelView* (source: View; model: Models.Model), NEW, EMPTY;

    (** Default model accessor.  Concrete views that wrap a
        model override; bare views return NIL. *)
    PROCEDURE (v: View) ThisModel* (): Models.Model, NEW, EXTENSIBLE;
    BEGIN
        RETURN NIL
    END ThisModel;


    (* -- View methods (Store protocol) ----------------------------------- *)

    (** EXTENSIBLE Internalize chain — super-calls into
        `Stores.StoreDesc.Internalize` (EMPTY), then reads the
        BB-faithful `Views.View` version stamp.  Every concrete
        view store body starts with this byte in the wire format
        (after the `Stores.Store` version byte that the super
        layer's ReadVersion will eventually consume).
        Subclasses (`TextViews.StdView`) that materialise all
        version bytes themselves with raw `ReadByte` loops do
        NOT go through this chain — they override Internalize
        outright, so the version stamp here is consumed
        exclusively by views that use the normal super-call
        protocol (`Containers.View`, `TextRulers.StdRuler`, …). *)
    PROCEDURE (v: View) Internalize* (VAR rd: Stores.Reader), EXTENSIBLE;
        VAR ver: INTEGER;
    BEGIN
        v.Internalize^(rd);
        rd.ReadVersion(minVersion, maxVersion, ver);
        IF rd.cancelled THEN RETURN END
    END Internalize;

    (** Symmetric Externalize chain — writes the `Views.View`
        version stamp after the super layer's stamp. *)
    PROCEDURE (v: View) Externalize* (VAR wr: Stores.Writer), EXTENSIBLE;
    BEGIN
        v.Externalize^(wr);
        wr.WriteVersion(maxVersion)
    END Externalize;


    (* -- View methods (embedding protocol) ------------------------------- *)

    (** Stable context for the view's lifetime.  Asserts the
        argument is non-NIL and that the receiver hasn't already
        been bound to a different context. *)
    PROCEDURE (v: View) InitContext* (context: Models.Context), NEW, EXTENSIBLE;
    BEGIN
        ASSERT(context # NIL, 21);
        ASSERT((v.context = NIL) OR (v.context = context), 22);
        v.context := context
    END InitContext;

    (** Read the view's preferred background color.  Default is
        EMPTY — concrete views override. *)
    PROCEDURE (v: View) GetBackground* (VAR color: Ports.Color), NEW, EMPTY;

    (** A child view (`view`) wants to take focus.  Default is
        EMPTY — container views override to mediate focus. *)
    PROCEDURE (v: View) ConsiderFocusRequestBy* (view: View), NEW, EMPTY;

    (** Tear down any per-view transient state.  EMPTY default. *)
    PROCEDURE (v: View) Neutralize*, NEW, EMPTY;


    (* -- View methods (Frame protocol) ----------------------------------- *)

    (** Hand out a frame descriptor for a fresh display of the
        view.  EMPTY default (frame stays NIL). *)
    PROCEDURE (v: View) GetNewFrame* (VAR frame: Frame), NEW, EMPTY;

    (** Paint the rectangle `[l, t, r, b]` into `f`.  ABSTRACT —
        every concrete view must implement. *)
    PROCEDURE (v: View) Restore* (f: Frame; l, t, r, b: INTEGER), NEW, ABSTRACT;

    (** Optional second-pass marks pass (selection, cursors).
        EMPTY default — only views that own selection state
        override. *)
    PROCEDURE (v: View) RestoreMarks* (f: Frame; l, t, r, b: INTEGER), NEW, EMPTY;


    (* -- View methods (handler protocol) --------------------------------- *)

    (** Receive a Models.Message broadcast.  EMPTY default. *)
    PROCEDURE (v: View) HandleModelMsg* (VAR msg: Models.Message), NEW, EMPTY;

    (** Receive a View Message broadcast.  EMPTY default. *)
    PROCEDURE (v: View) HandleViewMsg* (f: Frame; VAR msg: Message), NEW, EMPTY;

    (** Receive a PropMessage round-trip.  EMPTY default — only
        views that publish properties override. *)
    PROCEDURE (v: View) HandlePropMsg* (VAR msg: PropMessage), NEW, EMPTY;


    (* -- View methods (Stores.Store overrides) --------------------------- *)

    (** Required by `Stores.Store` (ABSTRACT there).  Views
        without a model don't have a Domain — return NIL.
        Concrete subclasses that wrap a model override to return
        the model's domain. *)
    PROCEDURE (v: View) Domain* (): Stores.Domain;
    BEGIN
        RETURN NIL
    END Domain;


END Views.
