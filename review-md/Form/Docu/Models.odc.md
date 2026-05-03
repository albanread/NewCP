**FormModels**

DEFINITION FormModels;

    IMPORT Ports, Models, Views, Containers;

    CONST minView Size = 4 * Ports.point; maxViewSize = 1000 * Ports.mm;

    TYPE

        Model = POINTER TO ABSTRACT RECORD (Containers.Model)

            (m: Model) Insert (v: Views.View; l, t, r, b: INTEGER), NEW, ABSTRACT;

            (m: Model) Delete (v: Views.View), NEW, ABSTRACT;

            (m: Model) Resize (v: Views.View; l, t, r, b: INTEGER), NEW, ABSTRACT;

            (m: Model) PutAbove (v, pos: Views.View), NEW, ABSTRACT;

            (m: Model) Move (v: Views.View; dx, dy: INTEGER), NEW, ABSTRACT;

            (m: Model) Copy (VAR v: Views.View; dx, dy: INTEGER), NEW, ABSTRACT;

            (m: Model) NewReader (old: Reader): Reader, NEW, ABSTRACT;

            (m: Model) NewWriter (old: Writer): Writer, NEW, ABSTRACT;

            (m: Model) ViewAt (x, y: INTEGER): Views.View, NEW, ABSTRACT;

            (m: Model) NofViews (): INTEGER, NEW, ABSTRACT

        END;

        Directory = POINTER TO ABSTRACT RECORD

            (d: Directory) New (): Model, NEW, ABSTRACT

        END;

        Context = POINTER TO ABSTRACT RECORD (Models.Context)

            (c: Context) ThisModel (): Model, ABSTRACT;

            (c: Context) GetRect (OUT l, t, r, b: INTEGER), NEW, ABSTRACT

        END;

        Reader = POINTER TO ABSTRACT RECORD

            view: Views.View;

            l, t, r, b: INTEGER;

            (r: Reader) Set (pos: Views.View), NEW, ABSTRACT;

            (r: Reader) ReadView (OUT v: Views.View), NEW, ABSTRACT

        END;

        Writer = POINTER TO ABSTRACT RECORD

            (w: Writer) Set (pos: Views.View), NEW, ABSTRACT;

            (w: Writer) WriteView (v: Views.View; l, t, r, b: INTEGER), NEW, ABSTRACT

        END;

        UpdateMsg = RECORD (Models.UpdateMsg)

            l, t, r, b: INTEGER

        END;

    VAR dir-, stdDir-: Directory;

    PROCEDURE New (): Model;

    PROCEDURE CloneOf (source: Model): Model;

    PROCEDURE Copy (source: Model): Model;

    PROCEDURE SetDir (d: Directory);

END FormModels.

*FormModels* are container models which contain views. They have no further intrinsic contents. Form models can be used to arrange rectangular views in arbitrary layouts. Their main use is as data entry forms and as dialog box layouts.

CONST **minViewSize**

This is the minimal width and height of a view which is embedded in a form model.

CONST **maxViewSize**

This is the maximal width and height of a view which is embedded in a form model.

TYPE **Model (Containers.Model)**

ABSTRACT

Form models are container models (-> Containers), which may contain rectangular views and nothing else.

PROCEDURE (m: Model) **Insert** (v: Views.View; l, t, r, b: INTEGER)

NEW, ABSTRACT, Operation

Insert view *v* with bounding box *(l, t, r, b)*.

Pre

v # NIL    20

v.context = NIL    22

l <= r    23

t <= b    24

Post

v in m

v.context # NIL & v.context.ThisModel() = m

PROCEDURE (m: Model) **Delete** (v: Views.View)

NEW, ABSTRACT, Operation

Remove *v* from *m*.

Pre

v in m    20

Post

~(v in m)

PROCEDURE (m: Model) **Resize** (v: Views.View; l, t, r, b: INTEGER)

NEW, ABSTRACT, Operation

Redefine bounding box of *v*.

Pre

v in m    20

l <= r    21

t <= b    22

PROCEDURE (m: Model) **PutAbove** (v, pos: Views.View)

NEW, ABSTRACT, Operation

Change the vertical order of view *v*, such that it comes to lie directly above *p* if *pos # NIL*, otherwise it is put to the bottom of the view list.

Pre

v in m    20

pos = NIL  OR  pos in m    21

PROCEDURE (m: Model) **Move** (v: Views.View; dx, dy: INTEGER)

NEW, ABSTRACT, Operation

Move view *v* by *(dx, dy)*, without changing its size.

Pre

v in m    20

PROCEDURE (m: Model) **Copy** (VAR v: Views.View; dx, dy: INTEGER)

NEW, ABSTRACT, Operation

Create a copy of *v* and put it at *v*'s bounding box shifted by *(dx, dy)*. Parameter *v* returns the copy.

Pre

v # NIL    20

v.context # NIL    21

v.context.ThisModel() = m    22

Post

v # NIL  &  v # v'

PROCEDURE (m: Model) **NewReader** (old: Reader): Reader

NEW, ABSTRACT

Returns a reader connected to *m*. An old reader may be passed as input parameter, if it isn't in use anymore.

Post

result # NIL

PROCEDURE (m: Model) **NewWriter** (old: Writer): Writer

NEW, ABSTRACT

Returns a writer connected to *m*. An old writer may be passed as input parameter, if it isn't in use anymore.

Post

result # NIL

PROCEDURE (m: Model) **ViewAt** (x, y: INTEGER): Views.View

NEW, ABSTRACT

Returns the topmost view in *m* which encloses position *(x, y)*.

Post

result # NIL

    where (l, t, r, b) is the bounding box of v: (l <= x <= r) & (t <= y <= b)

result = NIL

    no view at (x, y)



PROCEDURE (m: Model) **NofViews** (): INTEGER

NEW, ABSTRACT

Returns the number of views currently in *m*.

Post

result >= 0

TYPE **Directory**

ABSTRACT

Directory for the allocation of concrete form models.

PROCEDURE (d: Directory) **New** (): Model

NEW, ABSTRACT

Create and return a new concrete form model.

Post

result # NIL

TYPE **Context (Models.Context)**

NEW, ABSTRACT

Context of a view in a form.

PROCEDURE (c: Context) **ThisModel** (): Model

ABSTRACT

Returns the form which contains the context. Covariant narrowing of result type.

PROCEDURE (c: Context) **GetRect** (OUT l, t, r, b: INTEGER)

NEW, ABSTRACT

Returns the bounding box of the context's view.

Post

l < r  &  t < b

TYPE **Reader**

ABSTRACT

Input rider on a form model.

**view**: Views.View

Most recently read view.

**l, t, r, b**: INTEGER    view # NIL => l < r & t < b

Bounding box of most recently read view.

PROCEDURE (r: Reader) **Set** (pos: Views.View)

NEW, ABSTRACT

Set position of reader *r* to the view above *pos *(i.e., the next view to be read will be the one directly above *pos*) or to the bottom if *pos = NIL*.

Pre

pos in Base(r)  OR  pos = NIL    20

PROCEDURE (r: Reader) **ReadView** (OUT v: Views.View)

NEW, ABSTRACT

Reads the next view, in ascending order. If there is none, *v* is set to *NIL*. The reader's *view* and *l, t, r, b* fields will be set accordingly (*l, t, r, b* are undefined if view is* NIL*).

Post

v = r.view

TYPE **Writer**

ABSTRACT

Output rider on a form.

PROCEDURE (w: Writer) **Set** (pos: Views.View)

Interface

Set position of writer *w* to the view above *pos *(i.e., the next view to be written will be inserted directly above *pos*) or to the bottom if *pos = NIL*.

Pre

pos in Base(r)  OR  pos = NIL    20

PROCEDURE (w: Writer) **WriteView** (v: Views.View; l, t, r, b: INTEGER)

NEW, ABSTRACT

Insert view *v* at the current position in *w*'s form.

Pre

v # NIL    20

v.context = NIL    22

l <= r    23

t <= b    24

Post

v.context # NIL

TYPE **UpdateMsg**

This message indicates that a rectangular part of a form needs to be updated on the screen.

The receiver must not switch on any marks as a reaction to having received this message.

UpdateMsgs are sent by concrete form model implementations after any view modifications.

UpdateMsgs are not extended.

VAR **dir**-, **stdDir**-: Directory;    dir # NIL  &  stdDir # NIL

Form model directories.

PROCEDURE **New** (): Model

Returns a new model. Equivalent to *dir.New()*.

Post

result # NIL

PROCEDURE **CloneOf** (source: Model): Model

Returns a new empty model of the same type as *source*.

Pre

source # NIL    20

Post

result # NIL

PROCEDURE **Copy** (source: Model): Model

Returns a new model of the same type as *source*, with a copy of the contents of *source*.

Pre

source # NIL    20

Post

result # NIL

PROCEDURE **SetDir** (d: Directory)

Assign directory.

Pre

d # NIL    20

Post

dir = d

