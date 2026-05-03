MODULE NewCmds;

*(*  *)*

    IMPORT Views, FormModels, FormViews, FormControllers;

    *(** sample command **)*

    PROCEDURE **Do***;

    *(** guard: FormCmds.FocusGuard **)*

        VAR c: FormControllers.Controller; f: FormModels.Model; rd: FormModels.Reader; v: Views.View;

    BEGIN

        c := FormControllers.Focus();    *(* get focus controller, if there is a focus view and if this view is a form view *)*

        IF c # NIL THEN

            IF c.HasSelection() THEN

                f := c.form;    *(* get the controller's form model *)*

                rd := f.NewReader(NIL);    *(* set up new reader at beginning of form *)*

                rd.ReadView(v);

                WHILE v # NIL DO    *(* iterate over all views in form *)*

                    *(* do something with *v* *)*

                    rd.ReadView(v)

                END

            END

        END

    END Do;

END NewCmds.
