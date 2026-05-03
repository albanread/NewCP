MODULE ObxViews0;

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

    IMPORT Views, Ports;

    TYPE View = POINTER TO RECORD (Views.View) END;

    PROCEDURE (v: View)  Restore (f: Views.Frame; l, t, r, b: INTEGER);

    BEGIN

        f.DrawRect(l, t, r, b, Ports.fill, Ports.red)

    END Restore;

    PROCEDURE Deposit*;

        VAR v: View;

    BEGIN

        NEW(v); Views.Deposit(v)

    END Deposit;

END ObxViews0.

