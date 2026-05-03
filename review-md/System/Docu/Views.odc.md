**Views**

DEFINITION Views;

    IMPORT Files, Fonts, Stores, Ports, Converters, Models;

    CONST

        undefined = 0;

        transparent = 0FF000000H;

        deep = FALSE; shallow = TRUE;

        keepFrames = FALSE; rebuildFrames = TRUE;

        dontAsk = FALSE; ask = TRUE;

        clean = 0; notUndoable = 1; invisible = 2;

    TYPE

        View = POINTER TO ABSTRACT RECORD (Stores.Store)

            context-: Models.Context;

            (v: View) InitContext (context: Models.Context), NEW, EXTENSIBLE;

            (v: View) GetBackground (VAR color: Ports.Color), NEW, EMPTY;

            (v: View) Neutralize, NEW, EMPTY;

            (v: View) ConsiderFocusRequestBy- (view: View), NEW, EMPTY;

            (v: View) GetNewFrame (VAR frame: Frame), NEW, EMPTY;

            (v: View) Restore (f: Frame; l, t, r, b: INTEGER), NEW, ABSTRACT;

            (v: View) RestoreMarks (f: Frame; l, t, r, b: INTEGER), NEW, EMPTY;

            (v: View) HandleViewMsg- (f: Frame; VAR msg: Message), NEW, EMPTY;

            (v: View) HandleCtrlMsg (f: Frame; VAR msg: CtrlMessage; VAR focus: View), NEW, EMPTY;

            (v: View) HandlePropMsg- (VAR p: PropMessage), NEW, EMPTY;

            (v: View) HandleModelMsg- (VAR msg: Models.Message), NEW, EMPTY;

            (v: View) CopyFrom- (source: View);

            (v: View) CopyFromSimpleView- (source: View), NEW, EMPTY;

            (v: View) CopyFromModelView- (source: View; model: Models.Model), NEW, EMPTY;

            (v: View) ThisModel (): Models.Model, NEW, EXTENSIBLE

        END;

        Alien = POINTER TO LIMITED RECORD (View)

            store-: Stores.Alien

        END;

        Message = ABSTRACT RECORD

            view-: View

        END;

        NotifyMsg = EXTENSIBLE RECORD (Message)

            id0, id1: INTEGER;

            opts: SET

        END;

        Frame = POINTER TO ABSTRACT RECORD (Ports.Frame)

            l-, t-, r-, b-: INTEGER;

            view-: View;

            front-, mark-: BOOLEAN;

            (f: Frame) Close, NEW, EMPTY

        END;

        RootFrame = POINTER TO RECORD (Frame)

            flags-: SET

        END;

        PropMessage = ABSTRACT RECORD END;

        CtrlMessage = ABSTRACT RECORD END;

        CtrlMsgHandler = PROCEDURE (op: INTEGER; f, g: Frame; VAR msg: CtrlMessage;

                                                            VAR mark, front, req: BOOLEAN);

        Title = ARRAY 64 OF CHAR;

        UpdateCachesMsg = EXTENSIBLE RECORD (Message) END;

        ScrollClassMsg = RECORD (Message)

            allowBitmapScrolling: BOOLEAN

        END;

    VAR HandleCtrlMsg-: CtrlMsgHandler;

    PROCEDURE Broadcast (v: View; VAR msg: Message);

    PROCEDURE Domaincast (domain: Stores.Domain; VAR msg: Message);

    PROCEDURE Omnicast (VAR msg: ANYREC);

    PROCEDURE HandlePropMsg (v: View; VAR msg: PropMessage);

    PROCEDURE Era (v: View): INTEGER;

    PROCEDURE BeginModification (type: INTEGER; v: View);

    PROCEDURE EndModification (type: INTEGER; v: View);

    PROCEDURE BeginScript (v: View; name: Stores.OpName; OUT script: Stores.Operation);

    PROCEDURE EndScript (v: View; script: Stores.Operation);

    PROCEDURE Do (v: View; name: Stores.OpName; op: Stores.Operation);

    PROCEDURE LastOp (v: View): Stores.Operation;

    PROCEDURE Bunch (v: View);

    PROCEDURE StopBunching (v: View);

    PROCEDURE ForwardCtrlMsg (f: Frame; VAR msg: CtrlMessage);

    PROCEDURE Update (v: View; rebuild: BOOLEAN);

    PROCEDURE UpdateIn (v: View; l, t, r, b: INTEGER; rebuild: BOOLEAN);

    PROCEDURE ReadView (VAR rd: Stores.Reader; OUT v: View);

    PROCEDURE WriteView (VAR wr: Stores.Writer; v: View);

    PROCEDURE CopyOf (v: View; shallow: BOOLEAN): View;

    PROCEDURE CopyWithNewModel (v: View; m: Models.Model): View;

    PROCEDURE ReadFont (VAR rd: Stores.Reader; OUT f: Fonts.Font);

    PROCEDURE WriteFont (VAR wr: Stores.Writer; f: Fonts.Font);

    PROCEDURE IsPrinterFrame (f: Frame): BOOLEAN;

    PROCEDURE InstallFrame (host: Frame; view: View; x, y, level: INTEGER; focus: BOOLEAN);

    PROCEDURE ThisFrame (host: Frame; view: View): Frame;

    PROCEDURE FrameAt (host: Frame, x, y: INTEGER): Frame;

    PROCEDURE Old (ask: BOOLEAN; VAR loc: Files.Locator; VAR name: Files.Name;

                                    VAR conv: Converters.Converter): View;

    PROCEDURE OldView (loc: Files.Locator; name: Files.Name): View;

    PROCEDURE Register (view: View; ask: BOOLEAN; VAR loc: Files.Locator; VAR name: Files.Name;

                                        VAR conv: Converters.Converter; OUT res: INTEGER);

    PROCEDURE RegisterView (view: View; loc: Files.Locator; name: Files.Name);

    PROCEDURE Open (view: View; loc: Files.Locator; name: Files.Name; conv: Converters.Converter);

    PROCEDURE OpenView (view: View);

    PROCEDURE OpenAux (view: View; title: Title);

    PROCEDURE Deposit (view: View);

    PROCEDURE RestoreDomain (domain: Stores.Domain);

    PROCEDURE Scroll (v: View; dx, dy: INTEGER);

    PROCEDURE SetDir (d: Directory);

    PROCEDURE MarkBorders (root: RootFrame);

    PROCEDURE MarkBorder (host: Frame; v: View; l, t, r, b: INTEGER);

    PROCEDURE Fetch (OUT view: View);

    PROCEDURE Available (): INTEGER;

    PROCEDURE ClearQueue;

    PROCEDURE RemoveFrame (host, f: Frame);

    PROCEDURE RemoveFrames (host: Frame; l, t, r, b: INTEGER);

    PROCEDURE BroadcastModelMsg (f: Frame; VAR msg: Models.Message);

    PROCEDURE BroadcastViewMsg (f: Frame; VAR msg: Message);

    PROCEDURE HandleCtrlMsg (op: INTEGER; f, g: Frame; VAR msg: CtrlMessage;

                                                    VAR target, front: BOOLEAN);

    PROCEDURE SetRoot (root: RootFrame; view: View; front: BOOLEAN; flags: SET);

    PROCEDURE AdaptRoot (root: RootFrame);

    PROCEDURE RootOf (f: Frame): RootFrame;

    PROCEDURE HostOf (f: Frame): Frame;

    PROCEDURE UpdateRoot (root: RootFrame; l, t, r, b: INTEGER; rebuild: BOOLEAN);

    PROCEDURE RestoreRoot (root: RootFrame; l, t, r, b: INTEGER);

    PROCEDURE ValidateRoot (root: RootFrame);

    PROCEDURE InitCtrl (p: CtrlMsgHandler);

    PROCEDURE IsInvalid (v: View): BOOLEAN;

    PROCEDURE RevalidateView (v: View);

END Views.

Figure 1. Model-View-Controller Separation

A *view* is a rectangular display object which provides visual presentation of data. Views are storable, and may be embedded recursively.

A view often contains a *Models.Model* which represents some data, and sometimes a *Controllers.Controller* which provides interaction of the view with the user. There may be several views for each model simultaneously, but at most one controller per view.

If several views share the same model, every change of the model must cause all its views to update their contents accordingly. Module *Models* provides a messaging mechanism through which visible views can be notified of model modifications, and thus re-establish the display's consistency.

It is possible to implement views which do not contain a model. These views cannot use the messaging mechanism of module *Models*. Therefore, such views usually don't share data and are independent from each other. Typically, these views are simple *controls* which implement a very specific functionality that relies on cooperation with the control's container, e.g., a form view container.

It is also possible to implement views which do not contain a controller. This is possible because all messages to a controller are sent to the controller's view, not directly to the controller itself. Thus a view can decide whether to handle these messages itself, or whether to forward them to a controller. Simple views don't contain a controller.

Because a view is an extension of a *Stores.Store*, it can be embedded in a model, such that it is externalized and internalized as part of this model. This makes it possible to realize compound documents, which contain views containing views containing views...

When a view needs to draw to the screen (or printer), it can do this through a *frame*. A frame is an access path (a mapper) to the port on which the view is presented. Since several windows may show the same document with its hierarchy of views, one and the same view may be visible several times simultaneously. This results in several frames for the same view simultaneously, and therefore in the need to update a view change in several frames.

In general, the reaction to a model modification happens in two steps. In the first step, the nature of the model's change is broadcast to all visible views, using the model broadcast mechanism of module *Models*. This causes every visible view to update its own state, if necessary. In the second step, every view which has changed its state uses the view broadcast mechanism of module *Views*, to notify all frames for this view. In fact, since frames are usually not extended, the view itself performs the update for each of its frames. This second broadcast step is normally invisible to the programmer, because he merely needs to determine the view's region which needs updating. The actual update of this region in each of its frames is done by the framework.

A view may behave differently depending on the model in which it is embedded, i.e., depending on its context. For this purpose, a variable of type *Models.Context* is carried by a view, as a link to its container.

For every user interaction, e.g., the press of a key, a view must be defined which should handle this interaction, i.e., a so-called *focus*. BlackBox doesn't know which view is the current focus. BlackBox only provides a strategy which decides which window is focus. Since a window may contain a hierarchy of views, a view which has received an interaction message - a controller message - must decide on its own whether it is the focus itself, or whether it contains another view which might be focus instead. In the former case, it handles the messages itself, in the latter case, it forwards the message to this view.

Every window contains a tree of frames. This tree corresponds to the visible views of the window. Every view may only draw inside its own borders, drawing outside of its borders must be prevented. Frames provide the necessary clipping facility. The management of the frame tree and of clipping is largely transparent to the view programmer.

Examples:

[<u>ObxPatterns  docu</u>](../../Obx/Docu/Patterns.odc.md)    views without models

[<u>ObxCalc  docu</u>](../../Obx/Docu/Calc.odc.md)

[<u>ObxOmosi  docu</u>](../../Obx/Docu/Omosi.odc.md)

[<u>ObxButtons  docu</u>](../../Obx/Docu/Buttons.odc.md)

[<u>ObxLines  docu</u>](../../Obx/Docu/Lines.odc.md)    views with models

[<u>ObxGraphs  docu</u>](../../Obx/Docu/Graphs.odc.md)

[<u>ObxBlackBox  docu</u>](../../Obx/Docu/BlackBox.odc.md)

[<u>ObxWrappers  docu</u>](../../Obx/Docu/Wrappers.odc.md)    wrapper

[<u>ObxTwins  docu</u>](../../Obx/Docu/Twins.odc.md)    special container

[<u>Form subsystem  map</u>](../../Form/Docu/Sys-Map.odc.md)    general container

CONST **undefined**

This value can be used to denote the width or height of a view as currently undefined.

CONST **transparent**

A view may be asked for its background color. In this case, the view may either return a *Ports.Color* value, or the value *transparent*. The latter value means that the view's container must find another source for a background color, i.e., for the color which is used to erase the background before the foreground is restored (use *Update *or *UpdateIn* to restore a view's area in all its frames). Transparency is useful if several views are superimposed on each other, which naturally occurs in a compound document.

CONST **deep, shallow**

There are two ways that a view can be copied: *deep* or *shallow*. This distinction arises from the fact that a view can carry two types of data: data that it owns completely, and data that it can share with other views. The most important example of the latter is the view's model, if it has one.

When a view is copied, it must be decided whether shareable state should actually be shared with the copy (shallow copy), or whether an independent copy of this state should be created (deep copy). These constants can be passed to a parameter of the *CopyModel* or the *CopyOf *procedure.

CONST **keepFrames, rebuildFrames**

When part of a view's area must be restored (in every frame on it), there are two possible kinds of restoration: a frame may be kept as it is and only its contents be redrawn, or it may be rebuilt, i.e., newly allocated, set up, and redrawn. The latter is less efficient than the former, and only necessary if the following holds: the view must be a container, and the operation which changed the view might have modified a subview's bounding box (and thus invalidated its subframes). In the rare case where frames are extended, it could sometimes also become necessary to rebuild the frames on a view.

CONST **dontAsk**, **ask**

Theses constants may be passed to the *Old* and *Register* procedures. They determine whether these procedures allow the user to interactively change *loc*, *name*, or *conv* used for the operation.

CONST **clean**

Possible value for parameter *type* of *BeginModification/EndModification*. Indicates an operation that does not make its document "dirty". Example: modifying a text in a way that is considered as "unimportant", such as collapsing or expanding a text fold (-> StdFolds).

CONST **invisible**

Possible value for parameter *type* of *BeginModification/EndModification*. Indicates an operation that "folds together" with the previous operation, i.e., does not itself become visible in the Undo/Redo menu items. Invisible operations can be used for operations that by themselves may not be expected to appear in an Undo/Redo menu. Example: setting options in a controller. When executing a Redo operation, after a (visible) operation, all invisible operations are executed. When executing an Undo operation, first all invisible operations are undone and afterwards the visible operations.

CONST **notUndoable**

Possible value for parameter *type* of *BeginModification/EndModification*. Indicates an operation that cannot be reversed ("undone"). This is important for operations where the undo feature would be too expensive.

TYPE **View (Stores.Store)**

ABSTRACT

A view is a storable object which may contain a model, possibly maintains a scroll position in this model, and generates frames for its display when needed. A view can be regarded as a special editor, and advanced views (containers) are able to contain arbitrary other views as part of their editable data (i.e., of their model).

Views are allocated by specific view directories, e.g., *TextViews.dir*.

Views are used by commands which manipulate the visual presentation of data.

Views are extended for new kinds of data to be presented visually. Besides the implementation of new commands, the implementation of new view extensions is the central activity of BlackBox programming.

*Restore* is the only procedure which necessarily must be implemented in an extension of *View*. It is called by the framework when the view must be redrawn on a screen or on a printer.

*Internalize*, *Externalize* must be implemented in views which contain persistent mutable data. In this case, a view without model should also implement the *CopyFromSimpleView* procedure, while a view with a model should implement the *CopyFromModelView* procedure instead. *Internalize* / *Externalize* is called by the framework when the user opens / saves a document. *CopyFromSimpleView* / *CopyFromModelView* should also be implemented by views with mutable state that should be printable. The reason is that the framework makes a shallow copy of a view that is being printed, in order to avoid the original view to be changed by pagination, scrolling, or similar modifications that may be performed during printing.

*ThisModel* must be implemented in views which contain models. It is called by the framework to find out whether this view should receive model messages for this model.

*ConsiderFocusRequestBy* should be implemented in container views. It can be called by an embedded view that wants to become focus itself.

*GetBackground* must be implemented in views which have non-transparent background colors. It is called by the framework as part of the restore mechanism.

*RestoreMarks* and *Neutralize* must be implemented in views which may contain marks like selections or carets. *RestoreMarks* is called by the framework as part of the restore mechanism, after its corresponding *Restore* has been called.

*HandleModelMsg* must be implemented in views which support partial view updates after a model change. It is called by the framework in order to deliver notifications about changes to the view's model.

*HandleViewMsg* must be implemented in views which support marks, or which don't use the delayed update mechanism for some other reason. It is called by the framework to deliver view messages via all currently visible paths (i.e., via frames) to this view.

*HandleCtrlMsg* must be implemented in editable views. It is called by the surrounding container if it considers the receiving view to be the current focus.

*HandlePropMsg* must be implemented in views which support preferences and properties (-> Properties). It is called by the surrounding container to find out about the way that the embedded view would like to be treated.

*InitContext* and *GetNewFrame* are usually not extended.

A view's domain is the same as the domain of its container and of all embedded views and models. The domain identifies the document in which the view, its context, and its contents are embedded. The view's *PropagateDomain* procedure propagates the assigned domain to its model, if there is one.

**context**-: Models.Context

The view's context links the view to its container. Communication between view and container occurs via the context. A context belongs (and is managed by) the container, but carried by the view.

PROCEDURE (v: View) **InitContext** (context: Models.Context)

NEW, EXTENSIBLE

Assigns *context* to *v.context*.

*InitContext* is called by *v*'s container, when v is being embedded in it (which means that the container creates a suitable context for *v*).

*InitContext* is usually not extended, only view wrappers need to extend it in order to forward a context.

Pre

context # NIL    21

v.context = NIL OR v.context = context    22

Post

v.context = context

PROCEDURE (v: View) **GetBackground** (VAR color: Ports.Color)

NEW, EMPTY

This procedure may return a background color of the view. Upon input, *color = transparent*.

*GetBackground* is called internally.

*GetBackground* is implemented if a view needs a non-transparent background color.

PROCEDURE (v: View) **Neutralize**

NEW, EMPTY

This procedure should remove all marks that a view carries.

*Neutralize* is called by the framework.

*Neutralize* is implemented by views which may contain marks.

PROCEDURE (v: View) **ConsiderFocusRequestBy-** (view: View)

NEW, EMPTY

A subview of *v* may request to become focus. Its container may or may not grant this request.

*ConsiderFocusRequestBy* is called by a subview.

*ConsiderFocusRequestBy* is implemented in a container view.

PROCEDURE (v: View) **GetNewFrame** (VAR f: Frame)

NEW, EMPTY

The procedure may generate a frame for the view. Upon entry, *f = NIL*.

This procedure is rarely implemented (mainly in native controls).

*GetNewFrame* is called internally.

*GetNewFrame* is implemented in views which need specialized extended view frames.

PROCEDURE (v: View) **Restore** (f: Frame; l, t, r, b: INTEGER)

NEW, ABSTRACT

A view implementation must implement this procedure, to draw all or part of its contents. For drawing, a frame is passed to the view, whose drawing procedures can be called in the *Restore* procedure.

Only the rectangle *(l, t, r, b)*, which is given in universal coordinates, needs to be restored. Since drawing is clipped to this rectangle automatically, it is sometimes the best solution to simply restore the whole view's contents. However, often it is significantly faster to restore only the contents of the rectangle.

If necessary, the size of the view can be determined by calling *v.context.GetSize*. Consistent with the frame drawing operations (as described in the documentation of module *Ports*), the origin for drawing is the view's top-left corner, with positive x-values to the right, and positive y-values to the bottom.

For drawing at screen pixel resolutions without rounding errors, the frame's *f.dot* field is useful. The value *f.unit* can be inspected to obtain the size of a pixel in universal coordinates (see the documentation of module *Fonts*). For example, this allows to adapt to the different resolutions during screen display and during printing.

If drawing should be done in different ways depending on whether the view is being displayed on screen or whether it is being printed, then *Views.IsPrinterFrame(f)* can be used to determine whether it is being printed or displayed on screen.

*Restore* is called internally by the framework, whenever some part of the view becomes newly visible, or after someone has called *Views.Update* or *Views.UpdateIn* for this view. *Restore* is rarely called directly by a view itself. No assumptions are allowed of when *Restore* is called, how often, in which order, etc. *Restore* should simply draw everything within the given rectangle, it must not assume that something is still on screen from the last time it was called.

Since views may be nested, a container must be drawn before the views contained in it are draw. The framework calls *Restore* methods in the correct order, from the "back" to the "front". An embedded view thus lies always "above" its container. As an exception of the back-to-front drawing rule, BlackBox allows to draw marks (in particular: selections) of a container *on top* of the contained views. This can be achieved by moving the restoration of marks to the view's *RestoreMarks* method. This is only necessary for view containers, however.

*Restore* must be implemented in every view extension. It is the only view procedure whose implementation is mandatory, not optional.

Pre

f # NIL    20

f.view = v    21

v.context # NIL    22

0 <= f.l <= l <= r <= f.r <= width of view    23

0 <= f.t <= t <= b <= f.b <= height of view    24

PROCEDURE (v: View) **RestoreMarks** (f: Frame; l, t, r, b: INTEGER)

NEW, EMPTY

Restore all marks (in particular any selection) of view *v* via frame *f*. Only the rectangle *(l, t, r, b)*, which is given in universal coordinates, needs to be restored. A frame contains the method *MarkRect* which is provided particularly for the drawing of marks.

*RestoreMarks* is called locally by the framework as part of the restore mechanism, after its corresponding *Restore* has been called.

*RestoreMarks* is implemented in views which support any kind of marks, e.g., selection, caret, or focus marks.

Pre

f # NIL    20

f.view = v    21

v.context # NIL    22

PROCEDURE (v: View) **HandleViewMsg**- (f: Frame; VAR msg: Message)

NEW, EMPTY

Message handler for view messages.

*HandleViewMsg* is called locally.

*HandleViewMsg* is implemented in views which support marks (e.g., selection marks), and in views which support different frame contents for the same view (a rare case).

Pre

f # NIL    guaranteed

f.view = v    guaranteed

v.context # NIL    guaranteed

msg.view = v  OR  msg.view = NIL    guaranteed

PROCEDURE (v: View) **HandleCtrlMsg** (f: Frame; VAR msg: CtrlMessage; VAR focus: View)

NEW, EMPTY

Message handler for messages to the focus.

*HandleCtrlMsg* is called by the view's container, indirectly via *ForwardCtrlMsg*. If a controller message needs to be forwarded to an embedded view, set *focus* to this view. The framework will perform forwarding after *HandleCtrlMsg* returns. In the rare cases where this is not an adequate solution, *ForwardCtrlMsg *must be used; the embedded view's *HandleCtrlMsg* must never be called directly. After *ForwardCtrlMsg*, *focus* must be set to *NIL*, so that the message is not forwarded twice.

During mouse tracking (i.e., when handling a *Controllers.TrackMsg*), drawing should only occur in frame *f*. This also implies that during mouse tracking, no update messages should be sent. If necessary, an update model message should be sent after the mouse button was released.

*HandleCtrlMsg* is extended in editable views.

Pre

f # NIL    20

f.view = v    21

v.context # NIL    22

focus = NIL    23

PROCEDURE (v: View) **HandlePropMsg**- (VAR p: PropMessage)

NEW, EMPTY

Property messages can be passed to a view via its *HandlePropMsg* procedure.

*HandlePropMsg* is called by the view's container. The global procedure *HandlePropMsg* (see further below) is used to send a property message to a view.

*HandlePropMsg* is called locally.

*HandlePropMsg* is implemented in views which support properties (not described here).

PROCEDURE (v: View) **HandleModelMsg**- (VAR msg: Models.Message)

NEW, EMPTY

Message handler for model messages.

*HandleModelMsg* is called locally.

*HandleModelMsg* is implemented in views with a model which support updates after a model modification.

Pre

msg.model # NIL    20

msg.model = v.ThisModel()    21

PROCEDURE (v: View) **CopyFrom**- (source: Stores.Store)

This method has become final. It calls the *CopyFromSimpleView* or *CopyFromModelView*, respectively. It checks that these procedures don't change the view context.

PROCEDURE (v: View) **CopyFromSimpleView**- (source: View)

NEW, EMPTY

The procedure should be implemented in views which have no model. It should copy view-specific data from *source*. *CopyFromModelView* is called as part of module *Views*' copy operations (*CopyOf*, *CopyWithNewModel*).

Note: it is not permissible to implement both *CopyFromModelView* and *CopyFromSimpleView* simultaneously!

Pre

source # NIL    guaranteed

TYP(source) = TYP(v)    guaranteed

CopyFromModelView must not be implemented    20

PROCEDURE (v: View) **CopyFromModelView**- (source: View; model: Models.Model)

NEW, EMPTY

The procedure must be implemented in views which have a model, and only in them. The major exception where a view without model may still implement *CopyFromModelView* instead of *CopyFromSimpleView* are wrapper views: using *CopyFromModelView* they can be implemented flexibly enough to wrap arbitrary views, whether they have models or not.

The procedure should initialize its model to *model*. If necessary, it can copy view-specific data from *source*. *CopyFromModelView* is called as part of module *Views*' copy operations (*CopyOf*, *CopyWithNewModel*).

Note that if *model = source.ThisModel()*, then a shallow copy is being performed.

Note: it is not permissible to implement both *CopyFromModelView* and *CopyFromSimpleView* simultaneously!

Pre

source # NIL    guaranteed

TYP(source) = TYP(v)    guaranteed

model # NIL  =>  TYP(model) = TYP(source.ThisModel())    guaranteed

CopyFromSimpleView must not be implemented    20

Post

v.ThisModel() = model

PROCEDURE (v: View) **ThisModel** (): Models.Model

NEW, EXTENSIBLE

Returns the view's model, if it has one. The default implementation returns *NIL*.

*ThisModel* is called internally.

*ThisModel* is replaced by views which contain models. A view with a model must always return the same model, i.e., the one which was assigned to it upon initialization.

TYPE **Alien (View)**

LIMITED

If the internalization of a view fails, either because its implementing module(s) cannot be loaded, or because it cancelled internalization (e.g., because of a version conflict), an alien is produced instead (this happens in procedure *ReadView*). An alien is immutable and doesn't contain a model. It contains an alien store which can be inspected to determine the type of the alien, and the cause for it to be an alien.

Every container must be able to operate even if one or several of its embedded views are aliens. If the view's model is an alien store, the view may turn itself into an alien.

Aliens are allocated in *ReadView*.

**store**-: Stores.Alien    store # NIL

The alien store which has been generated by a *Stores.Reader* during internalization of the view.

TYPE **Message**

ABSTRACT

Base type of all view messages. Such messages are sent when a view's state has changed, in order to render the display consistent again. There may be several frames displaying the same view, such that every one of them needs to be updated.

Messages are sent by views when their states have changed and if they cannot use BlackBox's delayed update mechanism. This is true mainly when drawing marks, e.g., selection marks.

Messages are extended to indicate what kind of update should be performed on a frame.

**view**-: View

The view which has changed. If *view = NIL*, all frames with the same domain are notified of the view change.

TYPE **NotifyMsg (Message)**

EXTENSIBLE

This message notifies all visible views about a change in an interactor's state (-> Dialog).

*NotifyMsg* is sent by the interactor procedures *Update* and *UpdateList* in module *Dialog*.

*NotifyMsg* is never extended.

*NotifyMsg* is sent only internally.

**id0, id1**: INTEGER

Identification of the interactor or of one of its fields.

**opts**: SET

Determines whether controls (not described here) should check their guards, for example.

TYPE **Frame (Ports.Frame)**

ABSTRACT

All input and output operations of a view pass through a frame. A frame manages the whole layout of views on a port, including clipping. Model and view messages are broadcast along frame trees. A frame tree's internal structure is hidden. Frames are volatile objects, they are allocated and released by BlackBox whenever necessary, e.g., when the frame's window is resized. Thus they cannot be used to carry application-specific state, except for caches.

Frames are allocated by a view's *GetNewFrame* procedure.

Frames are managed internally, and passed as parameters to view procedures whenever necessary.

Standard frames are sufficient for most purposes, and thus rarely extended.

**l-, t-, r-, b-**: INTEGER    0 <= l <= r  &  0 <= t <= b

The visible area of the view in this frame. The values are in universal coordinates, relative to the frame's view's top-left corner.

For example, *f.l + f.gx* is the distance of the left frame border from the left port (display, printer) border in universal coordinates; *(f.l + f.gx) DIV f.unit* is the same distance in pixels.

**view**-: View    view # NIL

The frame's view.

**front-**: BOOLEAN

Flag which tells whether the frame is part of the front window

**mark-**: BOOLEAN

Flag which tells whether the frame is on its window's focus path, i.e., whether marks (caret, selection) should be drawn. Typically, marking procedures work the following way:

    IF f.mark THEN

        IF f.front THEN DrawMark(f) ELSE DrawBackgroundMark(f) END

    END

PROCEDURE (f: Frame) **Close**

NEW, EMPTY

Perform finalization before the frame is removed.

After a call to *Close*, *f.view* and *f.rider* are set to *NIL* and *f.ConnectTo(NIL)* is called.

*Close* is called internally.

TYPE **RootFrame**

This type is used internally.

**flags**-: SET

Window-specific flags. Reserved for future use.

TYPE **PropMessage**

ABSTRACT

Use its alias *Properties.Message* instead (properties are not described here).

TYPE **CtrlMessage**

ABSTRACT

Base type of all controller messages. Use its alias *Controllers.Message* instead.

TYPE **CtrlMsgHandler**

Used internally.

TYPE **Title**

Type for view titles, e.g., in windows.

TYPE **UpdateCachesMsg (Message)**

EXTENSIBLE

Used internally.

TYPE **ScrollClassMsg (Message)**

EXTENSIBLE

Used internally.

PROCEDURE **Broadcast** (v: View; VAR msg: Message)

Broadcast *msg* for *view*. Before broadcasting, parameter *v* is assigned to the message's *view*-Field. The actual broadcast only takes place if *v.domain # NIL*.

Broadcast is called by a view whenever its state has changed and the delayed restore mechanism is not sufficient, e.g., to update frame-specific marks. *v* will receive *msg* once for every visible frame on itself.

The handler of a view message may not recursively broadcast another view message, since this could cause the messages to be received in another order than they have been sent, which would result in errors very hard to find.

Pre

v # NIL    20

no recursion    21

Post

msg.view = v

PROCEDURE **Domaincast** (domain: Stores.Domain; VAR msg: Message)

Broadcasts *msg* inside *domain*. For visual objects, the domain corresponds to the object's document. *Domaincast* is only necessary in exceptional cases; usually a view message is sent in places where the corresponding view, and not only its domain, is known. In these cases, *Broadcast* is the appropriate (and faster) procedure.

PROCEDURE **Omnicast** (VAR msg: Message)

Broadcast *msg* to all open views, independent of their domain (i.e., of their document). All views will receive this message with *msg.view = NIL*. *Omnicast* is slower than *Broadcast*, and only necessary in exceptional cases, e.g., for clock views which should be updated every second through a message omnicast.

PROCEDURE **HandlePropMsg** (v: View; VAR msg: PropMessage)

Use this procedure to send a property message to a view. It is equivalent to

    v.HandlePropMsg(msg)

except that a much better error handling is performed.

PROCEDURE **Era** (v: View): INTEGER

For views with models, returns the era in which the view was last synchronized with the model.

Pre

v # NIL    20

Post

v.ThisModel() # NIL

    in-synch(v) iff Era(v) = Models.Era(v.ThisModel())

PROCEDURE **BeginModification** (type: INTEGER; v: View)

PROCEDURE **EndModification** (type: INTEGER; v: View)

PROCEDURE **BeginScript** (v: View; name: Stores.OpName; OUT script: Stores.Operation)

PROCEDURE **EndScript** (v: View; script: Stores.Operation)

PROCEDURE **Do** (v: View; name: Stores.OpName; op: Stores.Operation)

PROCEDURE **LastOp** (v: View): Stores.Operation

PROCEDURE **Bunch** (v: View)

PROCEDURE **StopBunching** (v: View)

These procedures handle modifications of a view. They are used in the same way as their model counterparts in module *Models* (-> Models). Note that these view procedures are provided for convenience, they are mostly identical to the *Models*. The only noticeable difference is that a view operation only affects this view, and doesn't affect other views even if they show the same model. For example, a script that modifies a view's model should use the *Models* procedure, so that all views are updated correctly. A script that modifies only a view's private state should use the procedures above.

PROCEDURE **ForwardCtrlMsg** (f: Frame; VAR msg: CtrlMessage)

This procedure should be used to send a controller message along the focus path. Usually, it is called within an implementation of *HandleCtrlMsg*. This is only necessary if it is not sufficient to just set the *focus* parameter of *HandleCtrlMsg* to the embedded view to which forwarding should occur. This in turn is only necessary if *HandleCtrlMsg* needs to do some postprocessing after forwarding has occurred. In this case, it calls *ForwardCtrlMsg* and then makes sure that *focus = NIL*.

Pre

f # NIL    20

PROCEDURE **Update** (v: View; rebuild: BOOLEAN)

Causes view *v* to be restored, in *all* frames displaying *v*. The update occurs delayed, after the currently executing command has terminated. *rebuild* should be set to *keepFrames* for non-containers or for operations which didn't modify the layout of a container (i.e., the places and sizes of embedded views). Otherwise, *rebuildFrames* should be passed.

Pre

v # NIL    20

PROCEDURE **UpdateIn** (v: View; l, t, r, b: INTEGER; rebuild: BOOLEAN)

Causes rectangle *(l, t, r, b)* of view *v* to be restored, in *all* frames displaying *v*. The update occurs delayed, after the currently executing command has terminated. *rebuild* should be set to *keepFrames* for non-containers or for operations which didn't modify the layout of a container (i.e., the places and sizes of embedded views). Otherwise, *rebuildFrames* should be passed.

Pre

v # NIL    20

PROCEDURE **ReadView** (VAR rd: Stores.Reader; OUT v: View)

Reads view *v*, using reader *rd*. If internalization is not possible, an alien view is returned.

PROCEDURE **WriteView** (VAR wr: Stores.Writer; v: View)

Writes view *v*, using writer *wr*. Alien views are handled correctly.

PROCEDURE **CopyOf** (v: View; shallow: BOOLEAN): View

Returns a shallow or deep copy of *v*.

Pre

v # NIL    20

Post

result # NIL

PROCEDURE **CopyWithNewModel** (v: View; m: Models.Model): View

Copies a view and assigns a new model to the copy.

Pre

v # NIL    20

v.ThisModel() # NIL    21

m # NIL    22

TYP(m) = TYP(v.ThisModel())    23

PROCEDURE **ReadFont** (VAR rd: Stores.Reader; OUT f: Fonts.Font)

Reads a font from a reader.

Post

f # NIL

PROCEDURE **WriteFont** (VAR wr: Stores.Writer; f: Fonts.Font)

Writes a font to a writer.

Pre

f # NIL    20

PROCEDURE **IsPrinterFrame** (f: Frame): BOOLEAN

This function can be used to determine whether *f*'s view is currently being restored on a printer or preview port.

Pre

f # NIL    20

PROCEDURE **InstallFrame** (host: Frame; view: View; x, y, level: INTEGER; focus: BOOLEAN)

If *view* has no corresponding embedded frame in *host*, *Install* allocates and installs a new one for it, otherwise the existing frame is kept. Parameters *x* and *y* give the position of the view's top-left corner relative to the container view's (i.e., *host.view*) top-left corner. The frame ordering can be influenced by passing a view level at which *view* lies logically. Levels need not be unique (pass 0 if you don't care, e.g., if you never have overlapping frames). A frame always lies "above" other frames with smaller levels. Parameter *focus* tells whether the frame is part of the focus path.

Pre

host # NIL    20

host is opened in window    21

view # NIL    22

view.context # NIL    23

view.Domain() # NIL    24

PROCEDURE **ThisFrame** (host: Frame; view: View): Frame

Searches the embedded frame of *host* which contains view *view*.

Pre

host # NIL    20

Post

result = NIL

    view = NIL  OR  not found

result # NIL

    view # NIL  &  found

    result.view = view

PROCEDURE **FrameAt** (host: Frame; x, y: INTEGER): Frame

Searches the embedded frame of *host* which contains point *(x, y)*.

Pre

host # NIL    20

Post

result = NIL

    no embedded frame at (x, y)

result # NIL

    result contains (x, y)

PROCEDURE **Old** (ask: BOOLEAN; VAR loc: Files.Locator; VAR name: Files.Name;

                                VAR conv: Converters.Converter): View

This procedure looks up a file, reads in the document in this file, and then returns the root view of this document. If the file is already opened in a window, the root view of this window's document is returned instead of reading the file. Parameter *ask* determines whether or not the user is asked interactively for the triple *(loc, name, conv)*, via a standard file dialog. With this dialog, the user can navigate in the host file system's directory structure.

*loc, name, conv* are treated as in-out parameters. On input, their values are used as defaults. If *ask* then the user may cause them to change.

*loc, name* determine the file from which the document was read.

*conv* determines the converter which is used for reading the document. *conv = NIL* means that no conversion is necessary, i.e., the file format already has the standard BlackBox format.

Pre

ask  OR  loc # NIL    20

ask  OR  name # ""    21

Post

result = NIL

    loc.res # 0

result # NIL

    loc.res = 0

    result.context # NIL

~ask

    loc = loc'  &  name = name'  &  conv = conv'

PROCEDURE **OldView** (loc: Files.Locator; name: Files.Name): View

*OldView* is an abbreviation of *Old(dontAsk, loc, name, NIL)*.

PROCEDURE **Register** (view: View; ask: BOOLEAN; VAR loc: Files.Locator;

                                        VAR name: Files.Name; VAR conv: Converters.Converter;

                                        OUT res: INTEGER)

Saves *view*'s document in a file. Parameter *ask* determines whether or not the user is asked interactively for the triple *(loc, name, conv)*, via a standard file dialog. With this dialog, the user can navigate in the host file system's directory structure.

*loc, name, conv* are treated as in-out parameters. On input, their values are used as defaults. If *ask* then the user may cause them to change.

*loc, name* determine the file to which the document is written.

*conv* determines the converter which is used for writing the document. *conv = NIL* means that no conversion is necessary, i.e., the file format gets the standard BlackBox format.

Pre

view # NIL    20

ask  OR  loc # NIL    22

ask  OR  name # ""    23

Post

operation was successful

    res = 0

operation was not successful

    res # 0

PROCEDURE **RegisterView** (view: View; loc: Files.Locator; name: Files.Name)

*RegisterView* is an abbreviation of *Register(view, dontAsk, loc, name, nil, res)*.

PROCEDURE **Open** (view: View; loc: Files.Locator; name: Files.Name;

                                    conv: Converters.Converter)

Open *view* in a new window. *(loc, name)* determines the file associated with *view*, if there is any. *conv* is the converter which will be used when the user saves the document. *conv = NIL* is passed when saving in the standard BlackBox file format is desired.

Pre

view # NIL    20

(loc = NIL) = (name = "")    21

PROCEDURE **OpenView** (view: View)

*OpenView* is an abbreviation of *Open(view, NIL, "", NIL)*.

PROCEDURE **OpenAux** (view: View; title: Title)

Opens *view* in an auxiliary window with *title* title.

Pre

view # NIL    20

title # ""    21

PROCEDURE **Deposit** (view: View)

Deposit a view for later use, typically for opening it in a window, or for pasting it to the focus.

Deposit is used only by allocation commands, i.e., commands which allocate and then deposit a concrete view type.

Pre

view # NIL    20

PROCEDURE **RestoreDomain** (domain: Stores.Domain)

This procedure forces a restoration of all update regions on views of *domain*.

Normally, the display is updated in a delayed fashion, i.e., an update region is built for all invalid view areas (using *Update* and *UpdateIn*), and the display update according to this region is performed when the framework is idle, i.e., between commands. However, sometimes it is necessary to enforce a display update *during* a command, for which this procedure can be used. The need for enforced update comes from scrolling: if a view should be scrolled, the effect of scrolling should become immediately visible.

PROCEDURE **Scroll** (v: View; dx, dy: INTEGER)

Scroll the contents of each frame on *v* by *(dx, dy)*.

Pre

v # NIL    20

PROCEDURE **SetDir** (d: Directory)

Assigns directory.

Pre

d # NIL    20

Post

stdDir' = NIL

    stdDir = d

stdDir' # NIL

    stdDir = stdDir'

dir = d

The following procedures are used internally:

PROCEDURE **MarkBorders** (root: RootFrame)

PROCEDURE **MarkBorder** (host: Frame; view: View; l, t, r, b: INTEGER)

PROCEDURE **Fetch** (OUT view: View)

PROCEDURE **Available** (): INTEGER

PROCEDURE **ClearQueue**

PROCEDURE **RemoveFrame** (host, f: Frame)

PROCEDURE **RemoveFrames** (host: Frame; l, t, r, b: INTEGER)

PROCEDURE **BroadcastModelMsg** (f: Frame; VAR msg: Models.Message)

PROCEDURE **BroadcastViewMsg** (f: Frame; VAR msg: Message)

PROCEDURE **HandleCtrlMsg** (op: INTEGER; f, g: Frame; VAR msg: CtrlMessage;

                                                    VAR target, front, req: BOOLEAN)

PROCEDURE **SetRoot** (root: RootFrame; view: View; front: BOOLEAN; flags: SET)

PROCEDURE **AdaptRoot** (root: RootFrame)

PROCEDURE **RootOf** (f: Frame): RootFrame

PROCEDURE **HostOf** (f: Frame): Frame

PROCEDURE **UpdateRoot** (root: RootFrame; l, t, r, b: INTEGER; rebuild: BOOLEAN)

PROCEDURE **RestoreRoot** (root: RootFrame; l, t, r, b: INTEGER)

PROCEDURE **ValidateRoot** (root: RootFrame)

PROCEDURE **InitCtrl** (p: CtrlMsgHandler)

PROCEDURE **IsInvalid** (v: View): BOOLEAN

PROCEDURE **RevalidateView** (v: View)

