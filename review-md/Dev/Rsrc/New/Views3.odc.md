MODULE NewViews;

*(*  *)*

    IMPORT Stores, Ports, Views, Controllers, Properties;

    CONST minVersion = 0; maxVersion = 0;

    TYPE

        View = POINTER TO RECORD (Views.View)

            **(* view fields *)**

        END;

        ViewOp = POINTER TO RECORD (Stores.Operation)

            view: View;

            **(* view-operation fields *)**

        END;

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

        VAR version: INTEGER;

    BEGIN

        *(* v is not initialized *)*

*        (* v.Domain() = NIL *)*

        IF ~rd.cancelled THEN

            rd.ReadVersion(minVersion, maxVersion, version);

            IF ~rd.cancelled THEN

                **(* read view fields *)**

            END

        END

    END Internalize;

    PROCEDURE (v: View) Externalize (VAR wr: Stores.Writer);

    BEGIN

        *(* v is initialized *)*

        wr.WriteVersion(maxVersion);

        **(* write view fields *)**

    END Externalize;

    PROCEDURE (v: View) CopyFromSimpleView (source: Views.View);

    BEGIN

        *(* v is not initialized *)*

*        (* v.Domain() = NIL *)*

*        (* source # NIL *)*

*        (* source is initialized *)*

*        (* TYP(v) = TYP(source) *)*

        WITH source: View DO

            **(* copy view fields *)**

        END

    END CopyFromSimpleView;

    PROCEDURE (v: View) Restore (f: Views.Frame; l, t, r, b: INTEGER);

        **VAR w, h: INTEGER;**

    BEGIN

        *(* f # NIL *)*

        **(* restore foreground in rectangle (l, t, r, b) *)**

        **(* replace the body of this procedure with your Restore behavior *)**

        **v.context.GetSize(w, h);**

        **f.DrawLine(0, 0, w, h, f.dot, Ports.red)**

    END Restore;

    PROCEDURE (v: View) HandleCtrlMsg (f: Views.Frame; VAR msg: Controllers.Message; VAR focus: Views.View);

    BEGIN

        *(* f # NIL *)*

        *(* focus = NIL *)*

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

        VAR v: View;

    BEGIN

        NEW(v);

        **(* initialize view fields *)**

        **RETURN** v

    END New;

    PROCEDURE **Deposit***;

    BEGIN

        Views.Deposit(New())

    END Deposit;

END NewViews.

