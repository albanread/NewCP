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
     carries `opts`, `model`, `view` fields plus `SetView`,
     `ThisFocus`, `GetOpts`, `SetOpts`, `HandlePropMsg`,
     `Internalize`, `Externalize`, and ABSTRACT `Mark`,
     `Restore`, `SelectAll`.  Module-level routing procedures
     (`BroadcastMessage`, `PassFocus`, …) remain deferred until
     `Controllers.Forwarder` lands.
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
            this.  Carries the option bits, bound model and bound
            view; the full focus/mark/selection surface is declared
            below as ABSTRACT / EXTENSIBLE methods. *)
        ControllerDesc* = ABSTRACT RECORD (Controllers.ControllerDesc)
            opts-:  SET;
            model-: Model;
            view-:  View
        END;
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


    (* -- ControllerDesc methods ----------------------------------------- *)

    (** Bind the controller to its view and model. *)
    PROCEDURE (c: Controller) SetView* (v: View; m: Model), NEW;
    BEGIN
        c.view  := v;
        c.model := m
    END SetView;

    (** Return the currently focused embedded view (NIL until routing lands). *)
    PROCEDURE (c: Controller) ThisFocus* (): Views.View, NEW, EXTENSIBLE;
    BEGIN
        RETURN NIL
    END ThisFocus;

    (** Return current option bits. *)
    PROCEDURE (c: Controller) GetOpts* (OUT opts: SET), NEW, EXTENSIBLE;
    BEGIN
        opts := c.opts
    END GetOpts;

    (** Set option bits within the `valid` mask. *)
    PROCEDURE (c: Controller) SetOpts* (opts, valid: SET), NEW, EXTENSIBLE;
    BEGIN
        c.opts := (c.opts - valid) + (opts * valid)
    END SetOpts;

    (** Handle Containers.GetOpts / Containers.SetOpts property round-trips. *)
    PROCEDURE (c: Controller) HandlePropMsg* (VAR msg: Views.PropMessage), NEW, EXTENSIBLE;
    BEGIN
        WITH msg: GetOpts DO
            msg.valid := modeOpts;
            msg.opts  := c.opts * modeOpts
        | msg: SetOpts DO
            c.SetOpts(msg.opts, msg.valid * modeOpts)
        ELSE
        END
    END HandlePropMsg;

    PROCEDURE (c: Controller) Internalize* (VAR rd: Stores.Reader), EXTENSIBLE;
        VAR ver: INTEGER;
    BEGIN
        c.Internalize^(rd);
        rd.ReadVersion(minVersion, maxCtrlVersion, ver);
        IF rd.cancelled THEN RETURN END
    END Internalize;

    PROCEDURE (c: Controller) Externalize* (VAR wr: Stores.Writer), EXTENSIBLE;
    BEGIN
        c.Externalize^(wr);
        wr.WriteVersion(maxCtrlVersion)
    END Externalize;

    (** Mark the focus/selection in a frame.  ABSTRACT — subclasses implement. *)
    PROCEDURE (c: Controller) Mark* (f: Views.Frame; focus: Views.View; show: BOOLEAN), NEW, ABSTRACT;

    (** Paint controller-owned marks into a frame.  ABSTRACT — subclasses implement. *)
    PROCEDURE (c: Controller) Restore* (f: Views.Frame; l, t, r, b: INTEGER), NEW, ABSTRACT;

    (** Select or deselect all content.  ABSTRACT — subclasses implement. *)
    PROCEDURE (c: Controller) SelectAll* (select: BOOLEAN), NEW, ABSTRACT;


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
        `Views.ViewDesc.Internalize` (which reads the
        `Views.View` version stamp), reads this layer's own
        `maxViewVersion` stamp, then defers to the
        subclass-supplied `Internalize2`.

        In BlackBox the version stamp is immediately followed by
        the embedded Model store and the (optional) Controller
        store; their materialisation (`rd.ReadStore` /
        `Stores.NewStore` / `TurnIntoAlien`) is deferred until
        the Kernel RTTI factory lands.  Subclass
        `Internalize2` bodies (`TextViews.StdView`) read those
        child stores directly after the stamp. *)
    PROCEDURE (v: View) Internalize* (VAR rd: Stores.Reader);
        VAR ver: INTEGER;
    BEGIN
        v.Internalize^(rd);
        IF rd.cancelled THEN RETURN END;
        rd.ReadVersion(minVersion, maxViewVersion, ver);
        IF rd.cancelled THEN RETURN END;
        v.Internalize2(rd)
    END Internalize;

    (** Symmetric Externalize — writes the `Containers.View`
        version stamp after the super layer's stamp, then
        delegates to `Externalize2` for subclass fields. *)
    PROCEDURE (v: View) Externalize* (VAR wr: Stores.Writer);
    BEGIN
        v.Externalize^(wr);
        wr.WriteVersion(maxViewVersion);
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

    (** Bind a controller to the view and wire it back to view/model. *)
    PROCEDURE (v: View) SetController* (c: Controller), NEW, EXTENSIBLE;
    BEGIN
        v.controller := c;
        IF c # NIL THEN c.SetView(v, v.model) END
    END SetController;

    (** Return the bound controller. *)
    PROCEDURE (v: View) ThisController* (): Controller, NEW, EXTENSIBLE;
    BEGIN
        RETURN v.controller
    END ThisController;

END Containers.
