**2 Compound Documents**

In the previous sections, we have discussed a variety of problems that occur with interactive systems, and we have seen various architecture and design approaches that help to solve them.

In the following sections we focus specifically on the programming problems posed by compound documents, and the design patterns that help to solve them.

**2.1 Persistence**

We call the object that is contained in a window a *view*. A view displays itself in the window, and it may interpret user input in this area, such as mouse clicks. In the most general case, a view can be opened and saved as a document. Other views may be created at run-time or are loaded from resource files. The contents of a "Find & Replace" dialog box is an example of the latter case.

In general, a view is a persistent object. This presumes a mechanism for the externalization (saving) and the internalization (opening) of a view. Using object-oriented programming, we can model a view as a specialization of a generic persistent object, which we call a *store*. A store can externalize and internalize its persistent state. A view is a specialized store, i.e., a subtype. In addition to externalizing and internalizing themselves, views can display themselves and can handle user input. These generic views can be further specialized, for example to text views, picture views, and so on (see Figure 2-1).

Figure 2-1. Subtype relations between views and stores

In the BlackBox Component Framework, stores are represented as type *Store* in module *Stores*; views are represented as type *View* in module *Views*; and there are various specific views such as type *View* in module *TextViews* or type *View* in module *FormViews* or type *Control* in module *Controls* (see Figure 2-2). This is only a small selection, actually there are many more view types. The store mechanism's file format is platform-independent, i.e., differences in byte ordering are compensated for.

Figure 2-2. Subtype relations between views and stores in BlackBox

Often, copying of a store is equivalent to writing it to a temporary file, and then reading it in again (externalize old object to file, then allocate and internalize new object from file). This makes it convenient to combine persistence and copying support in the same store abstraction. However, BlackBox stores are able to change their identities or to "dissolve" upon externalization to a file, in which case copying is not equivalent to externalize/internalize anymore. For example, error marker views which indicate syntax errors remove themselves upon externalization - yet a visible error marker can be copied with drag & drop like any other view. To allow for such flexibility, BlackBox doesn't automatically treat copying as an externalize/internalize pair. Stores have explicit *Externalize*, *Internalize* and *CopyFrom* methods. Upon externalization, a store gets the opportunity to substitute another store for itself, possibly *NIL*. This is the mechanism which allows to change its identity to another one, a so-called *proxy*.

**2.2 Display independence**

The computing world is heterogeneous, with different hardware and operating system platforms on the market. The Internet, with its millions of platforms of all flavors, has finally made it clear how important platform-independence of code and data is. This is not easy to achieve, because software and hardware can vary widely between platforms. Display devices, such as monitors and printers, are a good example. Their spatial and color resolutions can differ enormously.

To achieve platform-independent code, views and their persistent state must not refer to specific devices. Instead, the *document space* and the *display space* must be clearly separated and a suitable mapping between the two must be provided by a framework. For example, in the document space, distances are measured in well-defined units such as inches or millimeters, while in the display space, distances are measured in pixels, whatever their actual sizes may be.

To model the separation between display and document space, where the document space is populated by persistent views, we need an abstraction of a display device. We call such an object a *frame*. A frame is a rectangular drawing surface which provides a set of drawing operations. When it is made visible, a view obtains a frame, and via this frame it can draw on the screen (or printer). The frame performs the mapping between the document space units and the pixels of the display device, so that views never need to deal with pixel sizes directly.

Figure 2-3. Separation of display and document space

In the BlackBox Component Framework, document space units are measured in 1/36000 millimeters. This value is chosen such that it minimizes rounding errors for typical display resolutions. A BlackBox frame provides a local coordinate system for its view. For the time being, you can regard a frame as a window and a view as a document. A view which isn't displayed has no frame. Frames are light-weight objects that come and go as needed. They are created and destroyed by the framework, and need not be implemented or extended by the view programmer.

The only case where it is necessary to extend frames is when platform-specific controls or OLE objects need to be wrapped as BlackBox views. Since writing such wrappers is a messy business, the framework provides standard OLE wrappers (Windows version) and wrappers for the important (pre-OLE) standard controls.

**2.3 Multi-view editing**

Simple views work fine if they are small enough to fit on a screen. However, texts, tables, or graphics can easily become larger than a screen or printed page. In this case, a frame (which lies strictly within the boundaries of its display device) can only display part of the view. To see the other parts, the user must *scroll* the view, i.e., change the way in which it translates its data to its local coordinate system. For example, a text view's first line (its origin) may be changed from the topmost line to another one further below, thus displaying a text part further below (see Figure 2-4).

Figure 2-4. Scrolling a text view

The need for scrolling can become very inconvenient when working with two parts of a view that are far apart, because it means that the user often must scroll back and forth. This problem can be solved by *multi-view editing*: the same data is presented in more than one view. The views may differ in their origins only, or they may differ more thoroughly. For example, one view may present a list of number values as a table of numbers, while another view may present the same data as a pie chart (see Figure 2-5).

Figure 2-5. Different kinds of views presenting the same data

An object which represents data that may be presented by several views is called a *model*. The separation into view and model go back to the Smalltalk project at Xerox PARC. In the patterns terminology, the view is an *observer* of the model.

In the BlackBox Component Framework, the model/view separation is optional. Small fixed-size views that easily fit in a window usually don't need separate models, while larger or more refined views do. This can result in a situation like the following one, where a view has a separate model, and both together form the document which is displayed in a window:

Figure 2-6. Separation of models and views in BlackBox

The user can open a second window, which displays its own view, but for the same model. This results in the following situation:

Figure 2-7: Multiple views for the same model

Only one of the two views is externalized when the document is saved in a file. To avoid schizophreny, one window is clearly distinguished as the main window, while all others are so-called *subwindows*. If you close a subwindow, only this window is closed. But if you close the main window, it and all its subwindows are closed simultaneously.

**2.4 Change propagation**

Multi-view editing creates a consistency problem. Editing a view means that the view causes its model to change. As a result, *all* views presenting this model must be updated if necessary, not only the view in which the user currently works. For example, if the user changes a value in the table view of Figure 2-5, then the pie chart in the other view must be updated accordingly, because their common model has changed.

Such a change propagation is easy to implement. After the model has performed an operation on the data that it represents, it notifies all views which present it, by sending them an *update message* describing the operation that has been performed.

Notification could be easily done by iterating over a linear list of views anchored in the model. Such a functionality can be provided once and for all, by properly encapsulating it in a class. This change propagation class provides an interface consisting of two parts. The first part allows to register new views that want to listen to update messages ("subscribe"). The second part allows to send update messages ("publish"):

Figure 2-8. Change propagation mechanism

This mechanism has an important advantage: it decouples model and views such that the model itself need not be aware of its views, and the list of views can be encapsulated in the change propagation mechanism. Such a reduced coupling between model and views is desirable because it reduces complexity. It makes the model unaware, and thus independent, of its views. This is important in an extensible system, because it allows to add new view types without modifying the model. This would not be possible if the model had to know too much about its views.

Sometimes the mechanism described above does too much. It sends update messages to all views, independent of whether they need updating or not. For example, when a word was deleted in a text model, then some text view may display a part of the text which is not affected by the deletion at all. This view need not do anything. Of course, all the views that *do* display the affected text stretch must be updated accordingly.

For this reason, the BlackBox Component Framework sends update messages only to those views which have frames. Since every frame knows its view, the change propagation mechanism actually registers the frames, not the views. Even better, the framework already has a list of frames for window management, so it can reuse this list. Thus, instead of having one change propagation object per model, BlackBox uses one global change propagation service integrated into the window manager. Messages are represented as static message records. This is an unconventional but robust, efficient, and light-weight implementation of the observer pattern.

**2.5 Nested views**

So far, we have assumed that a view (possibly with a model) is a document, and a frame is a window. Actually, things are a bit more difficult. The reason is that views may be nested. Some views are able to contain ("embed") other views; for this reason they are called container views. Some containers are very general, and allow to embed arbitrary numbers and types of other views. For example, a text view may embed table views, bitmap views, and so on. It may even embed another text view, meaning that arbitrarily deep nesting hierarchies can be created.

How does this affect what we have said so far? Obviously, we need to generalize our notion of views and frames. A view can contain other views, which can contain still other views, etc.; i.e., views become hierarchical. In principle, those hierarchies need not even be trees, they can be arbitrary directed acyclic graphs (DAGs). This means that several views in a document may refer to the same model, but no references may go "upwards" in the document hierarchy, because this would lead to an endless recursion when drawing the document, like a TV which displays itself.

The frames represent a subset of the view hierarchy. Since a frame is always completely contained within its parent frame, the frame hierarchy is strictly a tree.

A document can now be regarded as the root of a view DAG, and a window can be regarded as the root of a frame tree.

Figure 2-9. Nested views and frames

This is a straight-forward generalization which doesn't add unexpected complexity. It is a simplified implementation of the *composite* design pattern. It was simplified because the type safety of Component Pascal makes it unnecessary to burden the abstract view type with container-specific operations that are meaningless for non-containers.

Unfortunately, more complexity comes in when we consider the combination of nested views and multi-view editing. First of all, multi-view editing means the separation of views and models. An embedded view belongs to the model, since it is part of the data that the container view presents. For example, a user who edits a text view may delete embedded views just like she may delete embedded characters. Figure 2-10 illustrates the situation of two container views with a container model in which some other view is embedded:

Figure 2-10. Container views with model, context, and embedded view

Note the interesting link between a model and an embedded view: the *context* object. It describes the location and possibly other attributes of one embedded view. The context's type is defined by the container. A context is created by the container when the view is being embedded. The embedded view can ask its context about its own location in the container, and may even obtain further information. For example, text container contexts may provide information about the embedded view's position in the text (character index), about its current font and color, and so on.

An embedded view may use its context object not only for getting information about its container, it may also cause the container to perform some action. For example, the embedded view may ask the container, via the context object, to make it smaller or larger. Containers are free to fulfill such requests, or to ignore them. In negotiations between a container and one of its embedded views, the container always wins. For example, a container doesn't allow an embedded view to become wider than the container itself.

A context object can be regarded as a call-back interface which the container installs in the embedded view as a plug-in, so the view can call back the container and obtain services from it. A context is part of a notification mechanism that allows the container to observe messages from the contained views.

In subsequent figures, context objects are not drawn expliclity in order to make the figures easier to understand. Just remember that every view carries a context that it can use when necessary.

Now that nested views and the separation of model and view are supported, it becomes possible to allow subwindows on embedded views, not only on root views. This can be quite helpful, if a small view must be edited. Just open a subwindow for it, enlarge the window, and editing becomes much easier.

Figure 2-11. Subwindow on embedded view

Compound documents introduce a further complication. For root views, scrolling state is not retained when the document is saved to disk. At least most users seem to prefer that the scrolling state of a view not be persistent. This is different for embedded views, though. Setting the origin of an embedded view is not so much for convenience (subwindows are better for that) as for publishing: the user chooses which part of the view should be visible in the container.

The same holds for an undo/redo mechanism: scrolling in a root view is not a true document editing manipulation, and thus undo/redo would add inconvenience rather than usefulness. However, the scrolling of an embedded view must be treated as a genuine editing operation, and thus should be undoable/redoable.

In summary, this means that the implementor of a view must treat non-persistent view state differently, depending whether the view is a root view or an embedded view.

Combining nested views with multi-view editing also destroys the one-to-one relationship between frames and views. As we will see shortly, this is the most far-reaching additional burden that a view implementor must handle.

To see why a view may be displayed in several frames, consider the situation in Figure 2-12:

Figure 2-12. Several frames displaying the same view

Such a situation cannot occur without nested views, because the outermost view is always copied (to allow independent scrolling, or other view-specific operations). But whenever a view is embedded in a container displayed in two different windows, then this view is displayed in two different frames. This makes the management of links between frames and views more complicated; but more importantly, it makes change propagation and screen updates more complicated.

Why? Let us assume that there are n frames which display the same view. Now the view's model is changed, and this change is propagated to all views which display the model. Since there is only *one* view for the model, the change propagation mechanism as we have discussed earlier will send the view an update message exactly once. However, there are n places on the screen which need updating. As a result, the view must perform n updates, by iterating over all frames that display it.

For a view, correctly maintaining the list of frames in which it is displayed is a complicated endeavour. Therefore, the BlackBox Component Framework relieves the view implementor of this error-prone task. A BlackBox view doesn't contain a frame list. Instead, the framework maintains the frame trees automatically. This was the reason that in the above figures, the arrows from frames to views only pointed in one direction, because in BlackBox the view doesn't know its frame(s). The correct frames are passed to a view whenever the framework asks the view to do something that may involve drawing. (This approach is related to the *flyweight* pattern.)

When the view receives an update message and decides that it needs to update its frame(s), it simply asks the framework to send another update message to all its frames. Each of these frames will then ask the view to update the frame where this is necessary. This mechanism will be discussed in more detail in the chapters on view construction.

Since there is no more one-to-one relationship between views and frames anymore, it is helpful to summarize the differences between frames and view:

**aspect    frame    view**

lives in    display space    document space

persistent    no    yes

extended by view implementor    usually not    yes

may contain device-specific data    yes    no

hierarchy    tree    DAG

associated with    one view    0..n frames

update caused by    its view    its model (if any)

completely enclosed by container    yes    not necessarily

Table 2-13. Differences between frames and views

**2.6 Delayed updates**

In the previous section we have seen how updates are performed in two steps: a model notifies its views of a change, and the views notify their frames of the region which needs updating.

Now consider a complex model change. For example, a selection of graphical objects, which are part of a graphics model, are moved by some distance. It would be possible to perform an update in three phases: in the first phase, the selected objects would be removed (requiring an update); then the selected objects would be moved; and finally, the selected objects would be inserted again at their destination (requiring another update).

This approach is not very efficient, since it requires two notifications and partial updates. Furthermore, it needs careful sequencing of the actions, since some of them need the model's state prior to movement, and others need the model's state after movement.

There is a much simpler and more efficient approach. The idea is to delay the second phase of the updates, i.e., the phase where the frames for a view are redrawn. Instead of updating the frames immediately, the view only remembers the geometrical region which needs updating. In the above example, it could add the areas of the graphical objects prior to movement and their areas after movement (see Figure 2-14).

Figure 2-14. Update region of a selection which was moved

The accumulated update region is then updated in one single step after the translation of the graphical objects has been performed. In the figure above, the update region is the total hatched area, consisting of the area occupied by the selection before the movement and the area occupied by it after the movement.

The framework must support this approach by providing some mechanism to add up geometrical regions, and to redraw a frame in exactly this region ("clipping" away all unnecessary drawing, to increase performance and to minimize flicker).

This delayed or *lazy updating* approach is used by the BlackBox Component Framework and all major windowing systems that exist today. Its only drawback is that for long-running commands, it may be necessary to force intermediate updates in order to inform the user of the command's progress. For this reason, forced updates are also supported by the BlackBox framework.

**2.7 Container modes**

To create graphical user interfaces, it is convenient to create form layouts interactively, rather than burning in the coordinates of controls in the source code of a program. A visual designer, i.e., a special-purpose graphics editor can be used for interactive manipulation of form layouts. This suggests two different modes in which a form, and its controls, can be used: design-time and run-time. When a form layout is edited in a visual designer, it is *design-time*: the form is designed, but not yet used. The completed form can be saved and later opened for use by the end user. This is called *run-time*.

There are two different approaches to handling layouts at run-time: either the framework allows to open the data that has been saved by the visual designer, or it runs code that has been generated out of this saved data. The first approach requires an interpreter for the visual designer's data, the second approach requires a generator between the visual designer and actual use of a form. Typically, the generator generates source code, which is possibly completed by the programmer, and then compiled into machine code using a standard programming language compiler.

Today, the latter approach should be considered obsolete, although it is still used by many tools: it is merely an inconvenient detour. Moreover, when it forces programmers to edit the generated source code, it makes iterative changes to a layout problematic: the source code has to be regenerated without losing the changes that have been made by the programmer. This is a needless source of inconsistencies, complexity, and it slows down development.

The former approach is much simpler. The visual designer's output data is treated as a resource that can be utilized *as is* at run-time. This requires an interpreter for the visual designer's file format.

The most elegant approach is to use the same editor (i.e., view implementation) and thus the same file format both for design-time and run-time. As file format, the standard format of a compound document can be reused. This means that a form layout at design-time is a completely ordinary compound document. At run-time, it is still the same document, albeit its interaction with the user is different.

At design-time, a control is a passive box which can be moved around, copied or deleted, but which has no interactive behavior of its own. At run-time, a control may be focused and the control's *contents* may be edited. For example, the string contained in a text field control may be edited, or a button may be pressed and released again.

The direct use of compound documents as user interfaces is called *compound user interfaces*. Note that this generalization of compound documents renders the term "document" rather misleading, because an application may contain many forms (and thus compound user interface documents) even if it does not manipulate any documents in the traditional sense (i.e., as seen by an end-user).

The BlackBox Component Framework utilizes its support for multi-view editing to go one step further. Since a document can be opened in several views simultaneosly, it is possible to open a form layout in two windows. BlackBox allows to use form views in different modes: layout and mask mode. These modes correspond to design-time and run-time, with one major improvement: they can be used simultaneously. For example, one window may display a form in layout mode, and another window may display it in mask mode (see Figure 2-15). When the developer edits the layout in the layout mode window, any layout change is immediately reflected in the mask mode window. Vice versa, user input in the mask mode window immediately becomes visible in the layout window. This is a result of the standard change propagation mechanism of BlackBox.

Figure 2-15. Data entry form in a layout-mode view (behind) and in a mask-mode view (front)

In fact, BlackBox is even more general. In addition to layout and mask mode, it supports a few further modes. The *edit* mode allows to modify the layout and the contents of controls simultaneously, and a *browser* mode makes the form read-only except for allowing to select and copy out parts of the document.

These general modes are not only available for graphical forms. They are a capability shared by *all* general container views, including text views. This means that it is possible (and sometimes more convenient for the developer) to use a text view instead of a form view when assembling a dialog box. The user won't notice the difference. To simplify the construction of such powerful containers, the framework provides a comprehensive container abstraction, with an abstract container model class, an abstract container view class, and an abstract container controller class. A controller can be regarded as a split-off part of a view. It performs all user interaction, including handling of keyboard input and mouse manipulations, and it also manages selections. There is a 1:1 relation between view and controller at run-time. Separating part of a view into a controller object improves extensibility (different controllers could be implemented for the same view, and the same controller works with all concrete view implementations) and it also allows to reduce complexity: the controller of a complex view, such as a container view, can become large, which makes the separation of concerns into different types (and typically into different modules) a good idea. In terms of design patterns, a controller is a *strategy* object.

The container abstractions of BlackBox have been designed to abstract from specific container user interfaces. This means that details of the container look-and-feel (such as the hatched focus marks of an OLE object) are hidden from the developer. In turn, this makes it possible to implement containers in different ways for different platforms, without affecting the developer of containers. In particular, the OLE and Apple OpenDoc container look-and-feel have been implemented. While OpenDoc isn't relevant anymore, some of the human interface guidelines developed for it are still useful today.

**2.8 Event handling**

When a user interacts with a compound document, this interaction always occurs in one view at a time. This view is called the current *focus*. By clicking around, the user can change the current focus. Among other events, the focus handles keyboard events, i.e., a keypress is interpreted by the focused view. The focus view and its containing views are called the *focus path*, with the innermost view being the focus.

It is possible to let a user interface framework manage the current focus. This requires a central manager for the focus. A more light-weight approach is to make focus management a decentral activity: leave focus management to the individual container views, which have to deal with focus changes and focus rendering anyway. Every container simply remembers which of its embedded views is the current focus, if any. The container is oblivious whether this view is on the current focus path or not.

In BlackBox, user events are sent along the focus path as message records, starting at the outermost view. Messages records are static (stack-allocated) variables of some subtype of *Controllers.Message*. There are controller messages for keyboard input, mouse tracking, drag & drop, and so on. A controller message is forwarded along the focus path until one of the views either interprets it or discards it. This view by definition is the focus.

In terms of design patterns, the focus path is a *chain of responsibility*.

What happens when the focus view receives and interprets a controller message? The view (or its model, if it has one) performs some operation on its own state. If this operation affects persistent state of the view or model, it should be reversible ("undoable"). How is an undo/redo mechanism implemented?

The main idea is that upon receipt of a controller message, the view / model doesn't perform a state modification directly. Instead, it creates a special *operation object* and registers it in the framework. The framework then can call the operation's appropriate procedure for performing the actual do/undo/redo functionality.

Operations are managed per document. Every document contains two operation stacks; one is the undo stack, the other is the redo stack. Executing an operation for the first time pushes the object on the undo stack and clears the redo stack. When the user performs an undo, the operation on top of the undo stack is undone, removed from the undo stack, and pushed onto the redo stack. When the user performs a redo, the operation on top of the redo stack is redone, removed from the redo stack, and pushed onto the undo stack.

A document's undo and redo stacks are cleared when the document is saved (check point). Furthermore, they may be cleared or made shallower when the framework runs out of memory. For this purpose, the garbage collector informs the framework about low-memory conditions, so old operation objects can be thrown away.

Some operations, such as a moving drag & drop (in contrast to the normal copying drag & drop), modify two documents simultaneously. In these rare cases, two operations are created: one for the source document (e.g., a delete operation) and one for the destination document (e.g., an insert operation).

Figure 2-16. Sequence of operations and resulting modifications of undo and redo stacks.

In Figure 2-16, a sequence of operations is shown, from 1) to 8). For the last five situations, the resulting undo and redo stacks are shown. For example, after operation 5), the undo stack contains the operations Inserting, Set Properties, and Inserting (from top to bottom of stack), while the redo stack contains Deleting.

The undo/redo mechanism is only concerned with the persistent state of a document. This is the state which can be saved in a file. Modifications of temporary state, such as a view's scroll position, are not recorded as operations, and thus cannot be undone (at least for root views).

Undoable operations either modify the persistent state of a view, or of its model (controller state is mostly temporary). The framework knows to which document a persistent object belongs, because all persistent objects of a document share the same domain. Domains will be discussed in Part III where the store mechanism is discussed in more detail.

The BlackBox Component Framework is unique in that its undo/redo mechanism is component-oriented: it allows to compose undoable operations into *compound operations*, which are undoable as a whole. Without this capability, nested operations would be recorded as a flat sequence of atomic operations.

Consider what this would mean for the end user. It would mean that the user could execute a menu command, which causes the execution of a hierarchy of operations. So far so good. But when the user wanted to undo this command, he or she would have to execute Edit->Undo individually for every single operation of the command, instead of only once.

For this reason, BlackBox provides support for arbitrarily nested operations: modules *Models* and *Views* both export a pair of *BeginScript* / *EndScript* procedures. In this context, "script" means a sequence or hierarchy of atomic operations which is undoable as a whole. Model and view operations can be freely mixed in a script.

Abstract operations are very light-weight. They only provide one single parameterless procedure, called *Do*. This procedure must be implemented in a reversible way, so that if it is executed an even number of times, it has no effect. If it is executed an odd number of times, it has the same effect as when it is executed once.

In the design patterns terminology, an operation is called a *command*.

**2.9 Controls**

Controls are light-weight views that only provide very specific functionality, which only makes sense in concert with other controls and a container. Typical controls are command buttons and text entry fields. They are used in dialog boxes or date entry forms. The parameters or other data being represented by a control must somehow be manipulated by a corresponding program. For example, a *Find* command takes a search string as input, or the data entered into a form must be sent to an SQL database. There may even be complicated interactions between the controls of a form. For example, when a search string is empty, the *Find* button may be disabled. If something is typed in, the button is enabled. Such interactions can become very involved.

In principle, a control is simply an observer view on its data. This means that it would be most straight-forward to implement the data (e.g., the search string) as a model.

For BlackBox, it was felt that this approach is too heavy-weight and unconvenient. Instead, a special implementation of the observer pattern was realized, where typical applications never need to access control objects directly. They are completely "abstracted away". The programmer only deals with the observed data. In order to make the definition and manipulation of this data as convenient as possible, it is simply a global Component Pascal variable, called an *interactor*. A control has a symbolic name (e.g. "TextCmds.find.ignoreCase") which enables it to find its variable, using built-in metaprogramming facilities of BlackBox. In the interactor's module itself, there is no object reference to a control.

When a control is being edited, it transparently uses the standard change propagation mechanism of BlackBox, i.e., it broadcasts view messages that notify other controls which may display the same variable.

Although the interactor's module doesn't have access to its controls, it can still influence the way controls are displayed (e.g., enabled or disabled) and the way they interact with each other. This is done by exporting suitable *guard* and *notifier* procedures. They are attached to a control in the same way as its interactor link, using a suitable control property editor. The important thing is that the program need not access the controls directly, and no direct control-to-control interactions need to be programmed. This greatly simplifies the implementation and maintenance of complex user interface behaviors. The mechanism is described in more detail in Chapter 4.

In principle, the framework accesses the interactor's module (via metaprogramming) as a *singleton mediator*.

