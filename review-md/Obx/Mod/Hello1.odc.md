MODULE ObxHello1;

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

    IMPORT Views, TextModels, TextMappers, TextViews;

    PROCEDURE **Do***;

        VAR t: TextModels.Model; f: TextMappers.Formatter; v: TextViews.View;

    BEGIN

        t := TextModels.dir.New();    *(* create a new, empty text object *)*

        f.ConnectTo(t);    *(* connect a formatter to the text *)*

        f.WriteString("Hello World"); f.WriteLn;    *(* write a string and a 0DX into new text *) *

        v := TextViews.dir.New(t);    *(* create a new text view for t *)*

        Views.OpenView(v)    *(* open the view in a window *)*

    END Do;

END ObxHello1.

