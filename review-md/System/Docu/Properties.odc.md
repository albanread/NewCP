**Properties**

DEFINITION Properties;

    IMPORT Fonts, Ports, Views, Controllers, Dialog;

    CONST

        color = 0; typeface = 1; size = 2; style = 3; weight = 4;

        width = 0; height = 1;

        maxVerbs = 16;

        noMark = FALSE; mark = TRUE;

        hide = FALSE; show = TRUE;

    TYPE

        Property = POINTER TO ABSTRACT RECORD

            next-: Property;

            known: SET;

            readOnly: SET;

            valid: SET;

            (p: Property) IntersectWith (q: Property; OUT equal: BOOLEAN)

        END;

        StdProp = POINTER TO RECORD (Property)

            color: Dialog.Color;

            typeface: Fonts.Typeface;

            size: INTEGER;

            style: RECORD

                        val, mask: SET

                    END;

            weight: INTEGER

        END;

        SizeProp = POINTER TO RECORD (Property)

            width, height: INTEGER

        END;

        Message = Views.PropMessage;

        PollMsg = RECORD (Message)

            prop: Property

        END;

        SetMsg = RECORD (Message)

            old, prop: Property

        END;

        Preference = ABSTRACT RECORD (Message) END;

        ResizePref = RECORD (Preference)

            fixed: BOOLEAN;

            horFitToPage: BOOLEAN;

            verFitToPage: BOOLEAN;

            horFitToWin: BOOLEAN;

            verFitToWin: BOOLEAN;

        END;

        SizePref = RECORD (Preference)

            w, h: INTEGER;

            fixedH, fixedW: BOOLEAN

        END;

        BoundsPref = RECORD (Preference)

            w, h: INTEGER

        END;

        FocusPref = RECORD (Preference)

            atLocation: BOOLEAN;

            x, y: INTEGER;

            hotFocus: BOOLEAN;

            setFocus: BOOLEAN

        END;

        ControlPref = RECORD (Preference)

            char: CHAR;

            focus: Views.View;

            getFocus: BOOLEAN;

            accepts: BOOLEAN

        END;

        PollVerbMsg = RECORD (Message)

            verb: INTEGER;

            label: ARRAY 64 OF CHAR;

            disabled, checked: BOOLEAN

        END;

        DoVerbMsg = RECORD (Message)

            verb: INTEGER;

            frame: Views.Frame

        END;

        CollectMsg = RECORD (Controllers.Message)

            poll: PollMsg

        END;

        EmitMsg = RECORD (Controllers.RequestMessage)

            set: SetMsg

        END;

        PollPickMsg = RECORD (Controllers.TransferMessage)

            mark: BOOLEAN;

            show: BOOLEAN;

            dest: Views.Frame

        END;

        PickMsg = RECORD (Controllers.TransferMessage)

            prop: Property

        END;

    VAR era-: INTEGER;

    PROCEDURE IncEra;

    PROCEDURE CollectProp (OUT prop: Property);

    PROCEDURE CollectStdProp (OUT prop: StdProp);

    PROCEDURE EmitProp (old, prop: Property);

    PROCEDURE PollPick (x, y: INTEGER; source: Views.Frame;

                                        sourceX, sourceY: INTEGER; mark, show: BOOLEAN;

                                        OUT dest: Views.Frame; OUT destX, destY: INTEGER);

    PROCEDURE Pick (x, y: INTEGER; source: Views.Frame; sourceX, sourceY: INTEGER;

                                    OUT prop: Property);

    PROCEDURE Insert (VAR list: Property; x: Property);

    PROCEDURE CopyOf (p: Property): Property;

    PROCEDURE CopyOfList (p: Property): Property;

    PROCEDURE Merge (VAR base, override: Property);

    PROCEDURE Intersect (VAR list: Property; x: Property; OUT equal: BOOLEAN);

    PROCEDURE IntersectSelections (a, aMask, b, bMask: SET; OUT c, cMask: SET;

                                                            OUT equal: BOOLEAN);

    PROCEDURE PreferredSize (v: Views.View; minW, maxW, minH, maxH,

                                                defW, defH: INTEGER; VAR w, h: INTEGER);

    PROCEDURE ProportionalConstraint (scaleW, scaleH: INTEGER;

                                                fixedW, fixedH: BOOLEAN; VAR w, h: INTEGER);

    PROCEDURE GridConstraint (gridX, gridY: INTEGER; VAR x, y: INTEGER);

    PROCEDURE ThisType (view: Views.View; type: Stores.TypeName): Views.View;

END Properties.

Practically every view has some state, and for much of that state it is desirable that the user can change it interactively. However, in an extensible system this means that for every new view at least one, or more likely several, new dialog boxes would have to be created, and would have to be learned by the user.

It is more economical for both programmer and user to have a generic and extensible way to deal with such state. Properties are provided for exactly this purpose, to communicate internal state to the user, and to communicate back any desired changes. They are the hooks for a general property editor.

Of course, some interactions are optimized for convenience, thus the most important editing operations are also accessible in a more direct way: via typing or direct manipulation with the mouse. Furthermore, combinations of often-used properties can still be handled by a custom-tailored dialog, instead of a less specific general property editor. However, except where there are standard messages already defined by lower levels of the BlackBox framework, properties are the preferred way to implement such interactions.

Properties are communicated via property messages. A particularly light-weight version of a property message is called a *Preference*: preferences inquire about a view's current preference, e.g., what its preferred size is. Preferences are normally used for the internal communication of an embedded view with its container, rather than with the user. Preferences are static message records, and a receiver should never change its state upon reception of such a message. These restrictions make preferences efficient and side-effect free, and thus easy to use.

As with all message records in BlackBox, property messages may, but need not, be handled.

CONST **color, typeface, size, style, weight**

When the focus view returns a *StdProp* property in the *CollectMsg*, the *StdProp.known*, the *StdProp.readOnly*, and the *StdProp.valid* sets may contain one or more of the above elements. If one of these elements is set in *StdProp.known*, the focus view knows about the corresponding property. If one of these elements is set in both *StdProp.valid* and *StdProp.known*, the focus view currently has an opinion about what the current value of this property is. These current values are returned in the other fields of *StdProp*. If one of these elements is set in both *StdProp.valid* and *StdProp.readOnly*, the focus view currently doesn't allow this property to be changed.

CONST **width, height**

When the focus view returns a *SizeProp* property in the *CollectMsg*, the *SizeProp.known* and the *SizeProp.valid* sets may contain one or more of these elements. If one of these elements is set in both *SizeProp.valid* and *SizeProp.known*, the *SizeProp* property contains the focus view's current width and height in the *width* and *height* fields.

CONST **noMark, mark**

These values may be passed to the *mark* parameter of procedure *PollPick*. They denote whether the target feedback during drag & pick should be drawn or not.

CONST **maxVerbs**

Maximum number of verbs in a context menu.

CONST **hide, show**

These values may be passed to the *show* parameter of procedure *PollPick*. They denote whether the target feedback during drag & pick should be shown or hidden. This parameter is only relevant if *mark* holds.

TYPE **Property**

ABSTRACT

Properties are a general mechanism to get and set attributes of a view from its environment. A view may know about attributes such as font, color, size, and arbitrary other attributes. Properties may be extended only *once*.

**next**-: Property

Properties are connected in a list, implemented by the *next* field.

**known**: SET

Each property record may describe up to 32 different attributes. These attributes are numbered from 0..31. The *know*n field tells which of these attributes are known to the view which responds to the *CollectMsg*.

**readOnly**: SET    readOnly - known = {}

Each known attribute may currently be read-only, i.e., not modifiable. The *readOnly* field tells which of the attributes are currently read-only.

The *readOnly* set must be a subset of the *known* set, i.e., no attribute can be read-only and unknown simultaneously.

**valid**: SET    valid - known = {}

Each known attribute may currently be undefined. This happens particularly when properties of multiple objects are collected together and some attributes differ from object to object (property with mixed values).

The *valid* field tells which of the attributes are currently defined. Their current values should be represented by further fields of the specific property record, e.g., field *color* in *StdProp* corresponds to element 0 in the *valid* set, field *typeface* to the element 1, etc.

The *valid* set must be a subset of the *known* set; i.e., no attribute can be valid and unknown simultaneously.

Additionally *valid* is used to specify which attributes should change if a property record is sent to an object.

PROCEDURE (p: Property) **IntersectWith** (q: Property; OUT equal: BOOLEAN)

A property record *p* must be able to compare itself with another property record *q*. If all attributes in *p* have the same values as in *q*, *equal* should be set to *TRUE*. Otherwise *p* should be set to the intersection of *p* and *q* and *equal* should be set to *FALSE*. The intersection is built by excluding all differing attributes from the valid set. It can be assumed that the type of *p* is exactly the same as the type of *q*.

*IntersectWith* must maintain the *Property* invariant that *valid* is a subset of *known*; otherwise it is not interested in *known*.

As an example, see the implementation of *StdProp.IntersectWith* below.

TYPE **StdProp (Property)**

These are the standard attributes known to any BlackBox implementation. They encompass font attributes as well as color.

**color**: Dialog.Color    valid iff constant *color IN StdProp.valid*

Current color.

**typeface**: Fonts.Typeface    valid iff constant *typeface IN StdProp.valid*

Current typeface.

**size**: INTEGER    valid iff constant *size IN StdProp.valid*

Current size. It usually, but not necessarily, refers to a font's size.

**style**: RECORD    valid iff constant *style IN StdProp.valid*

                                style IN StdProp.valid => StdProp.style.mask # {}

                                StdProp.style.val - StdProp.style.mask = {}    (* val is subset of mask *)

Current style. Field *style.mask* denotes which style flags are valid, *style.val* denotes which of the valid flags are currently set.

**weight**: INTEGER    valid iff constant *weight IN StdProp.valid*

Current font weight.

    PROCEDURE (p: StdProp) IntersectWith* (q: Property; OUT equal: BOOLEAN);

        VAR valid: SET; c, m: SET; eq: BOOLEAN;

    BEGIN

        WITH q: StdProp DO

            equal := TRUE;

            valid := p.valid * q.valid;

             IF p.color.val # q.color.val THEN EXCL(valid, color) END;

             IF p.typeface # q.typeface THEN EXCL(valid, typeface) END;

             IF p.size # q.size THEN EXCL(valid, size) END;

             IntersectSelections(p.style.val, p.style.mask, q.style.val, q.style.mask, c, m, eq);

             IF m = {} THEN

                EXCL(valid, style)

            ELSIF (style IN valid) & ~eq THEN

                p.style.mask := m; equal := FALSE

            END;

             IF p.weight # q.weight THEN EXCL(valid, weight) END;

             IF p.valid # valid THEN p.valid := valid; equal := FALSE END

        END

    END IntersectWith;

TYPE **SizeProp (Property)**

This property record represents the size of a rectangular area, e.g., of a view.

**width**: INTEGER    valid iff constant *width IN SizeProp.valid*

The current width in universal units.

**height**: INTEGER    valid iff constant *height IN SizeProp.valid*

The current height in universal units.

TYPE **Message**

ABSTRACT

Base type of all property messages.

TYPE **PollMsg (Message)**

This message is sent to get the receiving view's properties. The receiver should return the properties of *all* its contents, not only the selection.

**prop**: Property

The list of returned properties which may be modified. No property exists twice in such a list.

TYPE **SetMsg (Message)**

This message is sent to set the receiving view's properties. The properties' *known* and *readOnly* fields are not used and thus not defined in this case. The receiver should set the properties of *all* its contents, not only the selection.

**old**: Property

This list is provided for modifications of the kind "replace old by new", e.g., "replace typeface Helvetica by Times". Can be *NIL*.

**prop**: Property

The list of properties to be changed. No property may exist twice in the list.

TYPE **Preference (Message)**

ABSTRACT

Preferences are special property messages. They are normally used for the communication between an embedded view and its container. They should operate as functions, i.e. the receiver should return values, but never change its state.

TYPE **ResizePref (Preference)**

The receiver of this message can indicate that it doesn't wish to be resized, by setting *fixed* to *TRUE*. A fixed size view doesn't show resize handled when it is selected as a singleton.

For the root view in a document or window the fields *horFitToPage*, *verFitToPage*, *horFitToWin*, and *verFitToWin* can be used to enforce automatic adaptation of the view size to the actual window or page size. For embedded views, these four flags have *no* effect, in contrast to *fixed*.

**fixed**: BOOLEAN    fixed => ~horFitToPage & ~verFitToPage & ~horFitToWin & ~verFitToWin

(view => caller, preset to FALSE)

Can be set to indicate that the receiver's size should remain the same.

**horFitToPage**: BOOLEAN    horFitToPage => ~horFitToWin

(view => caller, preset to FALSE)

Can be set to indicate that the receiver's width should be adapted to the actual page width.

**verFitToPage**: BOOLEAN    verFitToPage => ~verFitToWin

(view => caller, preset to FALSE)

Can be set to indicate that the receiver's height should be adapted to the actual page height.

**horFitToWin**: BOOLEAN    horFitToWin => ~horFitToPage

(view => caller, preset to FALSE)

Can be set to indicate that the receiver's width should be adapted to the actual window width.

**verFitToWin**: BOOLEAN    verFitToWin => ~verFitToPage

(view => caller, preset to FALSE)

Can be set to indicate that the receiver's height should be adapted to the actual window height.

TYPE **SizePref (Preference)**

The sender of this message proposes a size for the receiving view; the size may be *Views.undefined*. The receiving view may override the proposal, e.g., in order to establish constraints between its width and height. The procedures *ProportionalConstraint* and *GridConstraint* are useful standard implementations of constraints.

Procedure *PreferredSize* implements the caller's side of the protocol, i.e., fills out, sends, and interprets a *SizeMsg*.

**w, h**: INTEGER    (w = Views.undefined) = (h = Views.undefined)

(view => caller, preset to *Views.undefined* if view is new or to caller's preference otherwise)

Desired width and height. Either both values are preset to *Views.undefined*, or none of them.

**fixedH, fixedW**: BOOLEAN

(caller => view)

These values are set up when the message is sent, to indicate whether height or width of the view should remain fixed. This can be used e.g. to keep a view's proportions, by adapting the width if the height is fixed, and vice versa. (See procedure *ProportionalConstraint*)

TYPE **BoundsPref (Preference)**

The receiving view should compute its bounding box, or, if this is too expensive to do, it should return an approximation of or a suggested substitute for the true bounding box.

Views that can display part of their contents, e.g., by supporting scrolling in one or both directions, will handle *BoundsPref* very differently from *SizePref*. While *SizePref* relates to constraints and preferences of the size of the current view onto the model, *BoundsPref* should compute the size the view would need to take to just fully display all of the model. For very large or complex models (such as uncast text running over many pages) this can be a very costly operation and therefore it is anticipated that views may return an approximation of a bounding size, or an alternate suggestion, such as a size that is "quite big" but no bigger than the model.

**w, h**: INTEGER

(view => caller, preset to *Views.undefined*)

Preferred width and height.

TYPE **FocusPref (Preference)**

A view can indicate if it doesn't want to become focus permanently, e.g., a button. It may even decide so depending on where the mouse has been clicked.

**atLocation**: BOOLEAN

(caller => view)

This flag is set if the receiver would become focus through a mouse click.

**x, y**: INTEGER    [units]

(caller => view, valid iff atLocation)

Position of mouse click relative to the receiving view's top-left corner.

**hotFocus**: BOOLEAN

(view => caller, preset to FALSE)

The receiver can set this flag to indicate that the view should release its focus immediately after the mouse is released. Command buttons are typical hot foci.

**setFocus**: BOOLEAN

(view => caller, preset to FALSE)

The receiver can set this flag to indicate that the view should become focus when the mouse is clicked inside (otherwise the view may become selected immediately, e.g., to be dragged & dropped). *setFocus* should be set for all true editors, such that context-sensitive menu commands can be attached to the view.

TYPE **ControlPref (Preference)**

A view can indicate its preferred behavior if it is embedded in a dialog (i.e., a container in mask mode, -> Containers) and a key is pressed.

The *ControlPref* message is sent first to the actual focus view and then to all other views in the container until one of the views sets either the *getFocus* or the *accepts* bit or both. If none of the views respond to the message, the character is forwarded to the focus view.

**char**: CHAR

(caller => view)

The key which was pressed.

**focus**: Views.View

(caller => view)

The actual focus view.

**getFocus**: BOOLEAN

(view => caller, valid if (view # focus), preset to (char = tab) )

Indicates that the view wants to become the focus view.

**accepts**: BOOLEAN

(view => caller, preset to ((view = focus) & (char # tab)) )

This flag should be set if the receiver would accept and interpret *char* to invoke some action.

The character is sent to the view in a separate *Edit* message.

TYPE **PollVerbMsg (Message)**

Message used to ask a view whether it supports custom verbs. A verb is some kind of action that the user can apply to a view and that is shown in a popup-menu.

**verb**: INTEGER    verb >= 0    (IN)

(caller => view)

The index of the verb to be polled.

**label**: ARRAY 64 OF CHAR

(view => caller, preset to "")

Displayed label of the verb.

**disabled**: BOOLEAN

(view => caller, preset to FALSE)

Indicates whether verb is enabled or disabled.

**checked**: BOOLEAN

(view => caller, preset to FALSE)

Indicates whether verb is unchecked or checked.

TYPE **DoVerbMsg (Message)**

Execute a particular custom verb.

**verb**: INTEGER

(caller => view)

The verb to be executed.

**frame**: Views.Frame

(caller => verb)

The frame of the view where the verb was invoked by the user.

TYPE **CollectMsg (Controllers.Message)**

This controller message is sent along the focus path, to poll the properties of the innermost focus view's selection or caret, i.e., unlike *PollMsg* it does not return the properties of the whole contents.

**poll**: PollMsg

TYPE **EmitMsg (Controllers.RequestMessage)**

This controller message is sent along the focus path, to set the properties of the innermost focus view's selection or caret, i.e., unlike *SetMsg* it does not set the properties of the whole contents.

**set**: SetMsg

TYPE **PollPickMsg (Controllers.TransferMessage)**

While an item is being dragged around for the purpose of picking properties (BlackBox's Drag & Pick mechanism), *PollPickMsgs* are sent to enable feedback about the pick target. Note that this is similar to the role of *Controllers.PollDropMsg* for Drag & Drop.

**mark**: BOOLEAN

A container which supports pick feedback should show (hide) its feedback mark when mark is set (cleared). You don't need to deal with the view's border mark (the rectangular outline of the view which is drawn while you are dragging over a view), this is handled completely by BlackBox itself through its container abstraction.

**show**: BOOLEAN

Indicates whether the mark should be drawn or removed.

**dest**: Views.Frame

The receiver should set dest to its own frame, if it would accept a pick.

TYPE **PickMsg (Controllers.TransferMessage)**

Extension

This message is used if properties are to be picked from the cursor's location. Note that this is similar to the *Controllers.DropMsg* for Drag & Drop.

**prop**: Property

The receiver should set prop to the properties at the cursor's location.

VAR **era**-: INTEGER    propEra >= 0

This variable is used by BlackBox to determine whether the focus view's properties may have changed.

PROCEDURE **IncEra**

Increments *era*. This procedure should be called whenever one or several properties of a view have changed. As a response, the system will eventually send a *PollMsg* to get the new properties, e.g., to reflect them in the menus or in a property editor.

Post

era = era' + 1

PROCEDURE **CollectProp** (OUT prop: Property)

Poll focus view's properties.

PROCEDURE **CollectStdProp** (OUT prop: StdProp)

Poll focus view's standard properties.

PROCEDURE **EmitProp** (old, prop: Property)

Set focus view's properties to *prop*. If *old* is not *NIL*, only the properties corresponding to *old* are replaced by their *prop* counterparts.

PROCEDURE **PollPick** (x, y: INTEGER; source: Views.Frame;

                                        sourceX, sourceY: INTEGER; mark, show: BOOLEAN;

                                        OUT dest: Views.Frame; OUT destX, destY: INTEGER)

Control pick feedback at location *(x, y)* relative to the coordinates of source, by sending a *PollPickMsg* *msg*. Presets *msg.dest* to *NIL*, *msg.mark* to *mark*, *msg.show* to *show*, and calls *Controllers.Transfer(x, y, source, sourceX, sourceY, msg)*. Returns *msg.dest*, *msg.destX*, and *msg.destY* in *dest*, *destX*, and *destY*, respectively.



Pre

source # NIL   20

PROCEDURE **Pick** (x, y: INTEGER; source: Views.Frame;

                                sourceX, sourceY: INTEGER; OUT prop: Property)

Pick properties at location *(x, y)* relative to the coordinates of source, by sending a *PickMsg* *msg*. Presets *msg.prop* to *NIL* and calls *Controllers.Transfer(x, y, source, sourceX, sourceY, msg)*. Returns *msg.prop* in *prop*.

PROCEDURE **Insert** (VAR list: Property; x: Property)

Insert new property record *x* in list *list*. A property whose type is already in the list replaces the property in the list. The dynamic type of *x* must be a direct extension of type *Property*.

Pre

x # NIL   20

x.next = NIL   21

x # list   22

x.valid - x.known = {}   23

list # NIL

   list.valid - list.known = {}   24

   extension-level(list) = 1   25

extension-level(x) = 1   26

PROCEDURE **CopyOf** (p: Property): Property

Returns a copy of the property *p *(not of the entire list!). The dynamic types of every property in*p* must be a direct extension of type *Property*.

Pre

extension-level(p) = 1   20

PROCEDURE **CopyOfList** (p: Property): Property

Returns a copy of the property list *p*. The dynamic types of every property in*p* must be a direct extension of type *Property*.

Pre

all x in {p, p.next, p.next.next, ...}: extension-level(x) = 1   20

PROCEDURE **Merge** (VAR base, override: Property)

Merge two property lists *base* and *override*. If the type of a property is in both lists, the attributes of the property in *override* will be selected for the merged list. The merged list is returned in *base*.

PROCEDURE **Intersect** (VAR list: Property; x: Property; OUT same: BOOLEAN)

It reduces the properties in *list* by all these properties which are not in *x*, or which have different values in *x*. Furthermore, it determines whether both lists are the same.

PROCEDURE **IntersectSelections** (a, aMask, b, bMask: SET; OUT c, cMask: SET;

                                                                    OUT equal: BOOLEAN)

Support procedure to implement the *IntersectWith* procedure of properties containing a set. This procedure is equivalent to the following code:

    cMask := aMask * bMask - (a / b);

    c := a * cMask;

    equal  := (aMask = bMask) & (bMask = cMask)

PROCEDURE **PreferredSize** (v: Views.View; minW, maxW, minH, maxH,

                                                    defW, defH: INTEGER; VAR w, h: INTEGER)

Sets up a *SizePref* and uses it to ask view *v* for its sizing preferences.

*[minW, maxW]* and *[minH, maxH]* are the legal ranges of width and height, respectively, that are acceptable to the caller. *(defW, defH)* is the caller-specified default size, i.e., the size that the caller prefers in the absence of any preferences of *v*; specifying *(Views.undefined, Views.undefined)* means "no default". *(w, h)* can be preset by the caller and will be used to preset the *SizePref*; normally the caller uses this to pass the current size of a view that is to be resized.

However, if *(w < Views.undefined)* or *(w > maxW)*, then *defW* will be used instead of *w*. Likewise, if *(h < Views.undefined)* or *(h > maxH)*, then *defH* will be used instead of *h*.

If *v* does not interpret a *SizePref* or returns *w = Views.undefined*, then *defW* overrides the preference of *v*. Likewise, if the *SizePref* returns *h = Views.undefined*, then *defH* overrides the preference of *v*.

Finally, *(w, h)* will be clipped to *([minW, maxW], [minH, maxH])* before returning to the caller. Therefore, if *v* has no (or no valid) preferences and the caller-specified default (or preset) value is *Views.undefined*, then the minimum values will be returned.



Pre

Views.undefined < minW   20

minW < maxW   21

Views.undefined < minH   23

minH < maxH   24

Views.undefined <= defW   26

Views.undefiend <= defH   28

PROCEDURE **ProportionalConstraint** (scaleW, scaleH: INTEGER;

                                                    fixedW, fixedH: BOOLEAN; VAR w, h: INTEGER)

Supports proportional resizing of rectangular shapes such as views. *(scaleW, scaleH)* is the size of a rectangle of the required proportions, e.g., *(2,3)* represents all rectangles having a ratio width to height of 2:3. *(w, h)* is the proposed size and *ProportionalConstraint* modifies this pair to satisfy the proportionality constraint. Normally, *ProportionalConstraint* performs its function by changing both, *w* and *h*, such that the resulting rectangle satisfies the constraint and that the new rectangle's area is as close to the old rectangle's area as possible.  By setting *fixedW*, the constraint is satisfied by only changing *h*.  Likewise, setting *fixedH* asks for at most changing *w*. When both, *fixedW* and *fixedH*, are set, then this is ignored and again the area-preserving heuristics is used.



Pre

w > Views.undefined   20

h > Views.undefined   21

scaleW > Views.undefined   22

scaleH > Views.undefined   23

~fixedW OR ~fixedH    24

PROCEDURE **GridConstraint** (gridX, gridY: INTEGER; VAR x, y: INTEGER)

Supports forcing the coordinates of a point, such as one of the corners of a view, to the nearest point of a grid. *(gridX, gridY)* specifies the resolution of the grid in *x* and *y* direction, e.g., *(5,3)* specifies a grid with valid coordinates being multiples of *5* in the *x* and multiples of *3* in the *y* direction.



Pre

gridX > Views.undefined   20

gridY > Views.undefined   21

