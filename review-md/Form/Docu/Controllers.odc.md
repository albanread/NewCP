**FormControllers**

DEFINITION FormControllers;

    IMPORT Views, Controllers, Containers, FormModels, FormViews;

    CONST noSelection = Containers.noSelection; noFocus = Containers.noFocus;

    TYPE

        Controller = POINTER TO ABSTRACT RECORD (Containers.Controller)

            form-: FormModels.Model;

            view-: FormViews.View;

            (c: Controller) ThisView (): FormViews.View, EXTENSIBLE;

            (c: Controller) Select (view: Views.View), NEW, ABSTRACT;

            (c: Controller) Deselect (view: Views.View), NEW, ABSTRACT;

            (c: Controller) IsSelected (view: Views.View): BOOLEAN, NEW, ABSTRACT;

            (c: Controller) GetSelection (): List, NEW, ABSTRACT;

            (c: Controller) SetSelection (l: List), NEW, ABSTRACT

        END;

        Directory = POINTER TO ABSTRACT RECORD (Controllers.Directory)

            (d: Directory) New (): Controller, EXTENSIBLE;

            (d: Directory) NewController (opts: SET): Controller, ABSTRACT

        END;

        List = POINTER TO RECORD

            next: List;

            view: Views.View

        END;

    VAR dir-, stdDir-: Directory;

    PROCEDURE Focus (): Controller;

    PROCEDURE Insert (c: Controller; view: Views.View; l, t, r, b: INTEGER);

    PROCEDURE SetDir (d: Directory);

    PROCEDURE Install;

END FormControllers.

*FormControllers* are standard controllers for *FormViews*. Note that forms can only be used in a non-modal way, i.e., a program doesn't wait until the user is finished with the form. In other words: the user is in control, not the computer.

TYPE **Controller (Containers.Controller)**

ABSTRACT

Standard controllers for form views.

**form**-: FormModels.Model    form # NIL

The controller's model.

**view**-: FormViews.View    view # NIL & view.ThisModel() = form

The controller's view.

PROCEDURE (c: Controller) **ThisView** (): FormViews.View

EXTENSIBLE

Covariant narrowing of function result.

PROCEDURE (c: Controller) **Select** (view: Views.View)

NEW, ABSTRACT

Adds a view to the current selection, if it isn't selected already.

Pre

view in c.form    20

~(noSel IN c.opts)    21

Post

c.IsSelected(view)

c.ThisFocus() = NIL

PROCEDURE (c: Controller) **Deselect** (view: Views.View)

NEW, ABSTRACT

Removes a view from the current selection, if it is selected.

Pre

view in c.form    20

Post

~c.IsSelected(view)

PROCEDURE (c: Controller) **IsSelected** (view: Views.View): BOOLEAN

NEW, ABSTRACT

Determines whether the given view is currently selected or not. *NIL* is not considered selected.

Pre

view = NIL  OR  view in c.form    20

PROCEDURE (c: Controller) **GetSelection** (): List

NEW, ABSTRACT

Returns the list of selected subviews.

Post

all views of the result list are in c.form

PROCEDURE (c: Controller) **SetSelection** (l: List)

NEW, ABSTRACT

Removes the existing selection, and selects the subviews of *l*.

Pre

all views of l are in c.form    20

TYPE **Directory**

ABSTRACT

Directory for form view controllers.

PROCEDURE (d: Directory) **New** (): Controller

EXTENSIBLE

Covariant extension of *Controllers.Directory.New*.

PROCEDURE (d: Directory) **NewController** (opts: SET): Controller

ABSTRACT

Covariant extension of *Controllers.Directory.NewController*.

VAR **dir**-, **stdDir**-: Directory    dir # NIL  &  stdDir # NIL

Controller directories.

PROCEDURE **Focus** (): Controller

Returns the focus controller, if the focus currently is a form view, otherwise it returns *NIL*.

PROCEDURE **Insert** (c: Controller; view: Views.View; l, t, r, b: INTEGER)

Inserts *view* into *c*'s view at the position *(l, t, r, b)*. If necessary, the position is slightly corrected (rounded) such that *view*'s top-left corner comes to lie on the grid. The size of *view* is not changed, however.

PROCEDURE **SetDir** (d: Directory)

Set directory *d*.

Pre

d # NIL    20

Post

dir = d

PROCEDURE **Install**

Used internally.

