MODULE NewViews;

*(*  *)*

    IMPORT Ports, Stores, Models, Views, Controllers, Properties, NewModels;

    CONST minVersion = 0; maxVersion = 0;

    TYPE

        **View*** = POINTER TO ABSTRACT RECORD (Views.View) END;

        **Directory*** = POINTER TO ABSTRACT RECORD END;

        StdView = POINTER TO RECORD (View)

            model: NewModels.Model;

            **(* view fields *)**

        END;

        StdDirectory = POINTER TO RECORD (Directory) END;

        Op = POINTER TO RECORD (Stores.Operation)

            view: StdView;

            **(* view-operation fields *)**

        END;

    VAR **dir**-, **stdDir**-: Directory;

    *(** View **)*

    PROCEDURE (v: View) **ThisModel*** ():  NewModels.Model, EXTENSIBLE; END ThisModel;

    *(* covariant narrowing of function result *)*

    *(** Directory **)*

    PROCEDURE (d: Directory) **New*** (m: NewModels.Model): View, NEW, ABSTRACT;

    *(* Op *)*

    PROCEDURE (op: Op) Do;

    BEGIN

        **(* perform view operation *)**

        Views.Update(op.view, Views.keepFrames)    *(* restore v in any frame that displays it *)*

    END Do;

    PROCEDURE NewOp (view: StdView **(* additional parameters *)** ): Op;

        VAR op: Op;

    BEGIN

        ASSERT(view # NIL, 100);

        NEW(op); op.view := view;

        **(* set up operation parameters *)**

        **RETURN** op

    END NewOp;

    *(* StdView *)*

    PROCEDURE (v: StdView) Internalize (VAR rd: Stores.Reader);

        VAR version: INTEGER; st: Stores.Store;

    BEGIN

        *(* v is not initialized *)*

*        (* v.Domain() = NIL *)*

        IF ~rd.cancelled THEN

            rd.ReadVersion(minVersion, maxVersion, version);

            IF ~rd.cancelled THEN

                rd.ReadStore(st);

                IF st IS NewModels.Model THEN

                    v.model := st(NewModels.Model);

                    **(* read other view fields *)**

                ELSE    *(* concrete model implementation couldn't be loaded-> an alien store was created *)*

                    rd.TurnIntoAlien(Stores.alienComponent)    *(* cancel internalization of *v* *)*

                END

            END

        END

    END Internalize;

    PROCEDURE (v: StdView) Externalize (VAR wr: Stores.Writer);

    BEGIN

        *(* v is initialized *)*

        *(* v.model # NIL *)*

*        (* v.model IS NewModels.Model *)*

        wr.WriteVersion(maxVersion);

        wr.WriteStore(v.model);

        **(* write view fields *)**

    END Externalize;

    PROCEDURE (v: StdView) CopyFromModelView (source: Views.View; model: Models.Model);

    BEGIN

        *(* v is not initialized *)*

*        (* v.Domain() = NIL *)*

*        (* source # NIL *)*

*        (* source is initialized *)*

*        (* TYP(source) = TYP(v) *)*

*        (* source.model # NIL *)*

*        (* model # NIL *)*

        ASSERT(model IS NewModels.Model, 20);

        WITH source: View DO

            v.model := model(NewModels.Model);

            **(* copy other view fields *)**

            **(***

**                Check and possibly update or initialize v's state which refers to its model.**

**                Example: scroll position is set to a legal value, e.g. to the beginning**

**            *)**

        END

    END CopyFromModelView;

    PROCEDURE (v: StdView) ThisModel (): NewModels.Model;

    BEGIN

        **RETURN** v.model

    END ThisModel;

    PROCEDURE (v: StdView) Restore (f: Views.Frame; l, t, r, b: INTEGER);

        **VAR w, h: INTEGER;**

    BEGIN

        **(* restore foreground in rectangle (l, t, r, b) *)**

        **(* replace the body of this procedure with your Restore behavior *)**

        **v.context.GetSize(w, h);**

        **f.DrawLine(0, 0, w, h, f.dot, Ports.red)**

    END Restore;

    PROCEDURE (v: StdView) HandleModelMsg (VAR msg: Models.Message);

    BEGIN

        WITH msg: Models.UpdateMsg DO

            WITH msg: NewModels.UpdateMsg DO

                **(* calculate bounding box of area to restore, and then call**

**                Views.UpdateIn(v, l, t, r, b, Views.keepFrames)**

**                *)**

            ELSE

                Views.Update(v, Views.keepFrames)    *(* restore v in any frame that displays it *)*

            END

        ELSE    *(* ignore other messages *)*

        END

    END HandleModelMsg;

    PROCEDURE (v: StdView) HandleCtrlMsg (f: Views.Frame; VAR msg: Controllers.Message;

                                                                                        VAR focus: Views.View);

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

    PROCEDURE (v: StdView) HandlePropMsg (VAR p: Properties.Message);

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

    *(* StdDirectory *)*

    PROCEDURE (d: StdDirectory) New (m: NewModels.Model): View;

        VAR v: StdView;

    BEGIN

        ASSERT(m # NIL, 20);

        (*ASSERT(**m already initialized**, 21);*)

        NEW(v); v.model := m; Stores.Join(v, m);

        **(* initialize other view fields *)**

        **RETURN** v

    END New;

    *(** miscellaneous **)*

    PROCEDURE **Focus*** (): View;

        VAR v: Views.View;

    BEGIN

        v := Controllers.FocusView();

        IF (v # NIL) & (v IS View) THEN **RETURN** v(View) ELSE **RETURN** NIL END

    END Focus;

    PROCEDURE **FocusModel*** (): NewModels.Model;

        VAR v: Views.View;

    BEGIN

        v := Controllers.FocusView();

        IF (v # NIL) & (v IS View) THEN **RETURN** v(View).ThisModel() ELSE **RETURN** NIL END

    END FocusModel;

    PROCEDURE **Deposit***;

    BEGIN

        Views.Deposit(dir.New(NewModels.dir.New()))

    END Deposit;

    PROCEDURE **SetDir*** (d: Directory);

    BEGIN

        ASSERT(d # NIL, 20);

        dir := d

    END SetDir;

    PROCEDURE Init;

        VAR d: StdDirectory;

    BEGIN

        NEW(d); stdDir := d; dir := d

    END Init;

BEGIN

    Init

END NewViews.

