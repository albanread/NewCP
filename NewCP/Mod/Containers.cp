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
     and everything that touches it: the `controller` field on
     `View`, `SetController`, `ThisController`, the focus / mark
     protocol, the controller half of Internalize/Externalize.
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

    IMPORT Stores, Models, Views;

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
            wraps; the `controller` slot is deferred to the next slice
            (waiting on `Controllers` to port).  `alienCtrl` is a
            placeholder for a deserialised controller whose concrete
            type wasn't in scope at load time. *)
        ViewDesc* = ABSTRACT RECORD (Views.ViewDesc)
            model-:     Model;
            alienCtrl-: Stores.Store
        END;
        View* = POINTER TO ViewDesc;


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

    (** EXTENSIBLE Internalize chain — super-calls into
        `Models.ModelDesc.Internalize`.  BlackBox additionally
        reads a `maxModelVersion` stamp here; we'll restore that
        once `Stores.Reader.ReadVersion` lands. *)
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
        `Views.ViewDesc.Internalize`.  The BlackBox version also
        reads a `maxViewVersion` stamp here, then reads the
        embedded Model store and the (optional) Controller
        store and binds them; the Reader extensions that needs
        aren't ported yet. *)
    PROCEDURE (v: View) Internalize* (VAR rd: Stores.Reader);
    BEGIN
        v.Internalize^(rd);
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
