**TextControllers**

DEFINITION TextControllers;

    IMPORT Models, Views, Controllers, Properties, Containers, TextModels, TextViews;

    CONST

        noAutoScroll = 16; noAutoIndent = 17;

        none = -1;

    TYPE

        Controller = POINTER TO ABSTRACT RECORD (Containers.Controller)

            view-: TextViews.View;

            text-: TextModels.Model;

            (c: Controller) CaretPos (): INTEGER, ABSTRACT, NEW;

            (c: Controller) SetCaret (pos: INTEGER), ABSTRACT, NEW;

            (c: Controller) GetSelection (OUT beg, end: INTEGER), ABSTRACT, NEW;

            (c: Controller) SetSelection (beg, end: INTEGER), ABSTRACT, NEW

            (c: Controller) ThisView (): TextViews.View, EXTENSIBLE

        END;

        Directory = POINTER TO ABSTRACT RECORD (Containers.Directory)

            (d: Directory) NewController (opts: SET): Controller, ABSTRACT;

            (d: Directory) New (): Controller, EXTENSIBLE

        END;

        FilterPref = RECORD (Properties.Preference)

            controller: Controller;

            frame: Views.Frame;

            x, y: INTEGER;

            filter: BOOLEAN

        END;

        FilterPollCursorMsg = RECORD (Controllers.Message)

            controller: Controller;

            x, y: INTEGER;

            cursor: INTEGER;

            done: BOOLEAN

        END;

        FilterTrackMsg = RECORD (Controllers.Message)

            controller: Controller;

            x, y: INTEGER;

            modifiers: SET;

            done: BOOLEAN

        END;

        ModelMessage = ABSTRACT RECORD (Models.Message) END;

        SetCaretMsg = EXTENSIBLE RECORD (ModelMessage)

            pos: INTEGER

        END;

        SetSelectionMsg = EXTENSIBLE RECORD (ModelMessage)

            beg, end: INTEGER

        END;

    VAR dir-, stdDir-: Directory;

    PROCEDURE SetDir (d: Directory);

    PROCEDURE Install;

    PROCEDURE Focus (): Controller;

    PROCEDURE SetCaret (text: TextModels.Model; pos: INTEGER);

    PROCEDURE SetSelection (text: TextModels.Model; beg, end: INTEGER);

END TextControllers.

*TextControllers* are the standard controllers for text views as defined in *TextViews*.

The *caret* denotes the position where the character is inserted that the user types next. If there is text after the caret position, this text is not overwritten. Instead, the new character is inserted between the text stretches before and after the caret.

What are the attributes of a newly typed character? If the caret is at the beginning of a non-empty text, the attributes of the newly inserted character are the ones of the first character of the text. Otherwise, if no white space precedes, or if the caret is at the end of the text, the attributes of the previous character are used. Otherwise, the attributes of the next character are used. In an empty text, its default attributes are used. (There are default attributes, and an invisible default ruler, for an empty text. See the commands *Text->Make Default Attributes* and *Text->Make Default Ruler*.)

If the character is typed in when a selection exists, then the attributes of the first character of the selection are used.

To find out what attributes would be used if a character were typed in, the property mechanism can be used (see module *Properties*). Properties also allow to change these attributes, even for an empty text. (However, when the caret is set to another position, these settings are lost. This mechanism is used for the *Attributes* menu.)

CONST **noAutoScroll**

Possible element of controller option set. If included, automatic scrolling of views is disabled. Autoscrolling is used to show the caret position or to show the position of the modification performed most recently.

CONST **noAutoIndent**

Possible element of controller option set. If included, automatic indentation after entering a *line* character is disabled.

CONST **none**

Possible argument to *controller.SetCaret* and *controller.SetSelection* to indicate removal of the caret or the selection, respectively. Likewise, *controller.CarPos* and *controller.GetSelection* may return *none* to indicate the absence of a caret or selection, respectively. (Note that *controller.GetSelection* may return any pair of equal values to signal absence of a selection.)

TYPE **Controller (Containers.Controller)**

ABSTRACT

Standard controllers for text views.

**view**-: TextViews.View

The view to which the controller is connected.

**text**-: TextModels.Model    view # NIL => text = view.ThisModel()

The text displayed by the controlled view; cached for easy access.

PROCEDURE (c: Controller) **InitView** (v: Views.View)

EXTENSIBLE

Strengthened preconditions!

Pre

v = NIL  #  c.view = NIL    21

c.view = NIL

    v IS TextViews.View    22

Post

v # NIL

    c.view = v

    c.text = c.view.ThisModel()

v = NIL

    c.view = NIL

    c.text = NIL

PROCEDURE (c: Controller) **CaretPos** (): INTEGER

NEW, ABSTRACT

Current position of the caret, or *none* if not set.

Post

result = none  OR  0 <= result <= c.text.Length()

PROCEDURE (c: Controller) **SetCaret** (pos: INTEGER)

NEW, ABSTRACT

Set the caret at position *pos*, or remove the caret if *pos* = *none*.

Pre

pos = none  OR  0 <= pos    20

pos <= c.text.Length()    21

Post

c.CarPos() = pos

PROCEDURE (c: Controller) **GetSelection** (VAR beg, end: INTEGER)

NEW, ABSTRACT

Get the current selection's range [*beg*, *end*), or *beg* = *end* if no selection exists.

Post

beg = end  OR  0 <= beg <= end <= c.text.Length()

PROCEDURE (c: Controller) **SetSelection** (beg, end: INTEGER)

NEW, ABSTRACT

Set the selection to the range [*beg*, *end*), or remove the current selection if *beg* = *end*.

Pre

beg = end  OR  0 <= beg < end <= c.text.Length()    20

Post

c.GetSelection(b, e): b = beg, e = end

PROCEDURE (c: Controller) ThisView (): TextViews.View

EXTENSIBLE

Covariant extension of *Controllers.Controller.ThisView*.

TYPE **Directory (Containers.Directory)**

ABSTRACT

Directory for controllers.

PROCEDURE (d: Directory) **NewController** (opts: SET): Controller

ABSTRACT

Return new controller with options *opts*.

PROCEDURE (d: Directory) **New** (): Controller

EXTENSIBLE

Covariant narrowing of function result. Return controller with default (empty) option set.

Except for performance, equivalent to:

    **RETURN** d.NewController({})

TYPE **FilterPref (Properties.Preference)**

Used by a text controller to check for filter preferences of the view embedded in a text that is closest to but preceding the text position corresponding to the coordinates *x*, *y* in the presented *frame*. If the view wishes to filter cursor polling and tracking messages for mouse tracking events at this coordinate, it can set *filter* to TRUE. If this is done, the controller will send *FilterPollCursorMsg* messages to the view to allow it to determine the cursor icon. Also, the controller will send *FilterTrackMsg* messages to the view to allow it to intercept mouse tracking operations of the controller. An example application for this filter mechanism are hyperlink views as provided, e.g., by module *StdLinks*. These link views add hyperlinking capabilities to any text, without requiring the text controller to understand anything about hyperlinks.

**controller**: Controller

The controller asking for this preference.

**frame**: Views.Frame

The frame the controller is currently operating on.

**x**, **y**: INTEGER

The event coordinates within the current frame.

**filter**: BOOLEAN

Preset to FALSE by the controller; to be set to TRUE by views wishing to filter cursor polling and tracking operations.

TYPE **FilterPollCursorMsg (Controllers.Message)**

This message is sent by the controller to views that in response to a *FilterPref* query indicated that they wish to filter cursor polling operations. *Note*: this message is passed to the view's *HandleCtrlMsg* method together with the surrounding text view's frame! The coordinates indicated in the message are within this surrounding frame's coordinate system, not within that of the view's own frame. (In fact, the coordinates will always be outside of that view's frame - otherwise the text controller would not have ended up asking the view for filtering preferences.)

**controller**: Controller

The sending controller.

**x**, **y**: INTEGER

The coordinates (in the controller's frame) the mouse pointer is at.

**cursor**: INTEGER

The cursor icon to be displayed at this position, if any.

**done**: BOOLEAN

If set, *cursor* will override the controller's choice of cursor icon.

TYPE **FilterTrackMsg (Controllers.Message)**

This message is sent by the controller to views that in response to a *FilterPref* query indicated that they wish to filter cursor tracking operations. *Note*: this message is passed to the view's *HandleCtrlMsg* method together with the surrounding text view's frame! The coordinates indicated in the message are within this surrounding frame's coordinate system, not within that of the view's own frame. (In fact, the coordinates will always be outside of that view's frame - otherwise the text controller would not have ended up asking the view for filtering preferences.)

**controller**: Controller

The sending controller.

**x**, **y**: INTEGER

The coordinates (in the controller's frame) the mouse pointer is at.

**modifiers**: SET

The modifier keys that were active when this event originated at position *x*, *y*.

**done**: BOOLEAN

If set, the controller assumes that the view has successfully tracked this mouse click. If not, the controller defaults to its normal tracking routine.

TYPE **ModelMessage (Models.Message)**

ABSTRACT

Messages to control virtual model extensions, such as marks (e.g., caret or selection). The text system uses such messages to synchronously update marks in all views of the same model.

TYPE **SetCaretMsg (ModelMessage)**

EXTENSIBLE

Set the caret in a view displaying text model *msg.model*.

**pos**: INTEGER

Set the caret at position *pos*.

TYPE **SetSelectionMsg (ModelMessage)**

EXTENSIBLE

Set the selection in a view displaying text model *msg.model*.

**beg**, **end**: INTEGER

Set the selection to cover the stretch [*beg*, *end*).

VAR **dir**-, **stdDir**-: Directory    dir # NIL, stdDir # NIL, stable stdDir = d

Directory and standard directory objects for controllers.

PROCEDURE **SetDir** (d: Directory)

Set the directory object.

Pre

d # NIL    20

Post

dir = d

PROCEDURE **Install**

Install the current controller directory object in *TextViews*.

Except for performance, equivalent to:

    TextViews.SetCtrlDir(dir)

PROCEDURE **Focus** (): Controller

Return the text controller that currently has the focus, if any.

Except for performance, equivalent to:

    VAR v: Views.View; c: Controllers.Controller;

    v := Controllers.FocusView();

    IF (v # NIL) & (v IS TextViews.View) THEN

        c := v(TextViews.View).ThisController();

        IF (c # NIL) & (c IS Controller) THEN **RETURN** c(Controller)

        ELSE **RETURN** NIL

        END

    ELSE **RETURN** NIL

    END

PROCEDURE **SetCaret** (text: TextModels.Model; pos: INTEGER)

In all views displaying *text*, set the caret to position *pos*.

Pre

text # NIL    20

pos = none  OR  0 <= pos    21

pos <= text.Length()    22

Except for performance, equivalent to:

    VAR cm: SetCaretMsg;

    cm.pos := pos; Models.Broadcast(text, cm)

PROCEDURE **SetSelection** (text: TextModels.Model; beg, end: INTEGER)

In all views displaying *text*, set the selection to the stretch [*beg*, *end*).

Pre

text # NIL    20

beg # end

    0 <= beg    21

    beg < end    22

    end <= text.Length()    23

Except for performance, equivalent to:

    VAR sm: SetSelectionMsg;

    sm.beg := beg; sm.end := end; Models.Broadcast(text, sm)

