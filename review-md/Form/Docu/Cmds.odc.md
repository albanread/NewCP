**FormCmds**

DEFINITION FormCmds;

    IMPORT Dialog;

    VAR

        grid: RECORD

            resolution: INTEGER;

            metricSystem: BOOLEAN

        END;

    PROCEDURE AlignLeft;

    PROCEDURE AlignRight;

    PROCEDURE AlignTop;

    PROCEDURE AlignBottom;

    PROCEDURE AlignToRow;

    PROCEDURE AlignToColumn;

    PROCEDURE InitGridDialog;

    PROCEDURE SetGrid;

    PROCEDURE SelectOffGridViews;

    PROCEDURE ForceToGrid;

    PROCEDURE SetAsFirst;

    PROCEDURE SetAsLast;

    PROCEDURE SortViews;

    PROCEDURE InsertAround;

    PROCEDURE FocusGuard (VAR par: Dialog.Par);

    PROCEDURE SelectionGuard (VAR par: Dialog.Par);

    PROCEDURE SingletonGuard (VAR par: Dialog.Par);

END FormCmds.

Command package for form views. Its main purpose is to support layout editing, through various alignment and grid control commands.

A possible menu using the above commands:

**MENU** "Layout" ("FormViews.View")

    "Align &Left"    ""    "FormCmds.AlignLeft"    "FormCmds.SelectionGuard"

    "Align &Right"    ""    "FormCmds.AlignRight"    "FormCmds.SelectionGuard"

    "Align &Top"    ""    "FormCmds.AlignTop"    "FormCmds.SelectionGuard"

    "Align &Bottom"    ""    "FormCmds.AlignBottom"    "FormCmds.SelectionGuard"

    "Align To Ro&w"    ""    "FormCmds.AlignToRow"    "FormCmds.SelectionGuard"

    "Align To &Column"    ""    "FormCmds.AlignToColumn"    "FormCmds.SelectionGuard"

    **SEPARATOR**

    "Set &Grid..."    ""    "FormCmds.InitGridDialog;

            StdCmds.OpenToolDialog('Form/Rsrc/Cmds', 'Set Grid')"

                "FormCmds.FocusGuard"

    "&Select Off-Grid Views"    ""    "FormCmds.SelectOffGridViews"    ""

    "&Force To Grid"    ""    "FormCmds.ForceToGrid"    "FormCmds.SelectionGuard"

    **SEPARATOR**

    "Set F&irst/Back"    ""    "FormCmds.SetAsFirst"    "FormCmds.SingletonGuard"

    "Set L&ast/Front"    ""    "FormCmds.SetAsLast"    "FormCmds.SingletonGuard"

    "Sort &Views"    ""    "FormCmds.SortViews"    "FormCmds.FocusGuard"

    **SEPARATOR**

    "Insert Group Box"    ""    "FormCmds.InsertAround"    "FormCmds.FocusGuard"

END

VAR **grid**: RECORD

Interactor for the grid dialog box (Form/Rsrc/Cmds).

**resolution**: INTEGER    resolution > 0

If *metricSystem*, then *resolution* specifies how many grid positions exist for one millimeter.

If *~metricSystem*, then *resolution* specifies how many grid positions exist for 1/16 inch.

A higher value means that higher precision is possible.

**metricSystem**: BOOLEAN

Determines whether the metric system (millimeters) is used or not (1/16 inches).

PROCEDURE **AlignLeft**

Guard: FormCmds.SelectionGuard

Move all selected views such that their left sides are aligned to the leftmost view in the selection.

PROCEDURE **AlignRight**

Guard: FormCmds.SelectionGuard

Move all selected views such that their right sides are aligned to the rightmost view in the selection.

PROCEDURE **AlignTop**

Guard: FormCmds.SelectionGuard

Move all selected views such that their top sides are aligned to the topmost view in the selection.

PROCEDURE **AlignBottom**

Guard: FormCmds.SelectionGuard

Move all selected views such that their bottom sides are aligned to the bottommost view in the selection.

PROCEDURE **AlignToRow**

Guard: FormCmds.SelectionGuard

Move all selected views such that their vertical centers become aligned horizontally.

PROCEDURE **AlignToColumn**

Guard: FormCmds.SelectionGuard

Move all selected views such that their horizontal centers become aligned vertically.

PROCEDURE **InitGridDialog**

Guard: FormCmds.FocusGuard

Sets up *grid.resolution* and *grid.metricSystem* according to the values of the focus controller.

PROCEDURE **SetGrid**

Guard: FormCmds.FocusGuard

Sets the focus view's *grid* and *gridFactor* to the values determined by *grid.resolution* and *grid.metricSystem*.

PROCEDURE **SelectOffGridViews**

Guard: FormCmds.FocusGuard

Selects all views in the focus form whose top-left corners don't lie on the grid.

PROCEDURE **ForceToGrid**

Guard: FormCmds.FocusGuard

Moves all views in the focus form such that their top-left corners come to lie on the grid.

PROCEDURE **SetAsFirst**

Guard: FormCmds.SingletonGuard

Sets the selected view to the first position ("bottom").

PROCEDURE **SetAsLast**

Guard: FormCmds.SingletonGuard

Sets the selected view to the last position ("top").

PROCEDURE **SortViews**

Guard: FormCmds.FocusGuard

Sorts the back-to-front order of all views in a form such that they are geometrically sorted, i.e., a view whose upper edge is further up, or at the same hight but further to the left, is considered to come before ("lower") than the other one.

PROCEDURE **InsertAround**

Guard: FormCmds.SelectionGuard

Inserts a group box around the currently selected views.

PROCEDURE **FocusGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current front focus isn't a form view.

PROCEDURE **SelectionGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current front focus isn't a form view, or if it doesn't contain a selection.

PROCEDURE **SingletonGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current front focus isn't a form view, or if it doesn't contain a singleton.

