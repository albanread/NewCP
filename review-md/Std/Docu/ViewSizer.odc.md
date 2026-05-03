**StdViewSizer**

DEFINITION StdViewSizer;

    IMPORT Dialog;

    VAR

        size: RECORD

            typeName-: Dialog.String;

            w, h: REAL

        END;

    PROCEDURE InitDialog;

    PROCEDURE SetViewSize;

    PROCEDURE SizeGuard (VAR par: Dialog.Par);

    PROCEDURE UnitGuard (VAR par: Dialog.Par);

END StdViewSizer.

*StdViewSizer* is a command package allowing the user to resize views embedded in containers by specifying width and height as numbers. This is useful whenever dragging view borders with the mouse is not precise enough. The command package works on any singleton view.

For entry in a menu, the following line is recommended:

"View Size..."    ""    "StdViewSizer.InitDialog; StdCmds.OpenToolDialog('Std/Rsrc/ViewSizer', 'View Size')"    "StdCmds.SingletonGuard"

VAR **size**: RECORD

Interactor for setting the view size.

**typeName**-: Dialog.String

Type of the current singleton view.

**w**: INTEGER

View width in cm or inch, depending on the value of *Dialog.metricSystem*.

**h**: INTEGER

View height in cm or inch, depending on the value of *Dialog.metricSystem*.

PROCEDURE **InitDialog**

Initialization command for *size* interactor.

PROCEDURE **SetViewSize**

Applies the interactor values in *size*. To set up *size* to the selected view, *InitDialog* needs to be called first.

PROCEDURE **SizeGuard** (VAR par: Dialog.Par)

Guard which ensures that the *size* interactor matches the current singleton selection. If there is no singleton selection, the guard disables.

PROCEDURE **UnitGuard** (VAR par: Dialog.Par)

Guard that sets *par.label* to "cm" or "inch" respectively, depending on the value of Dialog.metricSystem.

