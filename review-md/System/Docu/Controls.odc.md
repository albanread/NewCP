**Controls**

DEFINITION Controls;

    IMPORT Meta, Dialog, Views, Properties;

    CONST

        opt0 = 0; opt1 = 1; opt2 = 2; opt3 = 3; opt4 = 4;

        link = 5; label = 6; guard = 7; notifier = 8; level = 9;

        default = opt0; cancel = opt1;

        left = opt0; right = opt1; multiLine = opt2; password = opt3;

        sorted = opt0;

        haslines = opt1; hasbuttons = opt2; atroot = opt3; foldericons = opt4;

    TYPE

        Control = POINTER TO ABSTRACT RECORD (Views.View)

            item-: Meta.Item;

            disabled-, undef-, readOnly-, customFont-: BOOLEAN;

            font-: Fonts.Font;

            label-: Dialog.String;

            prop-: Prop;

            (c: Control) Internalize- (VAR rd: Stores.Reader);

            (c: Control) Externalize- (VAR wr: Stores.Writer);

            (c: Control) CopyFromSimpleView- (source: Views.View);

            (c: Control) HandleViewMsg- (f: Views.Frame; VAR msg: Views.Message);

            (c: Control) HandleCtrlMsg (f: Views.Frame; VAR msg: Views.CtrlMessage;

                                                    VAR focus: Views.View);

            (c: Control) HandlePropMsg- (VAR msg: Views.PropMessage);

            (c: Control) Internalize2- (VAR rd: Stores.Reader), NEW, EMPTY;

            (c: Control) Externalize2- (VAR wr: Stores.Writer), NEW, EMPTY;

            (c: Control) CopyFromSimpleView2- (source: Control), NEW, EMPTY;

            (c: Control) HandleViewMsg2- (f: Views.Frame; VAR msg: Views.Message), NEW, EMPTY;

            (c: Control) HandleCtrlMsg2- (f: Views.Frame; VAR msg: Views.CtrlMessage;

                                                    VAR focus: Views.View), NEW, EMPTY;

            (c: Control) HandlePropMsg2- (VAR p: Views.PropMessage), NEW, EMPTY;

            (c: Control) CheckLink- (VAR ok: BOOLEAN), NEW, EMPTY;

            (c: Control) Update- (f: Views.Frame; op, from, to: INTEGER), NEW, EMPTY;

            (c: Control) UpdateList- (f: Views.Frame), NEW, EMPTY

        END;

        Directory = POINTER TO ABSTRACT RECORD

            (d: Directory) NewPushButton (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewCheckBox (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewRadioButton (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewListBox (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewSelectionBox (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewField (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewUpDownField (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewDateField (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewTimeField (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewTreeControl (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewColorField (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewComboBox (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewCaption (p: Prop): Control, NEW, ABSTRACT;

            (d: Directory) NewGroup (p: Prop): Control, NEW, ABSTRACT

        END;

        Prop = POINTER TO RECORD (Properties.Property)

            opt: ARRAY 5 OF BOOLEAN;

            link, label, guard, notifier: Dialog.String;

            level: INTEGER;

            (p: Prop) IntersectWith (q: Properties.Property; OUT equal: BOOLEAN)

        END;

        DefaultsPref = RECORD (Properties.Preference)

            disabled, undef, readOnly: BOOLEAN

        END;

        PropPref = RECORD (Properties.Preference)

            valid: SET

        END;

    VAR

        dir-, stdDir-: Directory;

        par-: Views.View;

    PROCEDURE Notify (c: Control; f: Views.Frame; op, from, to: INTEGER);

    PROCEDURE OpenLink (c: Control; p: Prop);

    PROCEDURE Relink;

    PROCEDURE DepositPushButton;

    PROCEDURE DepositCheckBox;

    PROCEDURE DepositRadioButton;

    PROCEDURE DepositListBox;

    PROCEDURE DepositSelectionBox;

    PROCEDURE DepositField;

    PROCEDURE DepositUpDownField;

    PROCEDURE DepositDateField;

    PROCEDURE DepositTimeField;

    PROCEDURE DepositTreeControl;

    PROCEDURE DepositColorField;

    PROCEDURE DepositComboBox;

    PROCEDURE DepositCaption;

    PROCEDURE DepositGroup;

    PROCEDURE DepositCancelButton;

    PROCEDURE SetDir (d: Directory);

END Controls.

Module *Controls* provides a variety of standard user interface elements, so-called *controls*. In BlackBox, a control is an extended view. As every view, a control can be embedded in any general container (-> Containers), such as forms (-> FormModels) but also texts (-> TextModels). Usually, controls are put into forms.

The standard controls provided by BlackBox are: command buttons (push buttons), check boxes, radio buttons, list boxes, selection boxes, (text, date, time and color) fields, combo boxes, tree controls, captions, and groups.

Unlike other views, these BlackBox controls can be *linked* to a program variable, or more exactly: to any field accessible through a globally declared variable. When the control is opened, BlackBox tries to link the control to its variable, using the advanced metaprogramming capabilities of the BlackBox *Meta* module. In this way, the link between control and variable can be built up automatically when a dialog is newly created or loaded from a file, and correct linking (i.e., correct typing) can be guaranteed even after a dialog layout had been edited or otherwise manipulated.

Controls may take on different states at run-time. Depending on the control and on the underlying user interface, these states may be represented in visually distinct ways:

enabled/disabled

    Only enabled controls may be modified interactively.

    To disable a control, its guard should set *par.disabled* to *TRUE*.

    A control which cannot be linked to its variable is always disabled.

defined/undefined

    Illegal or otherwise undefined values may be hilighted as such.

    To mark a control as undefined, its guard should set *par.undef* to *TRUE*.

normal/read-only

    Controls may be denoted as read-only, i.e., under program control only.

    To mark a control as read-only, its guard should set *par.readOnly* to *TRUE*.

    Controls which are linked to read-only variables are always read-only.

Note that the above states are temporary, i.e., determined wholly at run-time; they are never externalized or internalized.

All controls have the following persistent properties, which are externalized when the controls are written to files:

link

    This is the name of a global variable, to which the control is linked, e.g.,

*    TextCmds.find.replace*.

label

    This is the displayed string.

    (not applicable to (text, date, time and color) fields, list boxes, selection boxes, combo boxes)

    A "&" character indicates which character of the label should be underlined

    (For the keyboard shortcut. This is a Windows feature, and not available on Mac OS).

    If you want a "&" to appear, you should write a "&&".

guard

    This optional command name denotes a guard procedure which allows to disable/enable a

    control selectively, and to set the undef and readOnly states as well.

notifier

    This optional command name denotes a notifier procedure which allows to do something

    whenever the value of a control was changed interactively.

light font

    A field may either be displayed in the standard font for this purpose, or in a more

    discreet ("light") font. On some platforms, this property may be interpreted only in fields

    and captions.

These properties, as well as further control-specific properties (see below) can be set interactively using a suitable tool (-> DevInspector), or programmatically using the *Properties.SetMsg*.

**Command Button**

Pressing a command button invokes a parameterless exported Component Pascal procedure. A command button is either linked to a parameterless exported procedure. It can be either a constant (a normal procedure) or a procedure variable.

Additional properties:

default    The button is activated when the user presses *return* or *enter*.

cancel    The button is activated when the user presses *escape*.

**Check Box**

A check box lets the user toggle between two states, checked and unchecked. A check box is linked to a *BOOLEAN* field or variable.

Alternatively, a check box can also be linked to a *SET* field or variable. The *level* property then specifies to which entry of the set the control is linked to. The control is checked if the inspected element is in the set. A check box can also be linked to a *Dialog.Selection* field or variable. The *level* property then indicates which element of the *val* set of the selection is inspected.

Additional property:

level    if a check box is linked to a set or to a *Dialog.Selection* variable, then the level field

    indicates which set element is displayed by the control.

**Radio Button**

Radio buttons let the user choose between several alternatives. Each alternative is represented as a radio button. At any time, exactly one of the radio buttons is "on", while all others are "off".

All radio buttons which belong to one selection are linked to the same integer field. Each radio button is "on" for another value of the integer field. This value can be configured with the *level* property.

Additional property:

level    a radio button is "on" when the value of the variable to which the button is

    bound is equal to the level value.

**List Box**

A list box presents a list of strings to the user. From this list, at most one entry can be selected. List boxes are linked to a *Dialog.List*.

**Selection Box**

A selection box presents a list of strings to the user. From this list, an arbitrary number of entries can be selected. Selection boxes are linked to a *Dialog.Selection*.

**Field**

A text field lets the user type in a string. Fields are either linked to *ARRAY n OF CHAR, BYTE, SHORTINT, INTEGER, LONGINT, SHORTREAL, REAL, Dialog.Combo* or *Dialog.Currency* fields. Fields perform some basic checks, such as for the maximum permissible string length, or for the permissible character set (for numbers, only digits and a few characters like "-" or "." make sense).

Additional property:

multi line    The field linked to *ARRAY n OF CHAR* may display several lines.

    A carriage return character is accepted on input.

password    Don't display the characters that have been typed in.

**UpDownField**

The *UpDownField* is a special text field linked to a *BYTE, SHORTINT, INTEGER* or *LONGINT* variable. The value of the integer can be incremented and decremented through arrows.

**DateField**

A date field lets the user specify a date. Date fields are linked to variables of type *Dates.Date*. On input, date fields only allow valid dates.

**Time Field**

A time field lets the user specify a time. Time fields are linked to variables of type *Dates.Time*. On input, time fields only allow valid times.

**Tree Control**

A tree control presents a tree of strings to the user. From this tree, at most one node can be selected. Tree Controls are linked to a *Dialog.Tree*.

Additional properties:

Sorted    Nodes with the same parent are displayed in alphabetical order

    rather than in the order they were added to the tree.

Lines    Lines are drawn between nodes in the tree.

Buttons    A button with a "+" or a "-" sign is drawn in front of nodes that have children.

Lines/Buttons at Root    Lines and buttons are drawn also at the root level in the tree.

Folder Icons    Icons are used to show nodes as folders or leafs.

**Color Field**

A color field lets the user specify a color. Color fields are linked to variables of type *Dialog.Color* or of type *Ports.Color*.

**Combo Box**

A combo box is a combination of a text field with a list box. Like a list box, it presents a list of strings from which can be chosen. Additionally, it provides a field in which a value may be typed in. This value may be one in the list, or another value altogether. Combo boxes are linked to a *Dialog.Combo*.

**Caption**

A caption is a static string, which cannot be manipulated by the user. Typically, a field is accompanied by a caption which tells what the meaning of the field is. Captions may either be unlinked, or linked to the same record field as their corresponding field. Captions are not completely passive in that they may be enabled or disabled (e.g., "greyed out") as other controls.

**Group**

A group is a rectangular frame with a label in it. It allows to logically group several controls together.

Guard commands can be associated with a control. Such a command has the following signature:

    PROCEDURE Guard (VAR par: Dialog.Par)

The main purpose of a guard command is to disable a control when necessary. This is done in the following way:

    PROCEDURE Guard (VAR par: Dialog.Par);

    BEGIN

        par.disabled := *condition*

    END Guard;

Controls which are always enabled need no guard. Besides disabling a control, guards may also set a control to an undefined or read-only state (*par.undef*, *par.readOnly*).

Notifier commands can be associated with a control. Such a command has the signature of *Dialog.NotifierProc*, i.e.,

    PROCEDURE Notifier (op, from, to: INTEGER)

The purpose of a notifier command is to give a program the opportunity to customize the behavior of a control when its state is being modified by the user. For example, a status message may be shown when the user clicks onto a command button, and remove again when the user releases the button again (this can be done using the *Dialog.ShowStatus* procedure). Or, a notifier may perform some checks after each character typed into a text field. Another example are selection boxes: for each item / item range which is selected/deselected, a notification is produced. This makes it possible to e.g. update the count of currently selected items of this selection box.

Examples:

[<u>ObxControls  docu</u>](../../Obx/Docu/Controls.odc.md)    the use of guards and notifiers

[<u>ObxDialog  docu</u>](../../Obx/Docu/Dialog.odc.md)    the use of selection boxes and combo boxes

[<u>ObxButtons docu</u>](../../Obx/Docu/Buttons.odc.md)    control not extended from Controls.Control - shows how to handle control properties

[<u>ObxCtrls</u>](../../Obx/Mod/Ctrls.odc.md)    slider control, extended from Controls.Control

[<u>ObxFldCtrls</u>](../../Obx/Mod/FldCtrls.odc.md)    special-purpose text field control, extended from Controls.Control

CONST **opt0, opt1, opt2, opt3, opt4**

Elements of a control property's *valid* set. They determine whether their corresponding *optN* fields are valid.

CONST **link**

Element of a control property's *valid* set. It determines whether the *link* field is valid.

CONST **label**

Element of a control property's *valid* set. It determines whether the *label* field is valid.

CONST **guard**

Element of a control property's *valid* set. It determines whether the *guard* field is valid.

CONST **notifier**

Element of a control property's *valid* set. It determines whether the *notifier* field is valid.

CONST **level**

Element of a control property's *valid* set. It determines whether the *level* field is valid.

CONST **default, cancel**

Aliases of *opt0* and *opt1*, used for command buttons.

CONST **left, right, multiLine, password**

Aliases of *opt0* to *opt3* used for text entry fields

CONST **sorted**

Alias of *opt0* used for list-structured controls (list/selection/combo boxes/tree control).

CONST **haslines**

Alias of *opt1* used for tree control.

CONST **hasbuttons**

Alias of *opt2* used for tree control.

CONST **atroot**

Alias of *opt3* used for tree control.

CONST **foldericons**

Alias of *opt4* used for tree control.

TYPE **Control (Views.View)**

ABSTRACT

Base type for controls that can be linked to global variables or procedures via metaprogramming. Such a control has no separate model, the item to which it is linked takes the role of a model.

**item**-: Meta.Item

This item describes the variable or procedure to which the control is linked.

**disabled**-, **undef**-, **readOnly**-: BOOLEAN

The current temporary state of any control.

**customFont**-: BOOLEAN

This flag determines whether the control's font is taken from the operating system configuration, or from the *font* variable below.

**font**-: Fonts.Font    font # NIL -> customFont

The control's custom font (*customFont* must be *TRUE* when *font* is used, i.e., when it is not *NIL*).

**label**-: Dialog.String

The control's label.

**prop**-: Prop    prop # NIL

The control attributes of this control.

PROCEDURE (c: Control) **Internalize2**- (VAR rd: Stores.Reader), NEW, EMPTY;

PROCEDURE (c: Control) **Externalize2**- (VAR wr: Stores.Writer), NEW, EMPTY;

PROCEDURE (c: Control) **CopyFromSimpleView2**- (source: Control), NEW, EMPTY;

PROCEDURE (c: Control) **HandleViewMsg2**- (f: Views.Frame; VAR msg: Views.Message)

PROCEDURE (c: Control) **HandleCtrlMsg2**- (f: Views.Frame; VAR msg: Views.CtrlMessage;

                                                                    VAR focus: Views.View)

PROCEDURE (c: Control) **HandlePropMsg2**- (VAR p: Views.PropMessage)

NEW, EMPTY

Extension hooks for *Internalize*, *Externalize*, *CopyFromSimpleView, HandleCtrlMsg, HandlePropMsg, HandlePropMsg*. These methods are made final, and call their new empty extension hooks.

PROCEDURE (c: Control) **CheckLink**- (VAR ok: BOOLEAN)

NEW, EMPTY

This hook procedure allows a control to check whether the item to which it is linked is acceptable. For example, a checkbox control would accept a link to a Boolean item, but not to an integer item.

PROCEDURE (c: Control) **Update**- (f: Views.Frame; op, from, to: INTEGER)

NEW, EMPTY

Update the display of a control. For list-structured controls, the list is not changed.

PROCEDURE (c: Control) **UpdateList**- (f: Views.Frame)

NEW, EMPTY

Update the display of a list-structured control, including a rebuilding of its list elements.

TYPE **Directory**

ABSTRACT

Directory type for standard controls.

PROCEDURE (d: Directory) **NewPushButton** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new command button.

PROCEDURE (d: Directory) **NewCheckBox** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new check box.

PROCEDURE (d: Directory) **NewRadioButton** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new radio button.

PROCEDURE (d: Directory) **NewListBox** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new list box.

PROCEDURE (d: Directory) **NewSelectionBox** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new selection box.

PROCEDURE (d: Directory) **NewField** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new text field.

PROCEDURE (d: Directory) **NewUpDownField** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new text field with up and down arrows.

PROCEDURE (d: Directory) **NewDateField** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new date field.

PROCEDURE (d: Directory) **NewTimeField** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new time field.

PROCEDURE (d: Directory) **NewTreeControl** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new tree control.

PROCEDURE (d: Directory) **NewColorField** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new color field.

PROCEDURE (d: Directory) **NewComboBox** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new combo box.

PROCEDURE (d: Directory) **NewCaption** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new text caption.

PROCEDURE (d: Directory) **NewGroup** (p: Prop): Control

NEW, ABSTRACT

Allocates and returns a new group.

TYPE **Prop**

ABSTRACT

A property object describes various attributes of a control. There are messages which allow to poll or set the control properties of a control (-> Properties, -> Controllers).

**opt**: ARRAY 5 OF BOOLEAN;

Up to five Boolean options. Their meaning depends on the control's type.

**link**: Dialog.String

Name of the field or procedure to which control is linked, e.g., "TextCmds.find.replace".

**label**: Dialog.String

String label of the control. Ignored for fields, list boxes, selection boxes, combo boxes and tree controls.

**guard**: Dialog.String

Optional name of guard command.

**notifier**: Dialog.String

Optional name of notifier command.

**level**: INTEGER    only valid for radio buttons.

Determines for which value of the bound variable the control is "on".

TYPE **DefaultsPref (Properties.Preference)**

This preference message allows to override the general defaults of a control's temporary state.

**disabled, undef, readOnly**: BOOLEAN

State of the control. It is being determined in three steps:

1) general default:

    disabled := ~c.item.Valid()

    undef := FALSE

    readOnly := c.item.vis = Meta.readOnly

2) control-specific default:

    by handling the *DefaultsPref* message, the general default can be overwritten.

    For example, with buttons: disabled := link = ""

3) guard call:

    If there is a guard, the guard result is used, otherwise the default.

TYPE **PropPref (Properties.Preference)**

Preference message which allows to specify the valid properties of the receiving control.

{link, label, guard, notifier, customFont} is the default. By handling the *PropPref* message, elements of the valid set can be added or removed.

**valid**: SET

VAR **dir-, stdDir-**: Directory    dir # NIL  &  stdDir # NIL

Directories for the lookup of standard controls.

VAR **par-**: Control

Before a control's guard or notifier procedure is called, and before any controller message is sent to it, the control is assigned to *par*. Afterwards, it is reset to its previous value. This variable can be used in a command to determine the currently active control.

PROCEDURE **Notify** (c: Control; f: Views.Frame; op, from, to: INTEGER)

Used internally.

PROCEDURE **OpenLink** (c: Control; p: Prop)

Try to link control *c* according to control property *p*.

Pre

c # NIL    20

p # NIL    21

PROCEDURE **Relink**

Force a re-evaluation of the control's link.

PROCEDURE  **DepositPushButton**

Allocate a new command button using *dir.NewPushButton* and deposit it.

PROCEDURE  **DepositCheckBox**

Allocate a new check box using *dir.NewCheckBox* and deposit it.

PROCEDURE  **DepositRadioButton**

Allocate a new command button using *dir.NewPushButton* and deposit it.

PROCEDURE  **DepositListBox**

Allocate a new radio button using *dir.NewRadioButton* and deposit it.

PROCEDURE  **DepositSelectionBox**

Allocate a new selection box using *dir.NewSelectionBox* and deposit it.

PROCEDURE  **DepositField**

Allocate a new field using *dir.NewField* and deposit it.

PROCEDURE  **DepositUpDownField**

Allocate a new up-down-field using *dir.NewUpDownField* and deposit it.

PROCEDURE  **DepositDateField**

Allocate a new field using *dir.NewDateField* and deposit it.

PROCEDURE  **DepositTimeField**

Allocate a new field using *dir.NewTimeField* and deposit it.

PROCEDURE **DepositTreeControl**

Allocate a new tree control using *dir.NewTreeControl* and deposit it.

PROCEDURE  **DepositColorField**

Allocate a new field using *dir.NewColorField* and deposit it.

PROCEDURE  **DepositComboBox**

Allocate a new combo box using *dir.NewComboBox* and deposit it.

PROCEDURE  **DepositCaption**

Allocate a new caption using *dir.NewCaption* and deposit it.

PROCEDURE  **DepositGroup**

Allocate a new group using *dir.NewGroup* and deposit it.

PROCEDURE  **DepositCancelButton**

Allocate a new command button using *dir.NewPushButton* and deposit it. The button's link property is initialized to "StdCmds.CloseDialog" and its cancel-property is set to *TRUE*.

PROCEDURE  **SetDir** (d: Directory)

Assigns directory.

*SetDir* is used in configuration routines.

Pre

d # NIL    20

Post

stdDir' = NIL

    stdDir = d

stdDir' # NIL

    stdDir = stdDir'

dir = d

