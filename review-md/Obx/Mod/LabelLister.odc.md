MODULE ObxLabelLister;

*(***

*    project    = "BlackBox"*

*    organization    = "www.oberon.ch"*

*    contributors    = "Oberon microsystems"*

*    version    = "[**<u>System/Rsrc/About</u>*](StdCmds.OpenToolDialog('System/Rsrc/About', 'About BlackBox'))*"*

*    copyright    = "[**<u>System/Rsrc/About</u>*](StdCmds.OpenToolDialog('System/Rsrc/About', 'About BlackBox'))*"*

*    license    = "[**<u>Docu/BB-License</u>*](../../Docu/BB-License.odc.md)*"*

*    changes    = ""*

*    issues    = ""*

***)*

    IMPORT Views, Controls, FormModels, FormControllers, StdLog;

    PROCEDURE **List***;

        VAR c: FormControllers.Controller; rd: FormModels.Reader; v: Views.View;

    BEGIN

        c := FormControllers.Focus();

        IF c # NIL THEN

            rd := c.form.NewReader(NIL);

            rd.ReadView(v);

            WHILE v # NIL DO

                IF v IS Controls.Control THEN

                    StdLog.String(v(Controls.Control).label); StdLog.Ln

                END;

                rd.ReadView(v)

            END

        END

    END List;

END ObxLabelLister.

