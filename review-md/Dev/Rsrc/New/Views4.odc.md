MODULE NewViews;

*(*  *)*

    IMPORT Stores, Ports, Models, Views, Controllers, Properties;

    CONST minVersion = 0; maxVersion = 0;

    TYPE

        Model = POINTER TO RECORD (Models.Model)

            **(* model fields *)**

        END;

        UpdateMsg = RECORD (Models.UpdateMsg)

            **(* message fields *)**

        END;

        ModelOp = POINTER TO RECORD (Stores.Operation)

            model: Model;

            **(* model-operation fields *)**

        END;

        View = POINTER TO RECORD (Views.View)

            model: Model;

            **(* view fields *)**

        END;

        ViewOp = POINTER TO RECORD (Stores.Operation)

            view: View;

            **(* view-operation fields *)**

        END;

    *(* ModelOp *)*

    PROCEDURE (op: ModelOp) Do;

        VAR msg: UpdateMsg;

    BEGIN

        **(* perform model operation and set up the fields of the update message accordingly *)**

        Models.Broadcast(op.model, msg)    *(* update all views on this model *)*

    END Do;

    PROCEDURE NewModelOp (model: Model **(* additional parameters *)** ): ModelOp;

        VAR op: ModelOp;

    BEGIN

        ASSERT(model # NIL, 100);

        NEW(op); op.model := model;

        **(* set up operation parameters *)**

        **RETURN** op

    END NewModelOp;

    *(* Model *)*

    PROCEDURE (m: Model) Internalize (VAR rd: Stores.Reader);

        VAR version: INTEGER;

    BEGIN

        *(* m is not initialized *)*

*        (* m.Domain() = NIL *)*

        IF ~rd.cancelled THEN

            rd.ReadVersion(minVersion, maxVersion, version);

            IF ~rd.cancelled THEN

                **(* read model fields *)**

            END

        END

    END Internalize;

    PROCEDURE (m: Model) Externalize (VAR wr: Stores.Writer);

    BEGIN

        *(* m is initialized *)*

        wr.WriteVersion(maxVersion);

        **(* write model fields *)**

    END Externalize;

    PROCEDURE (m: Model) CopyFrom (source: Stores.Store);

    BEGIN

        *(* m is not initalized *)*

*        (* m.Domain() = NIL *)*

*        (* source # NIL *)*

*        (* source is initialized *)*

*        (* TYP(source) = TYP(m) *)*

        WITH source: Model DO

            **(* perform deep copy of source *)**

        END

    END CopyFrom;

    *(* ViewOp *)*

    PROCEDURE (op: ViewOp) Do;

    BEGIN

        **(* perform view operation *)**

        Views.Update(op.view, Views.keepFrames)    *(* restore v in any frame that displays it *)*

    END Do;

    PROCEDURE NewViewOp (view: View **(* additional parameters *)** ): ViewOp;

        VAR op: ViewOp;

    BEGIN

        ASSERT(view # NIL, 100);

        NEW(op); op.view := view;

        **(* set up operation parameters *)**

        **RETURN** op

    END NewViewOp;

    *(* View *)*

    PROCEDURE (v: View) Internalize (VAR rd: Stores.Reader);

        VAR version: INTEGER; st: Stores.Store;

    BEGIN

        *(* v is not initialized *)*

*        (* v.Domain() = NIL *)*

        IF ~rd.cancelled THEN

            rd.ReadVersion(minVersion, maxVersion, version);

            IF ~rd.cancelled THEN

                rd.ReadStore(st);

                v. model := st(Model);

                **(* read other view fields *)**

            END

        END

    END Internalize;

    PROCEDURE (v: View) Externalize (VAR wr: Stores.Writer);

    BEGIN

        *(* v is initialized *)*

*        (* v.model # NIL *)*

*        (* v.model IS Model *)*

        wr.WriteVersion(maxVersion);

        wr.WriteStore(v.model);

        **(* write view fields *)**

    END Externalize;

    PROCEDURE (v: View) CopyFromModelView (source: Views.View; model: Models.Model);

    BEGIN

        *(* v is not initialized *)*

*        (* v.Domain() = NIL *)*

*        (* source # NIL *)*

*        (* source is initialized *)*

*        (* TYP(source) = TYP(v) *)*

*        (* source.model # NIL *)*

*        (* model # NIL *)*

        ASSERT(model IS Model, 20);

        WITH source: View DO

            v.model := model(Model);

            **(* copy other view fields *)**

            **(***

**                Check and possibly update or initialize v's state which refers to its model.**

**                Example: scroll position is set to a legal value, e.g. to the beginning**

**            *)**

        END

    END CopyFromModelView;

    PROCEDURE (v: View) ThisModel (): Models.Model;

    BEGIN

        **RETURN** v.model

    END ThisModel;

    PROCEDURE (v: View) Restore (f: Views.Frame; l, t, r, b: INTEGER);

        **VAR w, h: INTEGER;**

    BEGIN

        *(* f # NIL *)*

        **(* restore foreground in rectangle (l, t, r, b) *)**

        **(* replace the body of this procedure with your Restore behavior *)**

        **v.context.GetSize(w, h);**

        **f.DrawLine(0, 0, w, h, f.dot, Ports.red)**

    END Restore;

    PROCEDURE (v: View) HandleModelMsg (VAR msg: Models.Message);

    BEGIN

        *(* msg.model # NIL *)*

*        (* msg.model = v.model *)*

        WITH msg: Models.UpdateMsg DO

            WITH msg: UpdateMsg DO

                **(* calculate bounding box of area to restore, and then call**

**                Views.UpdateIn(v, l, t, r, b, Views.keepFrames)**

**                *)**

            ELSE

                Views.Update(v, Views.keepFrames)    *(* restore v in any frame that displays it *)*

            END

        ELSE    *(* ignore other messages *)*

        END

    END HandleModelMsg;

    PROCEDURE (v: View) HandleCtrlMsg (f: Views.Frame; VAR msg: Controllers.Message; VAR focus: Views.View);

    BEGIN

        *(* f # NIL *)*

*        (* focus = NIL *)*

        WITH msg: Controllers.PollOpsMsg DO

            **(* specify which editing operations are supported *)**

        | msg: Controllers.TrackMsg DO

            **(* implement mouse tracking *)**

        | msg: Controllers.EditMsg DO

            **(* implement editing operations *)**

        ELSE*    (* ignore other messages *)*

        END

    END HandleCtrlMsg;

    PROCEDURE (v: View) HandlePropMsg (VAR p: Properties.Message);

        CONST defaultWidth = 100 * Ports.mm; defaultHeight = 70 * Ports.mm;

    BEGIN

        WITH p: Properties.FocusPref DO

            p.setFocus := TRUE

        | p: Properties.SizePref DO

            IF p.w = Views.undefined THEN p.w := defaultWidth END;

            IF p.h = Views.undefined THEN p.h := defaultHeight END

        ELSE    *(* ignore other messages *)*

        END

    END HandlePropMsg;

    *(** miscellaneous **)*

    PROCEDURE **Focus*** (): Views.View;

        VAR v: Views.View;

    BEGIN

        v := Controllers.FocusView();

        IF (v # NIL) & (v IS View) THEN **RETURN** v(View) ELSE **RETURN** NIL END

    END Focus;

    PROCEDURE **New*** (): Views.View;

        VAR m: Model; v: View;

    BEGIN

        NEW(m);

        **(* initialize model fields *)**

        NEW(v); v.model := m; Stores.Join(v, m);

        **(* initialize other view fields *)**

        **RETURN** v

    END New;

    PROCEDURE **Deposit***;

    BEGIN

        Views.Deposit(New())

    END Deposit;

END NewViews.

