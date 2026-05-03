MODULE NewCmds;

*(*  *)*

    IMPORT Dialog, NewViews;

    *(** miscellaneous **)*

    PROCEDURE **Do***;

    *(** guard: NewCmds.FocusGuard **)*

        VAR v: NewViews.View;

    BEGIN

        v := NewViews.Focus();

        IF v # NIL THEN

            **(* do something with the focused view *)**

        END

    END Do;

    *(** standard guard **)*

    PROCEDURE **FocusGuard*** (VAR par: Dialog.Par);

    *(** in non-NewViews menus; otherwise implied by menu type **)*

    BEGIN

        par.disabled := NewViews.Focus() = NIL

    END FocusGuard;

END NewCmds.

