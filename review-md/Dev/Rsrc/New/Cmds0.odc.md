MODULE NewCmds;

*(*  *)*

    IMPORT TextModels, TextViews, TextControllers;

    *(** sample command **)*

    PROCEDURE **Do***;

    *(** guard: TextCmds.FocusGuard **)*

        VAR c: TextControllers.Controller; t: TextModels.Model; rd: TextModels.Reader; ch: CHAR;

    BEGIN

        c := TextControllers.Focus();    *(* get focus controller, if there is a focus view and if this view is a text view *)*

        IF c # NIL THEN

            IF c.HasSelection() THEN

                t := c.text;    *(* get the controller's text model *)*

                rd := t.NewReader(NIL);    *(* set up new reader at beginning of text *)*

                rd.ReadChar(ch);

                WHILE ~rd.eot DO    *(* iterate over all characters in text *)*

                    *(* do something with *ch* *)*

                    rd.ReadChar(ch)

                END

            END

        END

    END Do;

END NewCmds.
