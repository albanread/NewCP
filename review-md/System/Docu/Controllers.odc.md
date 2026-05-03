**Controllers**

DEFINITION Controllers;

    IMPORT Fonts, Ports, Stores, Views;

    CONST

        frontPath = FALSE; targetPath = TRUE;

        decLine = 0; incLine = 1; decPage = 2; incPage = 3; gotoPos = 4;

        nextPageX = 0; nextPageY = 0; gotoPageX = 2; gotoPageY = 3;

        cut = 0; copy = 1; pasteChar = 2; paste = 4;

        doubleClick = 0; extend = 1; modify = 2;

        noMark = FALSE; mark = TRUE;

        hide = FALSE; show = TRUE;

    TYPE

        Controller = POINTER TO ABSTRACT RECORD (Stores.Store) END;

        Directory = POINTER TO ABSTRACT RECORD

            (d: Directory) New (): Controller, NEW, ABSTRACT

        END;

        Forwarder = POINTER TO ABSTRACT RECORD

            (n: Forwarder) Forward (target: BOOLEAN; VAR msg: Message), NEW, ABSTRACT;

            (n: Forwarder) Transfer (VAR msg: TransferMessage), NEW, ABSTRACT

        END;

        Message = Views.CtrlMessage;

        PollFocusMsg = EXTENSIBLE RECORD (Message)

            focus: Views.Frame

        END;

        PollSectionMsg = RECORD (Message)

            focus, vertical: BOOLEAN

            wholeSize, partSize, partPos: LONGINT;

            valid, done: BOOLEAN

        END;

        PollOpsMsg = RECORD (Message)

            type: Stores.TypeName;

            pasteType: Stores.TypeName;

            singleton: Views.View;

            selectable: BOOLEAN;

            valid: SET

        END;

        ScrollMsg = RECORD (Message)

            focus, vertical: BOOLEAN

            op: INTEGER;

            pos: LONGINT;

            done: BOOLEAN

        END;

        PageMsg = RECORD (Message)

            op: INTEGER;

            pageX, pageY: INTEGER;

            done, eox, eoy: BOOLEAN

        END;

        TickMsg = RECORD (Message)

            tick: INTEGER

        END;

        MarkMsg = RECORD (Message)

            show, focus: BOOLEAN

        END;

        SelectMsg = RECORD (Message)

            set: BOOLEAN

        END;

        RequestMessage = ABSTRACT RECORD (Message)

            requestFocus: BOOLEAN

        END;

        EditMsg = RECORD (RequestMessage)

            op: INTEGER;

            modifiers: SET;

            char: CHAR;

            view: Views.View;

            w, h: INTEGER;

            isSingle: BOOLEAN;

            clipboard: BOOLEAN

        END;

        ReplaceViewMsg = RECORD (RequestMessage)

            old, new: Views.View

        END;

        CursorMessage = ABSTRACT RECORD (RequestMessage)

            x, y: INTEGER

        END;

        PollCursorMsg = RECORD (CursorMessage)

            cursor: INTEGER;

            modifiers: SET

        END;

        TrackMsg = RECORD (CursorMessage)

            modifiers: SET

        END;

        WheelMsg = RECORD (CursorMessage)

            done: BOOLEAN;

            op, nofLines: INTEGER

        END;

        TransferMessage = ABSTRACT RECORD (CursorMessage)

            source: Views.Frame;

            sourceX, sourceY: INTEGER

        END;

        PollDropMsg = RECORD (TransferMessage)

            mark: BOOLEAN;

            show: BOOLEAN;

            type: Stores.TypeName;

            isSingle: BOOLEAN;

            w, h: INTEGER;

            rx, ry: INTEGER;

            dest: Views.Frame

         END;

        DropMsg = RECORD (CursorMessage)

            view: Views.View;

            isSingle: BOOLEAN;

            w, h: INTEGER;

            rx, ry: INTEGER

        END;

    VAR path-: BOOLEAN;

    PROCEDURE Forward (VAR msg: Message);

    PROCEDURE FocusFrame (): Views.Frame;

    PROCEDURE FocusView (): Views.View;

    PROCEDURE FocusModel (): Models.Model;

    PROCEDURE Register (f: Forwarder);

    PROCEDURE Delete (f: Forwarder);

    PROCEDURE ForwardVia (target: BOOLEAN; VAR msg: Message);

    PROCEDURE SetCurrentPath (target: BOOLEAN);

    PROCEDURE PollSection (VAR msg: PollSectionMsg);

    PROCEDURE PollOps (VAR msg: PollOpsMsg);

    PROCEDURE PollCursor (x, y: INTEGER; OUT cursor: INTEGER);

    PROCEDURE Transfer (x, y: INTEGER; source: Views.Frame; sourceX, sourceY: INTEGER;

                                            VAR msg: TransferMessage);

    PROCEDURE PollDrop (x, y: INTEGER; source: Views.Frame; sourceX, sourceY: INTEGER;

                                            mark, show: BOOLEAN; type: Stores.TypeName; isSingle: BOOLEAN;

                                            w, h, rx, ry: INTEGER; OUT dest: Views.Frame; OUT destX, destY: INTEGER);

    PROCEDURE Drop (x, y: INTEGER; source: Views.Frame; sourceX, sourceY: INTEGER;

                                    view: Views.View; isSingle: BOOLEAN; w, h, rx, ry: INTEGER);

    PROCEDURE PasteView (view: Views.View; w, h: INTEGER; clipboard: BOOLEAN);

    PROCEDURE ResetCurrentPath;

    PROCEDURE SetCurrentPath (target: BOOLEAN);

END Controllers.

Figure 1. Model-View-Controller Separation

A *controller* implements the interactive behavior for a view class. It is an object which transforms controller messages into model or view transformations.

A *controller message* is a message which is sent along exactly one path in a view hierarchy, the focus path. Every view on such a path decides for itself whether it is the terminal of this path, i.e., whether it is the current focus, or whether the message should be forwarded to one of its embedded views. BlackBox supports two focus paths, namely a target and a front focus. Both paths may fall together into one. The target path defines which view is the target of commands in dialogs. The front path defines which view is being edited, via mouse, keyboard, or menus.

Figure 2.  Simplified Example of Focus Hierarchy

It is important to note that all controller messages which are not relevant for a particular view type can simply be ignored.

CONST **frontPath**

This value may be passed as *target* parameter to several procedures of this module. It lets a focus message be sent along the front focus path.

CONST **targetPath**

This value may be passed as *target* parameter to several procedures of this module. It lets a controller message be sent along the target focus path.

CONST **decLine, incLine, decPage, incPage**

These values can be assigned to the *ScrollMsg.op* field. They will cause the receiver view to scroll by increments or decrements of one line or page. It is up to the receiver to define what constitutes a "line" in its model, except that it should be smaller than a page. A page should correspond to the width/height of the focus frame.

CONST **gotoPos**

This value can be assigned to the *ScrollMsg.op* field. It causes the receiver view to scroll to the position given by *ScrollMsg.pos*.

CONST **nextPageX, nextPageY**

This value can be assigned to the *PageMsg.op* field. It causes the receiver view to display the next page in x resp. in y direction.

CONST **gotoPageX, gotoPageY**

This value can be assigned to the *PageMsg.op* field. It causes the receiver view to display a given page in x resp. in y direction.

CONST **cut, copy**

These values can be assigned to the *EditMsg.op* field. They should be handled by the receiver view in the following way:

ꀢ cut    delete selection, and assign a new view containing the deleted data or a copy thereof to *EditMsg.view*

ꀢ copy    copy selection, and assign a new view containing the copied data to *EditMsg.view*

CONST **pasteChar**

This value can be assigned to the *EditMsg.op* field. It denotes the input of some (Unicode or Latin-1) character.

CONST **paste**

This value can be assigned to the *EditMsg.op* field. The receiver view should insert a copy of the data that *EditMsg.view* contains. If it cannot insert the data directly because it doesn't know its type, it should insert a copy of the whole *EditMsg.view* into its model if it has this capability, i.e., if it is a container.

CONST **doubleClick**

This value means that a mouse button has been pressed in a way which is interpreted by the underlying user interface as a "double click". A double click is signaled to the application as a modifier; other modifiers are *extend* and *modify*.

CONST **extend, modify**

BlackBox operates with two virtual modifier keys. The *extend* key is used to extend or toggle selections (usually the *shift* key), the *modify* key to change the default behavior of a command (e.g., to change a drag-and-move into a drag-and-copy, using the control key (Windows) / option key (Mac OS)). The behavior of possible additional modifier keys is platform-specific (see the document on platform-specific issues for such modifiers). Modifier sets are used in *Controllers.TrackMsg, Controllers.EditMsg, *and* Ports.Frame.Input.*

CONST **noMark, mark**

Used internally.

CONST **hide, show**

These constants may be used for the *PollDropMsg* message and the *PollDrop* procedure. They determine whether target feedback during drag & drop should be drawn (*show*) or removed (*hide*).

TYPE **Controller (Stores.Store)**

ABSTRACT

A controller is the third component of the model-view-controller triple. Its main purpose is to translate controller messages into model or view transformations. Simple applications can implement the controller's functionality in the view itself, thus needing no separate controller object at all. Container views usually have a separate controller.

TYPE **Directory**

ABSTRACT

Usually there is a controller directory in the views module of a subsystem, since the view directory must be able to allocate an appropriate controller. Such a directory is installed by the corresponding controller module upon loading. To guarantee the loading of the corresponding controller module is loaded, the view module's body usually contains a statement of the form *Dialog.Call("XYZControllers.Install, "", res)*.

PROCEDURE (d: Directory) **New** (): Controller

NEW, ABSTRACT

Returns a new controller.

Post

result # NIL

result.ThisView() = NIL

TYPE **Forwarder**

ABSTRACT

Used internally.

PROCEDURE (n: Forwarder) **Forward** (target: BOOLEAN; VAR msg: Message)

NEW, ABSTRACT

Used internally.

PROCEDURE (n: Forwarder) **Transfer** (VAR msg: TransferMsg)

NEW, ABSTRACT

Used internally

TYPE **Message**

ABSTRACT

Base type of all controller messages. In contrast to model and view messages, a controller message is never broadcast. Instead, it is passed along a focus path. The target and front focus paths are predefined.

TYPE **PollFocusMsg (Message)**

EXTENSIBLE

This message is sent to find out the leaf view of a focus path. The message is handled by the framework itself. If your view should not give away its identity when it is focus, set *focus* to *NIL* when receiving the message. Otherwise ignore the message.

**focus**: Views.Frame

After the message returns from the traversal of a focus path, it should contain the frame of the leaf view of this path.

TYPE **PollSectionMsg (Message)**

This message is sent to poll the focus view's current scroll state. BlackBox contains a generic scrolling mechanism which scrolls simply by changing the scrolled frame's origin. Explicit handling of scrolling is only necessary for views which can become extremely large, and whose efficient implementation crucially depends on keeping frames small.

**focus**: BOOLEAN

This flag tells whether a container should forward the message to its focus or not. Non-containers obviously cannot forward to a focus.

**vertical**: BOOLEAN

Tells whether the vertical or horizontal direction is polled.

**wholeSize**: INTEGER    wholeSize >= 1

This value denotes the focus view's width or height, in coordinates which a view can freely choose, i.e., they need not necessarily be universal units.

**partSize**: INTEGER    0 <= partSize <= wholeSize

This value denotes the focus view frame's width or height, in the same coordinates as above. If *partSize* cannot be easily defined, it should be set to 0.

**partPos**: INTEGER    0 <= partPos <= wholeSize - partSize

This value denotes the focus view's origin, in the same coordinates as above.

**valid**: BOOLEAN

The receiving view should set this flag if it supports scrolling in the given direction. *valid* indicates that *wholeSize*, *partSize*, and *partPos* are valid indicators of the view's scroll position.

**done**: BOOLEAN

This flag should be set if the message has been interpreted, i.e., if the above output fields have been set. For some controllers this may depend on *vertical*, i.e., the controller only supports one scrolling direction.

TYPE **PollOpsMsg (Message)**

This message is sent to inquire which editing operations the focus view supports, depending on its current selection.

**type**: Stores.TypeName

This field denotes a context for the focus view. This context is used to determine which menus are relevant for the focus view. As a convention, a view assigns the type name of its abstract base pointer type to *type*, e.g., "TextViews.View". This convention guarantees globally unique context names, since module names are considered globally unique. If the view doesn't support any such context, ignore this field.

**pasteType**: Stores.TypeName    valid iff type = paste

The type of the view of which a copy would be pasted, if a paste operation occurred.

**singleton**: Views.View

A container view which supports a selection should set this field to the selected view, if this view is the only contents currently selected.

**selectable**: BOOLEAN

This field should be set to *TRUE* if the focus view contains selectable elements, independent of whether they are currently selected or not.

**valid**: SET    valid IN {cut, copy, paste}

This set denotes which edit operations are currently possible, out of the set *{cut, copy, paste}*. It is used by the framework to enable or disable the appropriate menu items. However, there is no guarantee that the framework will not send an edit message for cut/copy/paste even if the view returns an empty set in *valid*. Such a message can simply be ignored.

TYPE **ScrollMsg (Message)**

This message is sent in order to let the focus view scroll to another position. It is used only in conjunction with *PollSectionMsg* (cf. above).

**focus**: BOOLEAN

This flag tells whether a container should forward the message to its focus or not. Non-containers obviously cannot forward to a focus.

**vertical**: BOOLEAN

Denotes whether scrolling should occur in the vertical or in the horizontal direction.

**op**: INTEGER    op IN {decLine..gotoPos}

Scroll operation to be performed.

**pos**: INTEGER    valid iff *op = gotoPos*    pos >= 0

This denotes the position to be scrolled to, in the same coordinates that the *PollSectionMsg* uses.

**done**: BOOLEAN

This flag should be set if the message has been interpreted; for some controllers, this may depend on *op*.

TYPE **PageMsg (Message)**

A page message is similar to a scroll message, but its measures are in pages. It can be interpreted if the view should behave differently depending whether it is being printed or not. This is done e.g. in *TextViews* to avoid the last line on a page to become clipped, which is acceptable on screen.

If this message is not interpreted for a view which cannot be printed on one page, a default printing strategy is used which decomposes the view into suitable pieces for printing.

**op**: INTEGER    op IN {nextPageX, nextPageY, gotoPageX, gotoPageY}

Where to scroll to.

**pageX, pageY**: INTEGER

Current page in x and in y directions.

**done**: BOOLEAN

This flag should be set if the message has been interpreted; for some controllers, this may depend on *op*.

**eox, eoy**: BOOLEAN

These flags should be set when it is attempted to go beyond the last page in the x resp. in y direction.

TYPE **TickMsg (Message)**

This message is sent to the front focus view periodically. It can be used to realize effects like a blinking caret.

**tick**: INTEGER

Tick count. The difference between two ticks is given by the global variable *resolution*.

TYPE **MarkMsg (Message)**

This message is sent whenever the target or front focus paths change. Before the change, the message is sent along the focus path with *show = FALSE*, such that the focus view can switch off visible marks like the selection or the caret. After the change (e.g., another window coming to the top), a *MarkMsg* is sent along the focus path with *show = TRUE*, such that the focus view can switch on its marks again, if there are any.

Behavior can be different if the focus path changes due to window reordering or due to user interaction (a user clicking in some other embedded view).

In the former case, a focused view may still keep its focus state (selection, caret), even while it is temporarily in a non-focus window. When the window becomes focus again, the old focus view becomes focus again. In this case, it should show the same selection/caret state again.

In the latter case, a view that becomes newly focused may behave in a special way. In particular, a control may focus its contents (e.g., a text field may select the entire text string that it contains). To detect such a situation, a field *focus* is provided in the *MarkMsg*.

Wrapper views and special container views (i.e., containers not derived from the types in module *Containers*) have to forward the *MarkMsg* to their embedded views.

**show**: BOOLEAN

Tells whether the focus view's marks should be switched on or off.

**focus**: BOOLEAN

Tells whether the view is being focused or defocused.

A view may use this information e.g. to select itself upon becoming focus (this is done by text field controls). This is done in the following way:

    IF msg.focus THEN

        IF msg.show THEN    *(* set selection on focus *)*

            *... select contents of view ...*

        ELSE    *(* remove selection on defocus *)*

            *... remove selection ...*

        END

    END

TYPE **SelectMsg (Message)**

This message is sent when the focus view should select all selectable items, or when it should deselect all selected items. Selection occurs e.g. when the *Edit->Select All* command is executed.

**set**: BOOLEAN

Determines whether everything should be selected *(set = TRUE)* or whether everything should be deselected *(set = FALSE)*.

TYPE **RequestMessage (Message)**

ABSTRACT

A view (or its controller) receiving a request message can indicate, by setting *requestFocus* to *TRUE*, that it wants to become focus afterwards, if it isn't already.

**requestFocus**: BOOLEAN

Set this flag to *TRUE* if receiver should become/remain focus after the message has been handled.

TYPE **EditMsg (RequestMessage)**

This message is sent when a key was pressed, or when a cut/copy/paste operation was invoked. The following operations can be supported (for every supported operation except pasting of characters, the corresponding flag in *PollOpsMsg.valid* should be set when receiving a *PollOpsMsg*):

ꀢ cut

Create a clone of the focus view, with a copy of its selection as contents. Assign this clone to *EditMsg.view* and delete the focus' selection. There is one exception: if the selection consists of exactly one view (a singleton), *this* view's clone should be copied to *EditMsg.view*, not a clone of its container.

ꀢ copy

Create a clone of the focus view, with a copy of the focus' selection as contents. Assign this clone to *EditMsg.view*. If the selection consists of exactly one view (a singleton), this view's clone should be copied to *EditMsg.view*.

ꀢ pasteChar

Interpret *EditMsg.char*, e.g., insert it into a text model.

*char* may be a control character. For control characters, check field *modifiers* also.

ꀢ paste

Insert *EditMsg.view* view into the focus view's contents.

If the receiver is a container:

    If *EditMsg.isSingle*:

        It should insert the view as a complete view, independent of its type.

        This means that merging of contents is not permitted, even if this were possible.

    If *~EditMsg.isSingle*:

        If it knows the contents of *EditMsg.view*, it should insert it, otherwise the complete view.

If the receiver is not a container:

    If it knows the contents of *EditMsg.view*, it should insert it, otherwise it should do nothing.

**op**: INTEGER    op IN {cut .. paste}

Operation to be performed.

**modifiers**: SET    valid iff op= pasteChar

Modifier keys.

**char**: CHAR    valid iff op = pasteChar

Character to be pasted, or to be interpreted in the case of control characters.

**view**: Views.View    valid iff op IN {paste, cut, copy}  &  view # NIL  &  view.context = NIL

If op = paste:    (IN parameter)

View which should be pasted.

If op IN {cut, copy}:    (OUT parameter)

View which should be assigned by the message handler.

**w, h**: INTEGER    valid iff op IN {paste, cut, copy}

                                w >= Views.undefined  &  h >= Views.undefined    [units]

If op = paste:    (IN parameter)

The desired width and height of the pasted view. These values can be treated as hints. If they are not suitable, others can be used. The value *Views.undefined* should be handled also.

If op IN {cut, copy}:    (OUT parameter)

Current width and height of the view.

**isSingle**: BOOLEAN    valid iff op IN {paste, cut, copy}

If op = paste:    (IN parameter)

Tells whether the pasted view should be inserted as singleton, i.e., not be merged even if this were possible.

If op IN {cut, copy}:    (OUT parameter)

The message handler should set this flag if the cut/copied view is selected as a singleton.

**clipboard**: BOOLEAN    valid iff op IN {cut, copy, paste}

This input parameter tells whether the cut/copied view will be transferred to the system clipboard, or whether the pasted view comes from the system clipboard.

TYPE **ReplaceViewMsg (RequestMessage)**

A container should check whether it contains *old*. If so, it should replace *old* by *new*, without modifying the view's context in any way.

**old, new**: Views.View    old # NIL & new # NIL

*old* should be replaced by *new*.

TYPE **CursorMessage (RequestMessage)**

ABSTRACT

The base type of all messages which denote some interaction that depends on the current cursor position. The cursor position is always measured in units relative to the view's top-left corner.

**x, y**: INTEGER    [units]

Current cursor position.

TYPE **PollCursorMsg (CursorMessage)**

This message is sent regularly to inquire which cursor the focus view currently desires.

**cursor**: INTEGER    cursor IN {Ports.arrowCursor .. Ports.refCursor}

This field can be set to the cursor appropriate for the focus view.

**modifiers**: SET

This is an input field that gives the current modifer state of the mouse.

TYPE **TrackMsg (CursorMessage)**

Extension

This message is sent when the mouse button is pressed down.

**modifiers**: SET

Determines which modifier keys have been pressed together with the mouse button.

TYPE **WheelMsg (CursorMessage)**

Extension

This message is sent when the wheel on a wheel mouse is rolled.

**done**: BOOLEAN

If a view handles this message it must set the *done* flag to *TRUE*.

**op**: INTEGER

Indicates which kind of event the mouse wheel caused. The same constants as for scrolling are used, but only the following values are valid: *incPage*, *decPage*, *incLine* and *decLine*.

**nofLines**: INTEGER    nofLines >= 1

If *op* is either *incLine* or *decLine* then *nofLines* indicates how many lines should be scrolled. For *incPage* and *decPage* this value is undefined.

TYPE **TransferMessage (CursorMessage)**

ABSTRACT

This is the base type of all messages which denote an interaction between several views, e.g., for drag and drop.

**source**: Views.Frame

The frame from which the interaction started, e.g., where the mouse button has been clicked for dragging.

**sourceX, sourceY**: INTEGER

The position in the source frame where the mouse button has been clicked, e.g., when starting to drag.

TYPE **PollDropMsg (TransferMessage)**

While an item is being dragged around, *PollDropMsgs* are sent to enable feedback about the drop target.

**mark**: BOOLEAN

A container which supports drop feedback should show (hide) its feedback mark when *mark* is set (cleared). You don't need to deal with the view's border mark (the rectangular outline of the view which is drawn while you are dragging over a view), this is handled completely by BlackBox itself.

**show**: BOOLEAN    valid iff mark

Indicates whether the mark should be drawn or removed.

**type**: Stores.TypeName

The type of the view to be dropped. The same naming convention as for the *PollOpsMsg* is used. Based on *type*, the message handler can determine whether the view to be dropped can be accepted. Note that if drag & drop happens completely within BlackBox, the inherited message field *source* may also be used for this test. However, *source* may be *NIL* if OLE drag & drop occurs.

**isSingle**: BOOLEAN

Tells whether the view to be dropped is a singleton selection.

**w, h**: INTEGER

Size of the view to be dropped. May be equal to *Views.undefined*.

**rx, ry**: INTEGER    rx >= 0  &  ry >= 0

The reference point inside the selection where drag & drop started.

**dest**: Views.Frame    (OUT parameter)

The receiver should set *dest* to its own frame, if it would accept a drop.

TYPE **DropMsg (CursorMessage)**

This message is used if a view should be dragged and dropped to the cursor location.

**view**: Views.View    view # NIL

The view which is dropped. It is a copy of the original, and ready to be inserted at the drop destination.

**isSingle**: BOOLEAN

Tells whether the view to be dropped is a singleton selection.

**w, h**: INTEGER    [units]

The size of the dropped view. One or both sizes may have the value *Views.undefined*.

**rx, ry**: INTEGER

The reference point inside the selection where drag & drop started.

VAR **path**-: BOOLEAN

Used internally.

PROCEDURE **Forward** (VAR msg: Message)

Send *msg* to to current focus.

PROCEDURE **FocusFrame** (): Views.Frame

Returns the current focus frame, if there is any.

PROCEDURE **FocusView** (): Views.View

Returns the current focus view, if there is any.

PROCEDURE **FocusModel** (): Models.Model

Returns the current focus view's model, if there is any.

The following procedures are used internally:

PROCEDURE **Register** (f: Forwarder)

Add forwarder *f* to the list of forwarders. If *f* is already registered, *Register(f)* does nothing.

Pre

f # NIL    20

PROCEDURE **Delete** (f: Forwarder)

Remove *f* from the list of forwarders. If *f *is not registered, nothing happens.



Pre

f # NIL    20

PROCEDURE **ForwardVia** (target: BOOLEAN; VAR msg: FocusMessage)

Send *msg* to either target or front focus.

PROCEDURE **SetCurrentPath** (target: BOOLEAN)

Set *path* to *target*. Must be balanced by a call to *ResetCurrentPath*, otherwise a trap will occur (in *Controllers.BalanceCheckActions.Do*).

PROCEDURE **PollSection** (VAR msg: PollSectionMsg)

Poll the current focus view's scroll state.

PROCEDURE **PollOps** (VAR msg: PollOpsMsg)

Poll the current focus view's currently valid editing operations.

PROCEDURE **PollCursor** (x, y: INTEGER; OUT cursor: INTEGER)

Poll the current focus view's currently desired cursor.

PROCEDURE **Transfer** (x, y: INTEGER; source: Views.Frame; sourceX, sourceY: INTEGER;

                                            VAR msg: TransferMessage)

PROCEDURE **PollDrop** (x, y: INTEGER; source: Views.Frame; sourceX, sourceY: INTEGER;

                                                mark, show: BOOLEAN; type: Stores.TypeName;

                                                isSingle: BOOLEAN; w, h, rx, ry: INTEGER;

                                                OUT dest: Views.Frame; OUT destX, destY: INTEGER)

PROCEDURE **Drop** (x, y: INTEGER; source: Views.Frame; sourceX, sourceY: INTEGER;

                                        view: Views.View; isSingle: BOOLEAN; w, h, rx, ry: INTEGER)

PROCEDURE **PasteView** (view: Views.View; w, h: INTEGER; clipboard: BOOLEAN)

PROCEDURE **ResetCurrentPath**

PROCEDURE **SetCurrentPath** (target: BOOLEAN)

