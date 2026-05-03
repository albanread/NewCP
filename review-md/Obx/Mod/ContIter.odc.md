MODULE ObxContIter;

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

    IMPORT Views, Containers, Controls;

    PROCEDURE **Do***;

    (** focus the first control whose label is "magic name" **)

        VAR c: Containers.Controller;v: Views.View;

    BEGIN

        c := Containers.Focus();

        IF c # NIL THEN

            c.GetFirstView(Containers.any, v);

            WHILE (v # NIL) & ~((v IS Controls.Control) & (v(Controls.Control).label = "magic name")) DO

                c.GetNextView(Containers.any, v)

            END;

            IF v # NIL THEN

                c.SetFocus(v)

            END

        END

    END Do;

END ObxContIter.

