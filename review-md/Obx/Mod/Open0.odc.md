MODULE ObxOpen0;

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

    IMPORT Files, Converters, Views, TextModels, TextMappers, TextViews;

    PROCEDURE **Do***;

        VAR loc: Files.Locator; name: Files.Name; conv: Converters.Converter;

            v: Views.View; t: TextModels.Model; f: TextMappers.Formatter;

    BEGIN

        loc := NIL; name := ""; conv := NIL;    *(* no defaults for Views.Old *)*

        v := Views.Old(Views.ask, loc, name, conv);    *(* ask user for a file and open it as a view *)*

        IF (v # NIL) & (v IS TextViews.View) THEN    *    (* make sure it is a text view *)*

            t := v(TextViews.View).ThisModel();    *(* get the text view's model  *)*

            f.ConnectTo(t);

            f.SetPos(t.Length());    *(* set the formatter to the end of the text *)*

            f.WriteString("appendix");    *(* append a string *)*

            Views.OpenView(v)    *(* open the text view in a window *)*

        END

    END Do;

END ObxOpen0.
