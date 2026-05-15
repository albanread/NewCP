MODULE Containers;
(*
   First slice of the BlackBox `Containers` port.

   `Containers` sits between `Views` / `Models` and concrete
   editors (TextViews, FormViews) — it carries the "every view
   wraps a model" shape and the Controller-mediated focus
   protocol.

   This slice ships the TYPE hierarchy and the abstract surface
   that subclasses need to extend. It deliberately omits the
   pieces that need modules not yet ported:

   - `Containers.Controller` (extends `Controllers.Controller`)
     is included as an abstract base now that `Controllers` has
     landed; its concrete method shape (`SetController`,
     `ThisController`, the focus / mark protocol, the controller
     half of Internalize/Externalize) is still deferred until
     `Controllers.Forwarder` and the module-level routing
     procedures port.
   - `DropPref` (extends `Properties.Preference`).
   - `ViewOp` / `ControllerOp` undo operations.
   - The Internalize/Externalize bodies that call `rd.ReadVersion`
     / `rd.ReadStore` / `rd.cancelled` / `wr.WriteVersion` —
     those `Stores.Reader` / `Stores.Writer` extensions are not
     wired yet. Our chain super-calls only.

   The slice IS enough to define `Containers.Model` and
   `Containers.View` as the parent types `TextModels.StdModel` and
   `TextViews.StdView` will eventually extend, and to verify the
   3-level vtable chain `Stores.Store -> Models.Model ->
   Containers.Model` works end-to-end.
*)

    IMPORT Stores, Models, Views, Controllers;

    CONST
        (** Controller.opts — option set for selection/focus/caret display. *)
        noSelection* = 0;
        noFocus*     = 1;
        noCaret*     = 2;

        mask*   = {noSelection, noCaret};
        layout* = {noFocus};

        modeOpts = {noSelection, noFocus, noCaret};

        (** Controller.SelectAll select flag. *)
        deselect* = FALSE;
        select*   = TRUE;

        (** Property polling: ANY vs SELECTION scope. *)
        any*       = FALSE;
        selection* = TRUE;

        (** Mark / MarkCaret / MarkSelection / MarkSingleton show flag. *)
        hide* = FALSE;
        show* = TRUE;

        indirect = FALSE;
        direct   = TRUE;

        (** Wire-format version stamps (each layer reads its own). *)
        minVersion      = 0;
        maxModelVersion = 0;
        maxViewVersion  = 0;
        maxCtrlVersion  = 0;


    TYPE
        (** Container-side abstract model.  Concrete container models
            (`TextModels.StdModel`, `FormModels.Model`) extend this.
            The base contributes the Container-level Internalize /
            Externalize version stamp wrapper plus two ABSTRACT
            embedding hooks. *)
        ModelDesc* = ABSTRACT RECORD (Models.ModelDesc) END;
        Model*     = POINTER TO ModelDesc;

        (** Container-side abstract view.  Carries the model the view
            wraps and the controller that mediates input/focus for
            it.  `alienCtrl` is a placeholder for a deserialised
            controller whose concrete type wasn't in scope at load
            time. *)
        ViewDesc* = ABSTRACT RECORD (Views.ViewDesc)
            model-:      Model;
            controller-: Controller;
            alienCtrl-:  Stores.Store
        END;
        View* = POINTER TO ViewDesc;

        (** Container-side abstract controller.  Concrete container
            controllers (`TextControllers.Controller`, …) extend
            this.  The full BlackBox surface adds `opts`, `model`,
            `view`, `focus`, `singleton`, `bVis` instance fields
            and a large method bundle; this slice carries just the
            type so `Containers.ViewDesc.controller` can be typed
            against it.  Subsequent slices grow the field set as
            `Forwarder` and the routing procs port. *)
        ControllerDesc* = ABSTRACT RECORD (Controllers.ControllerDesc) END;
        Controller*     = POINTER TO ControllerDesc;

        (** Container-side abstract directory.  Concrete container
            controllers' Directory types (`TextControllers.Directory`,
            `FormControllers.Directory`, …) extend this.  BB-faithful
            base type — empty record per the framework convention; the
            specialisation methods (`NewController`, `New`) live on
            the derived directories. *)
        DirectoryDesc* = ABSTRACT RECORD END;
        Directory*     = POINTER TO DirectoryDesc;


        (** Common base for the container-flavoured view messages
            below.  Distinguishes container-broadcast messages from
            the generic `Views.Message`. *)
        ViewMessage* = ABSTRACT RECORD (Views.Message) END;

        FocusMsg* = RECORD (ViewMessage)
            set*: BOOLEAN
        END;

        SingletonMsg* = RECORD (ViewMessage)
            set*: BOOLEAN
        END;

        FadeMsg* = RECORD (ViewMessage)
            show*: BOOLEAN
        END;


        (** Property round-trips for the Container "options" SET
            (noSelection / noFocus / noCaret).  GetOpts is a query
            (valid OUT mask, opts OUT bits); SetOpts is a write
            (valid mask + opts bits to set). *)
        GetOpts* = RECORD (Views.PropMessage)
            valid*, opts*: SET
        END;

        SetOpts* = RECORD (Views.PropMessage)
            valid*, opts*: SET
        END;


    (* -- ModelDesc abstract surface ------------------------------------- *)

    (** Read the embedding's preferred geometry limits — used by
        the host frame to negotiate sizing. *)
    PROCEDURE (m: Model) GetEmbeddingLimits* (OUT minW, maxW, minH, maxH: INTEGER), NEW, ABSTRACT;

    (** Swap an embedded view in-place — used when the host
        promotes / demotes / replaces a child view through
        external commands. *)
    PROCEDURE (m: Model) ReplaceView* (old, new: Views.View), NEW, ABSTRACT;

    (** Optional "I'm being initialised by copying from this
        other model" hook.  EMPTY default — only models that
        carry state outside the wire format override. *)
    PROCEDURE (m: Model) InitFrom* (source: Model), NEW, EMPTY;

    (** EXTENSIBLE Internalize chain — super-call into
        `Models.ModelDesc.Internalize`.  See the matching
        Views.View Internalize comment on the deferred
        `ReadVersion`. *)
    PROCEDURE (m: Model) Internalize* (VAR rd: Stores.Reader), EXTENSIBLE;
    BEGIN
        m.Internalize^(rd)
    END Internalize;

    (** Symmetric Externalize chain. *)
    PROCEDURE (m: Model) Externalize* (VAR wr: Stores.Writer), EXTENSIBLE;
    BEGIN
        m.Externalize^(wr)
    END Externalize;


    (* -- ViewDesc abstract surface -------------------------------------- *)

    (** Filter on which models the view can adopt.  ABSTRACT —
        every concrete container view must implement.  Called by
        `InitModel` to assert that `m`'s type is compatible
        before storing it. *)
    PROCEDURE (v: View) AcceptableModel* (m: Model): BOOLEAN, NEW, ABSTRACT;

    (** Subclass hook for additional post-model-bind init —
        EMPTY default. *)
    PROCEDURE (v: View) InitModel2* (m: Model), NEW, EMPTY;

    (** Bind the view to a model.  Asserts model is acceptable
        (via the ABSTRACT `AcceptableModel`), then stores it.
        BlackBox additionally registers the view as a notifier
        on the model via `Stores.Join(v, m)`; that's deferred
        until Stores grows a domain-bookkeeping surface. *)
    PROCEDURE (v: View) InitModel* (m: Model), NEW;
    BEGIN
        ASSERT((v.model = NIL) OR (v.model = m), 20);
        ASSERT(m # NIL, 21);
        ASSERT(v.AcceptableModel(m), 22);
        v.model := m;
        v.InitModel2(m)
    END InitModel;

    (** Subclass hook for additional fields the View-layer
        Externalize should emit. EMPTY default. *)
    PROCEDURE (v: View) Externalize2* (VAR wr: Stores.Writer), NEW, EMPTY;

    (** Symmetric subclass hook for additional fields after the
        Container-level Internalize chain finishes. *)
    PROCEDURE (v: View) Internalize2* (VAR rd: Stores.Reader), NEW, EMPTY;

    (** Container-level Internalize.  Super-calls into
        `Views.ViewDesc.Internalize`, then defers to the
        subclass-supplied `Internalize2`.

        The BlackBox version additionally reads a
        `maxViewVersion` stamp here, then reads the embedded
        Model store, the (optional) Controller store, and binds
        them on `v.model` / `v.controller` / `v.alienCtrl` —
        with type-guard checks falling back to `TurnIntoAlien`
        on mismatch.  That materialization step still needs the
        Kernel.NewObj-driven RTTI factory that turns a
        `ReaderHandle` for an inline store into a typed
        `Stores.Store`-rooted value without introducing an import
        cycle here.  Until that direct runtime path lands,
        subclass Internalize2
        bodies (`TextViews.View.Internalize2`) handle BOTH the
        version stamp AND the child-store reads via
        `rd.ReadVersion` / `rd.ReadStore` / SkipStore. *)
    PROCEDURE (v: View) Internalize* (VAR rd: Stores.Reader);
    BEGIN
        v.Internalize^(rd);
        IF rd.cancelled THEN RETURN END;
        v.Internalize2(rd)
    END Internalize;

    (** Symmetric Externalize.  Subclass-supplied
        `Externalize2` runs at the end. *)
    PROCEDURE (v: View) Externalize* (VAR wr: Stores.Writer);
    BEGIN
        v.Externalize^(wr);
        v.Externalize2(wr)
    END Externalize;

    (** Default model accessor — return the bound model. *)
    PROCEDURE (v: View) ThisModel* (): Models.Model, EXTENSIBLE;
    BEGIN
        RETURN v.model
    END ThisModel;

    (** Optional "I'm being initialised by copying from this
        other view, here's the cloned model" hook.  EMPTY default.
        Mirrors the Container shape of `View.CopyFromModelView`. *)
    PROCEDURE (v: View) CopyFromModelView2* (source: Views.View; model: Models.Model), NEW, EMPTY;

END Containers.
