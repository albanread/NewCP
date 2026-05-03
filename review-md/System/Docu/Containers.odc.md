**Containers**

DEFINITION Containers;

    IMPORT Controllers, Stores, Views, Models, Properties;

    CONST

        noSelection = 0; noFocus = 1; noCaret = 2;

        mask = {noSelection, noCaret}; layout = {noFocus};

        deselect = FALSE; select = TRUE;

        any = FALSE; selection = TRUE;

        hide = FALSE; show = TRUE;

    TYPE

        Model = POINTER TO ABSTRACT RECORD (Models.Model)

            (m: Model) GetEmbeddingLimits (OUT minW, maxW, minH, maxH: INTEGER), NEW, ABSTRACT;

            (m: Model) ReplaceView (old, new: Views.View), NEW, ABSTRACT;

            (m: Model) InitFrom- (source: Model), NEW, EMPTY

        END;

        View = POINTER TO ABSTRACT RECORD (Views.View)

            (v: View) CopyFromModelView2- (source: Views.View; model: Models.Model), NEW, EMPTY;

            (v: View) Externalize2- (VAR rd: Stores.Writer), NEW, EMPTY;

            (v: View) HandleCtrlMsg2- (f: Views.Frame; VAR msg: Views.CtrlMessage;

                                                    VAR focus: Views.View), NEW, EMPTY;

            (v: View) HandleModelMsg2- (VAR msg: Models.Message), NEW, EMPTY;

            (v: View) HandlePropMsg2- (VAR p: Views.PropMessage), NEW, EMPTY;

            (v: View) HandleViewMsg2- (f: Views.Frame; VAR msg: Views.Message), NEW, EMPTY;

            (v: View) Internalize2- (VAR rd: Stores.Reader), NEW, EMPTY;

            (v: View) InitModel (m: Model), NEW;

            (v: View) InitModel2- (m: Model), NEW, EMPTY;

            (v: View) AcceptableModel- (m: Model): BOOLEAN, NEW, ABSTRACT;

            (v: View) GetRect (f: Views.Frame; view: Views.View; OUT l, t, r, b: INTEGER), NEW, ABSTRACT;

            (v: View) ThisModel (): Model, EXTENSIBLE;

            (v: View) SetController (c: Controller), NEW;

            (v: View) ThisController (): Controller, NEW, EXTENSIBLE;

            (v: View) GetRect (f: Views.Frame; view: Views.View; OUT l, t, r, b: INTEGER), NEW, ABSTRACT;

            (v: View) CatchModelMsg (VAR msg: Models.Message), NEW, EMPTY;

            (v: View) CatchViewMsg (f: Views.Frame; VAR msg: Views.Message), NEW, EMPTY;

            (v: View) CatchCtrlMsg (f: Views.Frame; VAR msg: Views.CtrlMessage;

                                                                        VAR focus: Views.View), NEW, EMPTY;

            (v: View) CatchPropMsg (VAR msg: Views.PropMessage), NEW, EMPTY

        END;

        Controller = POINTER TO ABSTRACT RECORD (Controllers.Controller)

            opts-: SET;

            (c: Controller) ThisView (): View, EXTENSIBLE;

            (c: Controller) SetOpts (opts: SET), NEW, EXTENSIBLE;

            (c: Controller) GetContextType (OUT type: Stores.TypeName), NEW, ABSTRACT;

            (c: Controller) GetValidOps (OUT valid: SET), NEW, ABSTRACT;

            (c: Controller) NativeModel (m: Models.Model): BOOLEAN, NEW, ABSTRACT;

            (c: Controller) NativeView (v: Views.View): BOOLEAN, NEW, ABSTRACT;

            (c: Controller) NativeCursorAt (f: Views.Frame; x, y: INTEGER): INTEGER, NEW, ABSTRACT;

            (c: Controller) PickNativeProp (f: Views.Frame; x, y: INTEGER;

                                                                                    VAR p: Properties.Property), NEW, EMPTY;

            (c: Controller) PollNativeProp (selection: BOOLEAN; VAR p: Properties.Property;

                                                        VAR truncated: BOOLEAN), NEW, EMPTY;

            (c: Controller) SetNativeProp (selection: BOOLEAN; old, p: Properties.Property), NEW, EMPTY;

            (c: Controller) MakeViewVisible (v: Views.View), NEW, EMPTY;

            (c: Controller) GetFirstView (selection: BOOLEAN; OUT v: Views.View), NEW, ABSTRACT;

            (c: Controller) GetNextView (selection: BOOLEAN; VAR v: Views.View), NEW, ABSTRACT;

            (c: Controller) GetPrevView (selection: BOOLEAN; VAR v: Views.View), NEW, EXTENSIBLE;

            (c: Controller) CanDrop (f: Views.Frame; x, y: INTEGER): BOOLEAN, NEW, EXTENSIBLE;

            (c: Controller) MarkDropTarget (src, dst: Views.Frame; sx, sy, dx, dy, w, h, rx, ry: INTEGER;

                                                        type: Stores.TypeName; isSingle, show: BOOLEAN), NEW, EMPTY;

            (c: Controller) Drop (src, dst: Views.Frame; sx, sy, dx, dy, w, h, rx, ry: INTEGER;

                                                                    view: Views.View; isSingle: BOOLEAN), NEW, ABSTRACT;

            (c: Controller) MarkPickTarget (src, dst: Views.Frame;

                                                                    sx, sy, dx, dy: INTEGER; show: BOOLEAN), NEW, EMPTY;

            (c: Controller) TrackMarks (f: Views.Frame; x, y: INTEGER;

                                                                     units, extend, add: BOOLEAN), NEW, ABSTRACT;

            (c: Controller) Resize (view: Views.View; l, t, r, b: INTEGER), NEW, ABSTRACT;

            (c: Controller) GetSelectionBounds (f: Views.Frame;

                                                                    OUT x, y, w, h: INTEGER), NEW, EXTENSIBLE;

            (c: Controller) DeleteSelection, NEW, ABSTRACT;

            (c: Controller) MoveLocalSelection (src, dst: Views.Frame;

                                                                    sx, sy, dx, dy: INTEGER), NEW, ABSTRACT;

            (c: Controller) CopyLocalSelection (src, dst: Views.Frame;

                                                                    sx, sy, dx, dy: INTEGER), NEW, ABSTRACT;

            (c: Controller) SelectionCopy (): Model, NEW, ABSTRACT;

            (c: Controller) NativePaste (m: Models.Model; f: Views.Frame), NEW, ABSTRACT;

            (c: Controller) ArrowChar (f: Views.Frame; ch: CHAR; units, select: BOOLEAN), NEW, ABSTRACT;

            (c: Controller) ControlChar (f: Views.Frame; ch: CHAR), NEW, ABSTRACT;

            (c: Controller) PasteChar (ch: CHAR), NEW, ABSTRACT;

            (c: Controller) PasteView (f: Views.Frame; v: Views.View; w, h: INTEGER), NEW, ABSTRACT;

            (c: Controller) HasSelection (): BOOLEAN, NEW, EXTENSIBLE;

            (c: Controller) Selectable (): BOOLEAN, NEW, ABSTRACT;

            (c: Controller) Singleton (): Views.View, NEW;

            (c: Controller) SetSingleton (s: Views.View), NEW, EXTENSIBLE;

            (c: Controller) SelectAll (select: BOOLEAN), NEW, ABSTRACT;

            (c: Controller) InSelection (f: Views.Frame; x, y: INTEGER): BOOLEAN, NEW, ABSTRACT;

            (c: Controller) MarkSelection (f: Views.Frame; show: BOOLEAN), NEW, EXTENSIBLE;

            (c: Controller) SetFocus (focus: Views.View), NEW;

            (c: Controller) HasCaret (): BOOLEAN, NEW, ABSTRACT;

            (c: Controller) MarkCaret (f: Views.Frame; show: BOOLEAN), NEW, ABSTRACT;

            (c: Controller) Mark (f: Views.Frame; l, t, r, b: INTEGER; show: BOOLEAN), NEW, EXTENSIBLE

        END;

        Directory = POINTER TO ABSTRACT RECORD

            (d: Directory) New (): Controller, NEW, EXTENSIBLE;

            (d: Directory) NewController (opts: SET): Controller, NEW, ABSTRACT

        END;

        DropPref = RECORD (Properties.Preference)

            mode-: SET;

            okToDrop: BOOLEAN

        END;

        GetOpts = RECORD (Views.PropMessage)

            valid, opts: SET

        END;

        SetOpts = RECORD (Views.PropMessage)

            valid, opts: SET

        END;

    PROCEDURE Focus (): Controller;

    PROCEDURE FocusSingleton (): Views.View;

    PROCEDURE MarkSingleton (c: Controller; f: Views.Frame; show: BOOLEAN);

    PROCEDURE FadeMarks (c: Controller; show: BOOLEAN);

END Containers.

Containers are views which may contain other (embedded) views. Typically, containers belong to one of two categories. The first category contains containers with a fixed structure and fixed types of the embedded views. For example, an e-mail container view may consist of exactly two views: a toolbar view at the top, and a text view below. The layout of these views typically cannot be edited. Such views are called *static containers*.

The second category of containers can embed arbitrary numbers of views, which can be of arbitrary types. Text views and form views are typical examples of such dynamic container views. Only this second category of container views, called *dynamic containers*, is supported by module *Containers*, and it is the topic of this text.

Dynamic containers consist of a variable number of embedded views, plus some *intrinsic contents*. Text views for example have text as their intrinsic contents, but also allow to let arbitrary views (i.e., non-intrinsic contents) flow along in the text. Form views are degenerated in the sense that they provide no intrinsic contents of their own.

Different compound document standards (OLE, OpenDoc) differ in the way they treat selections and the focus. The focus is the currently edited view, the view which receives keyboard events, the view which determines the currently available menus, the view which contains the currently relevant selection, caret, or other mark.

*Containers* provides the building blocks for container views, including special container models and container controllers. Other than most BlackBox modules, *Containers* exports several partially implemented types, rather than pure interface types. What is implemented is provided in a form suitable for the used platform, i.e. user interface differences are hidden by the implementation of *Containers*.

In particular, *Containers* fully implements *singleton selections*, i.e., selections which cover exactly one view and no intrinsic contents. Such selections are subject to special operations and require platform-dependent treatment. Also, *Containers* implements the focus concept: a single embedded view within a controller's model that is picked by the user as current focus.

Example: [<u>Form subsystem  map</u>](../../Form/Docu/Sys-Map.odc.md)

CONST **noSelection**, **noFocus**, **noCaret**

Possible elements of *Controller.opt*. *noSelection* denotes that selections should be switched off, *noFocus* denotes that no embedded view should be allowed to become focus, and *noCaret* denotes that the caret (insertion mark) and thus the possibility to type or paste should be switched off.

CONST **mask**, **layout**

Two particularly useful subsets of *Controller.opt*. A *mask* prevents editing of the container's intrinsic contents, but enables focusing and thereby editing of the contained objects; for example, this allows to use a form without changing the form itself. A *layout* does just the opposite: focusing and therefore editing of contained objects is inhibited, but the container's intrinsic contents may be freely edited; for example, this is useful when editing a form without wanting to actually activate, say, a button, while editing its position in the form.

CONST **deselect**, **select**

Possible values of the *select* parameter of *Controller.SelectAll*.

CONST **any**, **selection**

Possible values of the *selection* parameters of *Controller.GetFirstView*, *Controller.GetNextView*, *Controller.GetPrevView*, *Controller.PollNativeProp*, and *Controller.SetNativeProp*. Controls whether the range of the operation is the current selection or the whole container contents.

CONST **hide**, **show**

Possible values of the *show* parameter of *FadeMarks*, *MarkSingleton*, *Controller.Mark*, *Controller.MarkCaret*, *Controller.MarkDropTarget*, *Controller.MarkPickTarget*, and *Controller.MarkSelections*. Controls whether the respective mark is to be hidden or shown.

TYPE **Model (Models.Model)**

ABSTRACT

Models for containers.

PROCEDURE (m: Model) **GetEmbeddingLimits** (OUT minW, maxW, minH, maxH: INTEGER)

NEW, ABSTRACT

Return minimum (*minW*, *minH*) and maximum (*maxW*, *maxH*) bounds on view sizes to be embedded in model *m*. If it is tried to embed a view into *m* with width < *minW*, width >= *maxW*, height < *minH*, or height >= *maxH*, then *m* should (but is not absolutely required to) modify the size of the embedded view in order to make it fit.

Post

0 <= minW <= maxW

0 <= minH <= maxH

PROCEDURE (m: Model) **ReplaceView** (old, new: Views.View)

NEW, ABSTRACT

In-place substitution of view *old* which must be embedded in *m* by view *new* which must not yet be embedded anywhere. As a result, *new* gets embedded in *m*, but *old* retains its context which it then shares with *new*. Replacing a view in-place allows *wrapping* of views: A new view takes place of an existing one, adds some new properties, but still can hold a reference to the old view and delegate requests to the old view. Since the old view has maintained its context, it will continue to function as if it where directly embedded in *m*.

Pre

old # NIL

old.context.ThisModel() = m

EmbeddedIn(old, m)

new # NIL

new.context = NIL

Post

NotEmbedded(old)

new.context.ThisModel() = m

new.context = old.context

PROCEDURE (m: Model) **InitFrom**- (source: Model)

NEW, EMPTY

Initialize model *m*, which has been newly allocated as an object of the same type as *source*. In some (rare) cases, it may be useful to share some internal data structures between *m* and *source*.

Pre

source # NIL

Type(m) = Type(source)

TYPE **View (Views.View)**

ABSTRACT

Views for containers.

PROCEDURE (v: View) **Internalize** (VAR rd: Stores.Reader)

Clarification of inherited procedure

Fully implements internalization for views without intrinsic persistent state by handling internalization of the container view's model and controller. If the model internalization fails, the view is turned into an alien and internalization of *v* is cancelled; otherwise the model is attached to *v*. If the controller internalization fails, the controller is kept for later externalization to prevent loss of information, but is otherwise not connected to the view (*v.ThisController() = NIL*), and the view is internalized normally (i.e., not turned into an alien).

Super call at the beginning is mandatory

PROCEDURE (v: View) **Externalize** (VAR wr: Stores.Writer)

Clarification of inherited procedure

Fully implements externalization for views without intrinsic persistent state by handling externalization of the container view's model and controller. If *v* has been internalized before with an alien controller, and no other controller has been installed thereafter, then the alien controller will be externalized, although *v.ThisController() = NIL*.

Super call at the beginning is mandatory

Pre

v.ThisModel() # NIL    20

PROCEDURE (v: View) **CopyFrom** (source: Stores.Store)

Clarification of inherited procedure

Assuming that the model of *v* has already been established, copy the controller and possibly other view state from *source*. If *source* holds a hidden alien controller (cf. *Internalize* above), a reference to it is also copied.

Super call at the beginning is mandatory

Pre

v.ThisModel() # NIL    20

Post

source.ThisController() = NIL

    v.ThisController() = NIL

source.ThisController() # NIL

    v.ThisController() = source.ThisController().Clone().CopyFrom(source.ThisController())

PROCEDURE (v: View) **ThisModel** (): Model

EXTENSIBLE

Return the model of *v*.

Covariant narrowing of function result.

PROCEDURE (v: View) **InitModel** (m: Containers.Model)

NEW

Assign a model to this view.

Pre

(v.ThisModel() = NIL) OR (v.ThisModel() = m)    20

m # NIL    21

v.AcceptableModel(m)    22

PROCEDURE (v: View) **InitModel2-** (m: Containers.Model)

NEW, EMPTY

Extension hook called by *InitModel*.

PROCEDURE (v: View) **SetController** (c: Controller)

NEW, EXTENSIBLE, Operation

Set the controller of *v*. If *v* holds a hidden alien controller, it is removed.

Pre

v.ThisModel() # NIL    20

Post

v.ThisController() = c

PROCEDURE (v: View) **ThisController** (): Controller

NEW, EXTENSIBLE

Return the controller of *v*.

PROCEDURE (v: View) **GetRect** (f: Views.Frame; view: Views.View; OUT l, t, r, b: INTEGER)

NEW, ABSTRACT

For display in frame *f*, determine the bounding box of *view* which must be a view contained in *v*. Should the computation of the bounding box be too expensive, returning an approximation is acceptable.

Post

l <= r

t<= b

PROCEDURE (v: View) **CatchModelMsg** (VAR msg: Models.Message)

NEW, EMPTY

Extension hook for *HandleModelMsg* which has become final.

PROCEDURE (v: View) **CatchViewMsg** (f: Views.Frame; VAR msg: Views.Message)

NEW, EMPTY

Extension hook for *HandleViewMsg* which has become final.

PROCEDURE (v: View) **CatchCtrlMsg**

                                     (f: Views.Frame; VAR msg: Controllers.Message; VAR focus: Views.View)

NEW, EMPTY

Extension hook for *HandleCtrlMsg* which has become final.

PROCEDURE (v: View) **CatchPropMsg** (VAR p: Properties.Message)

NEW, EMPTY

Extension hook for *HandlePropMsg* which has become final.

PROCEDURE (v: View) **HandleCtrlMsg** (f: Views.Frame;

    VAR msg: Controllers.Message; VAR focus: Views.View)

Clarification of inherited procedure

If *v* has a controller installed, calls *v*.*ThisController*().*HandleCtrlMsg*(*f*, *msg*,* focus*), then calls *v*.*CatchCtrlMsg*(*f*, *msg*,* focus*). That is, the controller sees the controller message *m*sg before the view does.

    Additionally, a strict filter is applied to throw away unwanted messages: messages are only delegated to the controller or view if they fulfill *one* of the following three criteria:

First, the frame *f* is a target or front frame. Second, the message is derived from *Controllers.PollOpsMsg*, *Controllers.PollCursorMsg*, *Controllers.TransferMessage*, or *Controllers.PageMsg*. Third, the context of *v* is normalizing, i.e., *v*.*context*.*Normalize()* holds, and the message is derived from *Controllers.PollSectionMsg* or *Controllers.ScrollMsg*. This is a standard message filtering condition making components more robust against spurious message sends.

    Finally, scrolling messages (derived from *Controllers.PollSectionMsg* or *Controllers.ScrollMsg*) may ask for shallow handling (~*msg*.*focus*), i.e., supression of forwarding to a possibly existing subfocus. This is generically handled by clearing *focus* on return in these cases.

Post

(msg IS Controllers.PollSectionMsg) OR (msg IS Controllers.ScrollMsg)

    ~msg.focus

        focus = NIL

TYPE **Controller (Controllers.Controller)**

ABSTRACT

Controllers for containers.

**opts**-: SET

Option set of controller; used to restrict controller functionality to defined subsets.

PROCEDURE (c: Controller) **ThisView** (): View

EXTENSIBLE

Return type is narrowed.

PROCEDURE (c: Controller) **SetOpts** (opts: SET)

NEW, EXTENSIBLE, Operation

Set the options of *c*. This is only an operation after a view has been installed, i.e., *c*.*ThisView() # NIL,* otherwise it is a simple assignment of *opts *to *c.opts*. Options 0..7 are used or reserved by BlackBox, the rest may be used by extensions.

Post

c.opts = opts

PROCEDURE (c: Controller) **GetContextType** (OUT type: Stores.TypeName)

NEW, ABSTRACT

Called by *c.HandleCtrlMsg* to fill in *Controllers.PollOpsMsg.type*.

PROCEDURE (c: Controller) **GetValidOps** (OUT valid: SET)

NEW, ABSTRACT

Called by *c.HandleCtrlMsg* to fill in *Controllers.PollOpsMsg.ops*.

PROCEDURE (c: Controller) **NativeModel** (m: Models.Model): BOOLEAN

NEW, ABSTRACT

Check whether *m* is a native model of *c*, i.e., a model that could be attached to a view attachable to *c*.

PROCEDURE (c: Controller) **NativeView** (v: Views.View): BOOLEAN

NEW, ABSTRACT

Check whether *v* is a native view of *c*, i.e., a view that could be attached to *c*.

PROCEDURE (c: Controller) **NativeCursorAt** (f: Views.Frame; x, y: INTEGER): INTEGER

NEW, ABSTRACT

The cursor that *c* would display in *f* at point (*x*, *y*), irrespective of possible embedded views at that position.

PROCEDURE (c: Controller) **PickNativeProp** (f: Views.Frame; x, y: INTEGER;

    VAR p: Properties.Property)

NEW, EMPTY

The properties of *c'*s native contents in *f* at point (*x*, *y*), irrespective of possible embedded views at that position.

PROCEDURE (c: Controller) **PollNativeProp** (selection: BOOLEAN;

    VAR p: Properties.Property; VAR truncated: BOOLEAN);

NEW, EMPTY

The properties of *c'*s native selected or whole contents, irrespective of possible embedded views in that range.

PROCEDURE (c: Controller) **SetNativeProp** (selection: BOOLEAN; old, p: Properties.Property)

NEW, EMPTY

Set the properties of *c'*s native selected or whole contents to *p*, irrespective of possible embedded views in that range. For properties also in *old* change only those property values of *c* that match the ones given in *old*. This allows for masked modification of properties. For example, in a colored model, all red objects could be changed to become green.

PROCEDURE (c: Controller) **MakeViewVisible** (v: Views.View)

NEW, EMPTY

Make the embedded view *v* visible in the controller's view.

Pre

v is embedded in c's view    20

PROCEDURE (c: Controller) **GetFirstView** (selection: BOOLEAN; OUT v: Views.View)

NEW, ABSTRACT

Get the first view embedded in *c*'s model, relative to the model's start or that of a possible selection.

PROCEDURE (c: Controller) **GetNextView** (selection: BOOLEAN; VAR v: Views.View)

NEW, ABSTRACT

Get the next view in the specified range, which is either the selected or the whole contents of c's model. The next view of the last view in the range is *NIL*.

PROCEDURE (c: Controller) **GetPrevView** (selection: BOOLEAN; VAR v: Views.View)

NEW, EXTENSIBLE

Get the previous view in the specified range, which is either the selected or the whole contents of c's model. The default uses *GetFirstView* and *GetNextView* to seek the previous view of *v*. The previous view of the first view in the range is *NIL*.

Pre

v # NIL    20

EmbeddedIn(v, c.ThisModel())    21

PROCEDURE (c: Controller) **CanDrop** (f: Views.Frame; x, y: INTEGER): BOOLEAN

NEW, EXTENSIBLE

Return whether the material being dragged could be dropped into frame *f* at its local coordinate *(x, y)*. The default is to accept any drop request, i.e., to return *TRUE*.

PROCEDURE (c: Controller) **MarkDropTarget** (src, dst: Views.Frame;

                                                                                    sx, sy, dx, dy, w, h, rx, ry: INTEGER;

                                                                                    type: Stores.TypeName;

                                                                                    isSingle, show: BOOLEAN)

NEW, EMPTY

Mark the drop target in destination frame *dst* at point (*dx*, *dy*) for a potential dropping of material from source frame *src*, origin (*sx*, *sy*). *show* determines whether the mark should be drawn or removed. *isSingle* determines whether the selection to be dropped is a singleton. *show* indicates whether the mark should be shown or removed. *(rx, ry)* is the reference point inside the selection, where the drag & drop operation has started. See also *Controllers.PollDropMsg*.

PROCEDURE (c: Controller) **Drop** (src, dst: Views.Frame; sx, sy, dx, dy, w, h, rx, ry: INTEGER;

                                                                view: Views.View; isSingle: BOOLEAN)

NEW, ABSTRACT

Drop the material being dragged from source frame *src*, origin (*sx*, *sy*) and encapsulated in *view* under control of *c* in destination frame *dst* at point (*dx*, *dy*). The default is to ignore the drop. *isSingle* determines whether the selection to be dropped is a singleton. *(rx, ry)* is the reference point inside the selection, where the drag & drop operation has started.  See also *Controllers.DropMsg*.

PROCEDURE (c: Controller) **MarkPickTarget** (src, dst: Views.Frame; sx, sy, dx, dy: INTEGER; show: BOOLEAN)

NEW, EMPTY

Mark the drop target in destination frame *dst* at point (*dx*, *dy*) for a potential dropping of material from source frame *src*, origin (*sx*, *sy*). *show* determines whether the mark should be drawn or removed. The default is not to mark at all.

PROCEDURE (c: Controller) **TrackMarks** (f: Views.Frame; x, y: INTEGER; units, extend, add: BOOLEAN)

NEW, ABSTRACT

Track marks in frame *f* starting at point (*x*, *y*) as specified by *units*, *extend*, and *add*. Marks are general selections and insertion points (carets). Tracking of larger logical units (e.g., words instead of characters) is requested by *units*. Continuous extension of an existing selection is requested by *extend*. Discontinuous addition to an existing selection is requested by *add*.

    Some controllers may ignore one or the other request, e.g., may not distinguish units of varying granularity, may not support multiple selected objects, or may not support discontinuous selections.

PROCEDURE (c: Controller) **Resize** (view: Views.View; l, t, r, b: INTEGER)

NEW, ABSTRACT

Request to resize *view*, which must be embedded in *c*'s model, to the size given by rectangle (*l*, *t*, *r*, *b*).

(Typically, a controller delegates this request to its model which implements the request by using *Properties.PreferredSize* to negotiate the new size with *view*.)

PROCEDURE (c: Controller) **GetSelectionBounds** (f: Views.Frame; OUT x, y, w, h: INTEGER)

NEW, ABSTRACT

Return the bounding box of the selection, by giving its top-left reference point and its width and height. The bounding box is used for giving drag & drop feedback.

PROCEDURE (c: Controller) **DeleteSelection**

NEW, ABSTRACT

Delete all objects in the current selection.

PROCEDURE (c: Controller) **MoveLocalSelection** (src, dst: Views.Frame; sx, sy, dx, dy: INTEGER)

NEW, ABSTRACT

Move selected objects within the model of *c* from the origin given by source frame *src* and point (*sx*, *sy*) to the target given by destination frame *dst* and point (*dx*, *dy*). Since this is a move of material within a single model, there is no need for conversions, and the most "natural" moving semantics can be provided.

PROCEDURE (c: Controller) **CopyLocalSelection** (src, dst: Views.Frame; sx, sy, dx, dy: INTEGER)

NEW, ABSTRACT

Copy selected objects within the model of *c* from the origin given by source frame *src* and point (*sx*, *sy*) to the target given by destination frame *dst* and point (*dx*, *dy*). Since this is a copy of material within a single model, there is no need for conversions, and the most "natural" copying semantics can be provided.

PROCEDURE (c: Controller) **SelectionCopy** (): Model

NEW, ABSTRACT

Return a copy of the selected objects, encapsulated in the returned model.

PROCEDURE (c: Controller) **NativePaste** (m: Models.Model; f: Views.Frame)

NEW, ABSTRACT

Paste a native model into the model of *c* as displayed in frame *f*. Since the model is native, it is to be merged into the model of *c* rather than wrapped into a view and embedded.

PROCEDURE (c: Controller) **ArrowChar** (f: Views.Frame; ch: CHAR; units, select: BOOLEAN)

NEW, ABSTRACT

Interpret the arrow character *ch*, i.e., one out of the following list. The interpretation is to be modified as requested by *units* and *select*. The standard interpretation of arrow characters is the modification of the current selection or the moving of the insertion point. Modifying or moving in larger units (e.g., words instead of characters) is requested by *units*. Establishment of a selection if requested by *select*.

    Some controllers may ignore one or the other request, e.g., may not distinguish units of varying granularity or may not support selections.

Table of arrow characters:



The intention is a modification or move in the indicated direction on the smallest supported granularity ("arrow"), on the basis of "pages" as defined by the size of frame *f*, or on the basis of the whole "document", i.e., the far extremes of the model of *c*. The precise interpretation of the directions and units is left to the specific controller.

PROCEDURE (c: Controller) **ControlChar** (f: Views.Frame; ch: CHAR)

NEW, ABSTRACT

Handle entry of control character *ch* related to the model and view of *c* as displayed in frame *f*.

Table of control characters:



PROCEDURE (c: Controller) **PasteChar** (ch: CHAR)

NEW, ABSTRACT

Paste character *ch* into the model of *c*.

PROCEDURE (c: Controller) **PasteView** (f: Views.Frame; v: Views.View; w, h: INTEGER)

NEW, ABSTRACT

Paste view *v* with desired size (*w*, *h*) into the model of *c*.

(Typically, a controller delegates this request to its model which implements the request by using *Properties.PreferredSize* to negotiate the new size with *v*.)

PROCEDURE (c: Controller) **HasSelection** (): BOOLEAN

NEW, EXTENSIBLE

Return whether the controller currently has a selection. By default, only singleton selections are supported. To be extended to include intrinsic selections.

Pre

c.ThisModel() # NIL    20

PROCEDURE (c: Controller) **Selectable** (): BOOLEAN

NEW, ABSTRACT

Return whether the controller could establish a non-empty selection. If something (or everything) is already selected, this is considered selectable.

PROCEDURE (c: Controller) **Singleton** (): Views.View

NEW

If the controller currently has a singleton selection, then the selected view is returned, else *NIL*.

PROCEDURE (c: Controller) **SetSingleton** (s: Views.View)

NEW, EXTENSIBLE

Set the controller's selection to a singleton selection covering view *s*. IF *s = NIL*, the current singleton selection is cleared. Needs to be extended to adjust intrinsic selection state accordingly.

Pre

c.ThisModel() # NIL    20

~(noSelection IN c.opts)    21

s # NIL

    s.context # NIL    22

    s.context.ThisModel() = c.ThisModel()    23

Post

c.Singleton() = s

PROCEDURE (c: Controller) **SelectAll** (select: BOOLEAN)

NEW, ABSTRACT

Set the selection to its maximum extent, i.e., select all intrinsic content of the controller's model plus all embedded views. For an empty model there is no visible result; for a model with only one embedded view and no intrinsic contents a singleton selection results.

PROCEDURE (c: Controller) **InSelection** (f: Views.Frame; x, y: INTEGER): BOOLEAN

NEW, ABSTRACT

Test whether in frame *f* the point (*x*, *y*) lies within the current selection.

PROCEDURE (c: Controller) **MarkSelection** (f: Views.Frame; show: BOOLEAN)

NEW, EXTENSIBLE

Depending on *show*, show or hide the selection's visual marking. The default handles singleton selections only. To be *replaced* to include intrinsic selections.

PROCEDURE (c: Controller) **SetFocus** (focus: Views.View)

NEW

Sets the current subfocus to *focus*; if *focus = NIL*, the current subfocus is removed.

Pre

c.ThisModel() # NIL    20

focus # NIL

    focus.context # NIL    21

    focus.context.ThisModel() = c.ThisModel()    22

Post

c.ThisFocus() = focus

PROCEDURE (c: Controller) **HasCaret** (): BOOLEAN

NEW, ABSTRACT

Return whether the controller has a valid caret (insertion point).

PROCEDURE (c: Controller) **MarkCaret** (f: Views.Frame; show: BOOLEAN)

NEW, ABSTRACT

Depending on *show*, show or hide the current caret's visual marking.

PROCEDURE (c: Controller) **Mark** (f: Views.Frame; l, t, r, b: INTEGER; show: BOOLEAN)

NEW, EXTENSIBLE

Except for performance, equivalent to:

    MarkFocus(c, f, show); c.MarkSelection(f, show); c.MarkCaret(f, show)

To be extended to cover additional intrinsic marking.

PROCEDURE (c: Controller) **RestoreMarks** (f: Views.Frame; l, t, r, b: INTEGER)

Clarification of inherited procedure

Calls *Mark* to show all marks in (*l*, *t*, *r*, *b*), then, if no subfocus exists and *c.opts* indicates *mask* mode, tries to establish a subfocus: the first embedded view that wants to claim the focus (*Properties.FocusPref.setFocus*) gets it. If the new subfocus wants to start off with an initially selected contents (*Properties.FocusPref.selectOnFocus*), it is also fully selected.

PROCEDURE (c: Controller) **Neutralize**

Clarification of inherited procedure

Remove all modal marks, including focus, selection, and caret. The default handles focus and selection. To be extended to handle caret and possibly further marks.

Except for performance, the default is equivalent to:

    c.SetFocus(NIL); c.SelectAll(deselect)

PROCEDURE (c: Controller) **HandleCtrlMsg** (f: Views.Frame; VAR msg: Controllers.Message;

                            VAR focus: Views.View)

Clarification of inherited procedure

Handles *Controllers.PollCursorMsg*, *Controllers.PollOpsMsg*, *Controllers.TrackMsg*, *Controllers.EditMsg*, *Controllers.TransferMessage*, *Controllers.SelectMsg*, *Controllers.MarkMsg*, *Controllers.ReplaceViewMsg*, *Properties.CollectMsg*, and *Properties.EmitMsg*.

PROCEDURE (c: Controller) **HandlePropMsg** (VAR p: Properties.Message)

Clarification of inherited procedure

The default is to handle *Properties.SetMsg* and *Properties.PollMsg* by splitting the handling of native and embedded properties and set or return the combined property list. Also, unless *noSelection*, *noFocus*, and *noCaret* are set in *c.opts*, field *Properties.FocusPref.setFocus* is set.

Pre

c.ThisModel() # NIL    20

TYPE **Directory**

ABSTRACT

Directory for controllers.

PROCEDURE (d: Directory) **New** (): Controller

NEW, EXTENSIBLE

Except for performance, equivalent to:

    **RETURN** d.NewController({})

PROCEDURE (d: Directory) **NewController** (opts: SET): Controller

NEW, ABSTRACT

Return new controller with options *opts*.

TYPE **DropPref** = RECORD (Properties.Preference)

This message is sent to a contained view in a container when the container needs to decide if something should be dropped into the view or into the container. This message can be intercepted to allow drag and drop also when the container itself normally would catch the drop messages, i.e. when the container is in layout mode.

**mode**-: SET

The current mode of the container.

**okToDrop**: BOOLEAN

Set to TRUE if the view should recieve drop messages also in the mode given by *mode.*

TYPE **GetOpts** = RECORD (Views.PropMessage)

This message is sent to a selected view to ask about the container mode of this view. If a view answers this message it should set *valid* to the options that apply to the view and *opts* to the current values of these options. Whenever GetOpts is answered the SetOpts-message must also be answered.

**valid**: SET

Should be set to the options that apply to the view.

**opts**: SET

Current values of the options pointed out in *valid*.

TYPE **SetOpts** = RECORD (Views.PropMessage)

Sent to a view to indicate that its options should be changed to the values indicated by *valid* and *opts*.

**valid**: SET

Mask to indicate which values should be set.

**opts**: SET

Values of the options indicated by *valid*.

PROCEDURE **Focus** (): Controller

Returns the current focus view's container controller, if possible.

PROCEDURE **FocusSingleton** (): Views.View

Returns the current focus view's singleton selection, if possible.

PROCEDURE **MarkSingleton** (c: Controller; f: Views.Frame; show: BOOLEAN)

Used internally.

PROCEDURE **FadeMarks** (c: Controller; show: BOOLEAN)

Used internally.

