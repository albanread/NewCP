**FormViews**

DEFINITION FormViews;

    IMPORT Ports, Views, Controllers, Containers, FormModels;

    CONST minBorder = 4 * Ports.point; maxBorder = 100 * Ports.mm;

    TYPE

        View = POINTER TO ABSTRACT RECORD (Containers.View)

            (v: View) ThisModel (): FormModels.Model, EXTENSIBLE;

            (v: View) SetBorder (border: INTEGER), NEW, ABSTRACT;

            (v: View) Border (): INTEGER, NEW, ABSTRACT;

            (v: View) SetGrid (grid, gridFactor: INTEGER), NEW, ABSTRACT;

            (v: View) Grid (): INTEGER, NEW, ABSTRACT;

            (v: View) GridFactor (): INTEGER, NEW, ABSTRACT;

            (v: View) SetBackground (background: Ports.Color), NEW, ABSTRACT

        END;

        Directory = POINTER TO ABSTRACT RECORD

            (d: Directory) New (f: FormModels.Model): View, NEW, ABSTRACT

        END;

    VAR

        dir-, stdDir-: Directory;

        ctrldir-: Controllers.Directory;

    PROCEDURE Focus (): View;

    PROCEDURE FocusModel (): FormModels.Model;

    PROCEDURE RoundToGrid (v: View; VAR x, y: INTEGER);

    PROCEDURE New (): View;

    PROCEDURE Deposit;

    PROCEDURE SetDir (d: Directory);

    PROCEDURE SetCtrlDir (d: Containers.Directory);

END FormViews.

*FormViews* are the standard views on *FormModels*.

CONST **minBorder, maxBorder**

The border of a form view is the minimal distance between any of the view borders and the bounding box of the embedded views. The preferred border can be set to a value in the range *[minBorder .. maxBorder]*.

TYPE **View (Views.View)**

ABSTRACT

PROCEDURE (v: View) **ThisModel** (): FormModels.Model

EXTENSIBLE

Result type is narrowed.

PROCEDURE (v: View) **SetBorder** (border: INTEGER)

NEW, ABSTRACT, Operation

Sets the view's preferred border (preferred minimal distance between any view edge and the closest embedded view).

Pre

border >= 0    20

Post

border < minBoder

    v.border = minBorder

border > maxBorder

    v.border = maxBorder

minBorder <= border <= maxBorder

    v.border = border

PROCEDURE (v: View) **Border** (): INTEGER

NEW, ABSTRACT

Returns the view's border.

Post

minBorder <= result <= maxBorder

PROCEDURE (v: View) **SetGrid** (grid, gridFactor: INTEGER)

NEW, ABSTRACT, Operation

Sets the view's preferred grid (preferred grid on which any embedded view's top-left corner should be aligned) and grid factor (when the grid is shown, every *gridFactor*-th grid unit a dotted line is displayed).

Pre

grid > 0    20

gridFactor > 0    21

Post

v.Grid() = grid  &  v.GridFactor() = gridFactor

PROCEDURE (v: View) **Grid** (): INTEGER

NEW, ABSTRACT

Returns the current grid.

Post

result > 0

PROCEDURE (v: View) **GridFactor** (): INTEGER

NEW, ABSTRACT

Returns the current grid factor.

Post

result > 0

PROCEDURE (v: View) **SetBackground** (background: Ports.Color)

NEW, ABSTRACT

Sets a form's background color. Default is *Ports.dialogBackground*.

TYPE **Directory**

ABSTRACT

Directory for form views.

PROCEDURE (d: Directory) **New** (m: FormModels.Model): View

Interface

Return a new view on *m*

Pre

m # NIL    20

Post

result # NIL

result.ThisModel() = m

VAR **dir**, **stdDir**-: Directory    dir # NIL  &  stdDir # NIL

Directory and standard directory for form views.

VAR **ctrldir**-: Controllers.Directory    ctrldir # NIL

Form controller directory, installed by module *FormControllers* upon loading.

PROCEDURE **Focus** (): View

Returns the focus form view, if it is one.

PROCEDURE **FocusModel** (): FormModels.Model

Returns the model of the focus form view, if it is one.

PROCEDURE **RoundToGrid** (v: View; VAR x, y: INTEGER)

Rounds the coordinate *(x, y)* to the closest point on *v*'s grid.

Pre

v # NIL    20

x > 0  &  y > 0    21

Post

x MOD v.grid = 0

y MOD v.grid = 0

PROCEDURE **New** (): View

Returns a new form view with a new empty form model, i.e., returns *FormViews.dir.New(FormModels.dir.New()).*

PROCEDURE **Deposit**

*Deposit* creates a new form view with a new empty form model and deposits the view.

*Deposit* is called internally.

PROCEDURE **SetDir** (d: Directory)

Assigns view directory.

Pre

d # NIL    20

Post

dir = d

PROCEDURE **SetCtrlDir** (d: Containers.Directory)

Assigns the controller directory for form views.

Pre

d # NIL    20

