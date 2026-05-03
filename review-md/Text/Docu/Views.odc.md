**TextViews**

DEFINITION TextViews;

    IMPORT Models, Views, Containers, TextModels, TextRulers, TextSetters;

    CONST

        show = FALSE; hide = TRUE;

        any = FALSE; focusOnly = TRUE;

    TYPE

        View = POINTER TO ABSTRACT RECORD (Containers.View)

            (v: View) ThisModel (): TextModels.Model, EXTENSIBLE;

            (v: View) DisplayMarks (hide: BOOLEAN), NEW, ABSTRACT;

            (v: View) HidesMarks (): BOOLEAN, NEW, ABSTRACT;

            (v: View) SetSetter (setter: TextSetters.Setter), NEW, ABSTRACT;

            (v: View) ThisSetter (): TextSetters.Setter, NEW, ABSTRACT;

            (v: View) SetOrigin (org, dy: INTEGER), NEW, ABSTRACT;

            (v: View) PollOrigin (OUT org, dy: INTEGER), NEW, ABSTRACT;

            (v: View) SetDefaults (r: TextRulers.Ruler; a: TextModels.Attributes),

                                NEW, ABSTRACT;

            (v: View) PollDefaults (OUT r: TextRulers.Ruler; OUT a: TextModels.Attributes),

                                NEW, ABSTRACT;

            (v: View) GetThisLocation (f: Views.Frame; pos: INTEGER; OUT loc: Location),

                                NEW, ABSTRACT;

            (v: View) GetRange (f: Views.Frame; OUT beg, end: INTEGER), NEW, ABSTRACT;

            (v: View) GetRect (f: Views.Frame; view: Views.View; OUT l, t, r, b: INTEGER);

            (v: View) ThisPos (f: Views.Frame; x, y: INTEGER): INTEGER, NEW, ABSTRACT;

            (v: View) ShowRangeIn (f: Views.Frame; beg, end: INTEGER), NEW, ABSTRACT;

            (v: View) ShowRange (beg, end: INTEGER; focusOnly: BOOLEAN), NEW, ABSTRACT

        END;

        Directory = POINTER TO RECORD

            defAttr-: TextModels.Attributes;

            (d: Directory) Set (defAttr: TextModels.Attributes), NEW, ABSTRACT;

            (d: Directory) New (text: TextModels.Model): View, NEW, ABSTRACT

        END;

        Location = RECORD

            start, pos: INTEGER;

            x, y: INTEGER;

            asc, dsc: INTEGER;

            view: Views.View;

            l, t, r, b: INTEGER

        END;

        PageMsg = RECORD (Views.PropMessage)

            current: INTEGER

        END;

        PositionMsg = RECORD (Models.Message)

            focusOnly: BOOLEAN;

            beg, end: INTEGER

        END;

    VAR

        ctrlDir-: Containers.Directory;

        dir-, stdDir-: Directory;

    PROCEDURE SetCtrlDir (d: Containers.Directory);

    PROCEDURE SetDir (d: Directory);

    PROCEDURE Focus (): View;

    PROCEDURE FocusText (): TextModels.Model;

    PROCEDURE Deposit;

    PROCEDURE ShowRange (text: TextModels.Model; beg, end: INTEGER; focusOnly: BOOLEAN);

    PROCEDURE ThisRuler (v: View; pos: INTEGER): TextRulers.Ruler;

END TextViews.

*TextViews* are the standard views for text models ([<u>TextModels</u>](Models.odc.md)).

CONST **show**, **hide**

Possible arguments to the *hide* parameter of *view.DisplayMarks*.

CONST **any**, **focusOnly**

Possible arguments to the *focusOnly* parameter of *ShowRange* and *view.ShowRange*.

TYPE **View (Containers.View)**

ABSTRACT

Standard view for text models.

PROCEDURE (v: View) **ThisModel** (): TextModels.Model

EXTENSBIBLE

Result type is narrowed.

PROCEDURE (v: View) **DisplayMarks** (hide: BOOLEAN)

NEW, ABSTRACT, [Operation]

Control hiding of hideable views embedded in the displayed text. If hiding is requested, all views that have a preference *hideable* will be suppressed, i.e. reduced to zero width and zero height. For example, this is used to selectively hide or show rulers in a text.

If *v.context # NIL* and *~v.context.Normalize()*, this is an operation.

Post

v.HidesMarks() = hide

PROCEDURE (v: View) **HidesMarks** (): BOOLEAN

NEW, ABSTRACT

Current status: whether view hides hideable views, or not.

PROCEDURE (v: View) **SetSetter** (setter: TextSetters.Setter)

NEW, ABSTRACT, Operation

Attach a setter to the view.

Pre

setter # NIL    20

Post

v.ThisSetter() = setter

PROCEDURE (v: View) **ThisSetter** (): TextSetters.Setter

NEW, ABSTRACT

Returns setter currently attached to view.

PROCEDURE (v: View) **SetOrigin** (org, dy: INTEGER)

NEW, ABSTRACT, [Operation]

Set the origin and vertical displacement of the view.

If v.context # NIL and ~v.context.Normalize(), this is an operation.

Pre

v.ThisModel() # NIL    20

0 <= org    21

org <= v.ThisModel().Length()    22

dy <= 0    23

Post

org = v.ThisSetter().ThisLine(org)

    v.PollOrigin(o, d): o = org, d = dy

org # v.ThisSetter().ThisLine(org)

    v.PollOrigin(o, d): o = v.ThisSetter().ThisLine(org), d = 0

PROCEDURE (v: View) **PollOrigin** (OUT org, dy: INTEGER)

NEW, ABSTRACT

Return origin and vertical displacement of view. The vertical displacement determines the offset (in units) of the top of the first displayed line, relative to the top of the view area. This is used to support partial display of the first line during scrolling.

Post

0 <= org <= v.ThisModel().Length()

dy <= 0

PROCEDURE (v: View) **SetDefaults** (r: TextRulers.Ruler; a: TextModels.Attributes)

NEW, ABSTRACT, Operation

Set the default ruler and attributes.

Pre

r # NIL    20

a # NIL    22

PROCEDURE (v: View) **PollDefaults** (OUT r: TextRulers.Ruler; OUT a: TextModels.Attributes)

NEW, ABSTRACT

Return default ruler and attributes.

PROCEDURE (v: View) **GetThisLocation** (f: Views.Frame; pos: INTEGER; OUT loc: Location)

NEW, ABSTRACT

Get the location characteristics of the position *pos* displayed in frame *f*. (The frame is required to take decive-dependent character positioning inside words into account.) If *pos* lies outside view, the next best position inside will be taken.

Pre

f # NIL    20

Displayed(f)    21

0 <= pos    22

pos <= v.ThisModel().Length()    23

Post

loc.view # NIL

    loc.l <= loc.r, loc.t <= loc.b    bounding box of view

PROCEDURE (v: View) **GetRect** (f: Views.Frame; view: Views.View; OUT l, t, r, b: INTEGER)

Pre

f # NIL    20

Displayed(f)    21

view.context # NIL    22

view.context.ThisModel() = v.ThisModel()    23

Except for performace, equivalent to:

    VAR loc: Location;

    v.GetThisLocation(f, view.context(TextModels.Context).Pos(), loc);

    IF loc.view = view THEN

        l := loc.l; t := loc.t; r := loc.r; b := loc.b

    ELSE

        l := MAX(INTEGER(; t := MAX(INTEGER); r := l; b := t

    END

PROCEDURE (v: View) **GetRange** (f: Views.Frame; OUT beg, end: INTEGER)

NEW, ABSTRACT

Get the stretch [*beg*, *end*) visible in *v*.

Pre

f # NIL    20

Displayed(f)    21

Post

beg = BeginOf(FirstLineVisible(v))

end = EndOf(LastLineVisible(v))

PROCEDURE (v: View) **ThisPos** (f: Views.Frame; x, y: INTEGER): INTEGER;

NEW, ABSTRACT

Text position corresponding to the point (*x*, *y*) relative to frame *f*.

Pre

f # NIL    20

Displayed(f)    21

Post

v.GetOrigin(o, d): o <= result <= v.ThisModel().Length()

PROCEDURE (v: View) **ShowRangeIn** (f: Views.Frame; beg, end: INTEGER);

NEW, ABSTRACT

If possible, make specified stretch [*beg*, *end*) visible in f.

Pre

f # NIL    20

Displayed(f)    21

Post

If possible, BeginOf(FirstLineVisible(v)) <= k <= EndOf(LastLineVisible(v)):

    beg = end

        k = beg

    beg < end

        beg <= k < end

PROCEDURE (v: View) **ShowRange** (beg, end: INTEGER; focusOnly: BOOLEAN);

NEW, ABSTRACT

Locates a reference frame displaying *v* and performs *v.ShowRangeIn* on it. If *~focusOnly*, an arbitrary frame will be taken, where a target frame takes precedence over all but a front frame, which takes precedence over all other frames. If *focusOnly*, only a front (or if none exists, a target) frame is taken as a reference, and no repositioning takes place if neither a target nor a front frame is found.

TYPE **Directory**

ABSTRACT

Directory for views.

**defAttr**-: TextModels.Attributes

Default attributes, used for initial formatting when typing to an empty text.

PROCEDURE (d: Directory) **Set** (defAttr: TextModels.Attributes)

NEW, EXTENSIBLE

Set the default attributes.

Pre

defAttr # NIL    *not explicitly checked*

defAttr.init    20

Post

d.defAttr = defAttr

PROCEDURE (d: Directory) **New** (text: TextModels.Model): View

NEW, ABSTRACT

Return a new view displaying *text*. If *text* = NIL, a new text is created using *TextModels.dir.New*.

Post

text = NIL

    result.ThisModel() = new-text

text # NIL

    result.ThisModel() = text

TYPE **Location**

Characteristics of an element's (i.e., a character's or embedded view's) location in a text view.

**start**, **pos**: INTEGER

Start of line and position of location.

**x**, **y**: INTEGER

Coordinates of location.

**asc**, **dsc**: INTEGER

Line's ascender and descender at location.

**view**: Views.View

**l**, **t**, **r**, **b**: INTEGER

If embedded view at location: The view and its bounding box.

TYPE  **PageMsg (Views.PropMessage)**

Message send by a text view to all embedded views on a page before rendering that page (for printing). This message can be used by views to display page number dependent behavior during printing.

**current**: INTEGER

Page number of the page to be rendered.

TYPE **PositionMsg (Models.Message)**

Message broadcast by *ShowRange* to request repositioning.

**focusOnly**: BOOLEAN

Reposition front (or, if none, target) view only.

**beg**, **end**: INTEGER

The range requested to become visible.

VAR **ctrlDir**-: Containers.Directory    stable ctrlDir # NIL

Controller directory object. If installed, this directory object is used by the view directory object to install a controller in a newly created view. Upon initialization, module *TextViews* tries to load module *TextControllers* which, if available, in turn installs the standard controller directory object in *ctrlDir*.

VAR **dir**-, **stdDir**-: Directory    dir # NIL, stdDir # NIL, stable stdDir = d

Directory and standard directory objects for views.

PROCEDURE **SetCtrlDir** (d: Containers.Directory)

Set controller directory object.

Pre

d # NIL    20

Post

ctrlDir = d

PROCEDURE **SetDir** (d: Directory)

Set directory object.

Pre

d # NIL    20

Post

dir = d

PROCEDURE **Focus** (): View

Return focus text view, if one exists.

Except for performace, equivalent to:

    VAR v: Views.View;

    v := Controllers.FocusView();

    IF (v # NIL) & (v IS View) THEN **RETURN** v(View) ELSE **RETURN** NIL END

PROCEDURE **FocusText** (): TextModels.Model

Return focus text, if one exists.

Except for performace, equivalent to:

    VAR v: View;

    v := Focus();

    IF v # NIL THEN **RETURN** v.ThisModel() ELSE **RETURN** NIL END

PROCEDURE **Deposit**

Deposit a default text view in the *Views* queue.

Except for performace, equivalent to:

    Views.Deposit(dir.New(NIL))

PROCEDURE **ShowRange** (text: TextModels.Model; beg, end: INTEGER;

                                                focusOnly: BOOLEAN)

For all views *v* displaying *text*, issue *v.ShowRange(beg, end, focusOnly)*.

Pre

text # NIL    20

Except for performace, equivalent to:

    VAR pm: PositionMsg;

    pm.beg := beg; pm.end := end; pm.focusOnly := focusOnly;

    Models.Broadcast(text, pm)

PROCEDURE **ThisRuler** (v: View; pos: INTEGER): TextRulers.Ruler

Locate the ruler dominating position *pos* in the text displayed by *v*.

Pre

v # NIL    20

v.ThisModel() # NIL    21

0 <= pos    22

pos <= v.ThisModel().Length()    23

Except for performace, equivalent to:

    VAR r: TextRulers.Ruler; a: TextModels.Attributes; rpos: INTEGER;

    v.PollDefaults(r, a); rpos := -1; TextRulers.GetValidRuler(v.ThisModel(), pos, -1, r, rpos);

    **RETURN** r

