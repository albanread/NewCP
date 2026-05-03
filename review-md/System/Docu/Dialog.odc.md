**Dialog**

DEFINITION Dialog;

    IMPORT Files;

    CONST

        pressed = 1; released = 2; changed = 3; included = 5; excluded = 6; set = 7;

        ok = 1; yes = 2; no = 3; cancel = 4;

        windows32s = 11; windows95 = 12; windowsNT3 = 13; windowsNT4 = 14; windows2000 = 15;

        windows98 = 16; windowsXP = 17; windowsVista = 18;

        macOS = 21; macOSX = 22; linux = 30; tru64 = 40;

        firstPos = 0; lastPos = -1;

        persistent = TRUE; nonPersistent = FALSE;

    TYPE

        String = ARRAY 256 OF CHAR;

        List = RECORD

            index, len-: INTEGER;

            (VAR l: List) GetItem (index: INTEGER; OUT item: String), NEW;

            (VAR l: List) SetItem (index: INTEGER; IN item: ARRAY OF CHAR), NEW;

            (VAR l: List) SetLen (len: INTEGER), NEW;

            (VAR l: List) SetResources (IN key: ARRAY OF CHAR), NEW

        END;

        Selection = RECORD

            len-: INTEGER;

            (VAR s: Selection) Excl (from, to: INTEGER), NEW;

            (VAR s: Selection) GetItem (index: INTEGER; OUT item: String), NEW;

            (VAR s: Selection) In (index: INTEGER): BOOLEAN, NEW;

            (VAR s: Selection) Incl (from, to: INTEGER), NEW;

            (VAR s: Selection) SetItem (index: INTEGER; IN item: ARRAY OF CHAR), NEW;

            (VAR s: Selection) SetLen (len: INTEGER), NEW;

            (VAR s: Selection) SetResources (IN key: ARRAY OF CHAR), NEW

        END;

        Combo = RECORD

            item: String;

            len-: INTEGER;

            (VAR c: Combo) GetItem (index: INTEGER; OUT item: String), NEW;

            (VAR c: Combo) SetItem (index: INTEGER; IN item: ARRAY OF CHAR), NEW;

            (VAR c: Combo) SetLen (len: INTEGER), NEW;

            (VAR c: Combo) SetResources (IN key: ARRAY OF CHAR), NEW

        END;



        TreeNode = POINTER TO LIMITED RECORD

            (tn: TreeNode) Data (): ANYPTR, NEW;

            (tn: TreeNode) GetName (OUT name: String), NEW;

            (tn: TreeNode) IsExpanded (): BOOLEAN, NEW;

            (tn: TreeNode) IsFolder (): BOOLEAN, NEW;

            (tn: TreeNode) NofChildren (): INTEGER, NEW;

            (tn: TreeNode) SetData (data: ANYPTR), NEW;

            (tn: TreeNode) SetExpansion (expanded: BOOLEAN), NEW;

            (tn: TreeNode) SetName (name: String), NEW;

            (tn: TreeNode) ViewAsFolder (isFolder: BOOLEAN), NEW

        END;

        Tree = RECORD

            (VAR t: Tree) Child (node: TreeNode; pos: INTEGER): TreeNode, NEW;

            (VAR t: Tree) Delete (node: TreeNode): INTEGER, NEW;

            (VAR t: Tree) DeleteAll, NEW;

            (VAR t: Tree) Move (node, parent: TreeNode; pos: INTEGER), NEW;

            (VAR t: Tree) NewChild (parent: TreeNode; pos: INTEGER; name: String): TreeNode, NEW;

            (VAR t: Tree) Next (node: TreeNode): TreeNode, NEW;

            (VAR t: Tree) NofNodes (): INTEGER, NEW;

            (VAR t: Tree) NofRoots (): INTEGER, NEW;

            (VAR t: Tree) Parent (node: TreeNode): TreeNode, NEW;

            (VAR t: Tree) Prev (node: TreeNode): TreeNode, NEW;

            (VAR t: Tree) Select (node: TreeNode), NEW;

            (VAR t: Tree) Selected (): TreeNode, NEW

        END;

        Color = RECORD

            val: INTEGER

        END;

        Currency = RECORD

            val: LONGINT;

            scale: INTEGER

        END;

        Par = RECORD

            disabled: BOOLEAN;

            checked: BOOLEAN;

            undef: BOOLEAN;

            readOnly: BOOLEAN;

            label: String

        END;

        GuardProc = PROCEDURE (VAR par: Par);

        NotifierProc = PROCEDURE (op, from, to: INTEGER);

        Language = ARRAY 3 OF CHAR;

        LangNotifier = POINTER TO ABSTRACT RECORD

            (n: LangNotifier) Notify-, NEW, ABSTRACT

        END;

    VAR

        metricSystem: BOOLEAN;

        showsStatus: BOOLEAN;

        version: INTEGER;

        platform: INTEGER;

        appName: ARRAY 32 OF CHAR;

        language-: Language;

        user: ARRAY 32 OF CHAR;

        thickCaret: BOOLEAN;

        caretPeriod: INTEGER;

        commandLinePars: String;

    PROCEDURE Update (IN x: ANYREC);

    PROCEDURE UpdateBool (VAR x: BOOLEAN);

    PROCEDURE UpdateByte (VAR x: BYTE);

    PROCEDURE UpdateChar (VAR x: CHAR);

    PROCEDURE UpdateInt (VAR x: INTEGER);

    PROCEDURE UpdateLInt (VAR x: LONGINT);

    PROCEDURE UpdateList (IN x: ANYREC);

    PROCEDURE UpdateReal (VAR x: REAL);

    PROCEDURE UpdateSChar (VAR x: SHORTCHAR);

    PROCEDURE UpdateSInt (VAR x: SHORTINT);

    PROCEDURE UpdateSReal (VAR x: SHORTREAL);

    PROCEDURE UpdateSString (IN x: ARRAY OF SHORTCHAR);

    PROCEDURE UpdateSet (VAR x: SET);

    PROCEDURE UpdateString (IN x: ARRAY OF CHAR);

    PROCEDURE MapParamString (in, p0, p1, p2: ARRAY OF CHAR; OUT out: ARRAY OF CHAR);

    PROCEDURE MapString (in: ARRAY OF CHAR; OUT out: ARRAY OF CHAR);

    PROCEDURE RegisterLangNotifier (notifier: LangNotifier);

    PROCEDURE RemoveLangNotifier (notifier: LangNotifier);

    PROCEDURE SetLanguage (lang: Language; persistent: BOOLEAN);

    PROCEDURE ResetLanguage;

    PROCEDURE ShowParamMsg (IN str, p0, p1, p2: ARRAY OF CHAR);

    PROCEDURE ShowMsg (IN str: ARRAY OF CHAR);

    PROCEDURE ShowParamStatus (IN str, p0, p1, p2: ARRAY OF CHAR);

    PROCEDURE ShowStatus (IN str: ARRAY OF CHAR);

    PROCEDURE FlushMappings;

    PROCEDURE GetOK (IN str, p0, p1, p2: ARRAY OF CHAR; form: SET; OUT res: INTEGER);

    PROCEDURE GetIntSpec (defType: Files.Type; VAR loc: Files.Locator; OUT name: Files.Name);

    PROCEDURE GetExtSpec (defName: Files.Name; defType: Files.Type; VAR loc: Files.Locator;

                                                OUT name: Files.Name);

    PROCEDURE GetColor (in: INTEGER; OUT out: INTEGER; OUT set: BOOLEAN);

    PROCEDURE Call (IN cmd, errorMsg: ARRAY OF CHAR; OUT res: INTEGER);

    PROCEDURE Beep;

    PROCEDURE Notify (id0, id1: INTEGER; opts: SET);

END Dialog.

Module *Dialog* provides a variety of auxiliary services to simplify user interaction of a program. In particular, the output of messages, e.g. error messages, is supported. Furthermore, various base types are provided: *List, Selection, Combo*, *Currency, Tree*, etc. These types are known to the framework (more exactly: they are known to module *Controls*) and can be displayed by suitable controls, i.e. views which display not a normal model, but instead a variable of one of the mentioned types.

CONST **pressed**

This value may be passed to the *op* field of a notifier procedure. It notifies about a mouse-down event, i.e. the primary mouse key has just been pressed.

CONST **released**

This value may be passed to the *op* field of a notifier procedure. It notifies about a mouse-up event, i.e. the primary mouse key has just been released.

CONST **changed**

This value may be passed to the *op* field of a notifier procedure. It notifies about some change of an interactor field's value. For a *Selection*, the more specific constants *included*, *excluded*, or *set* are used.

CONST **included**

This value may be passed to the *op* field of a notifier procedure. It notifies about an inclusion of the range *[from..to]* in a *Selection*. Before the operation, this range was not included in the set.

CONST **excluded**

This value may be passed to the *op* field of a notifier procedure. It notifies about an exclusion of the range *[from..to]* in a *Selection*. Before the operation, this range was included in the set.

CONST **set**

This value may be passed to the *op* field of a notifier procedure. It notifies about a change in a *Selection* or *SET*, resulting in a set {from..to}. Any previous selection was cleared.

CONST **ok**

This value may be used as an element of the *form* set parameter of procedure *GetOK*. It indicates that the user has pressed the *OK* button.

CONST **yes**

This value may be used as an element of the *form* set parameter of procedure *GetOK*. It indicates that the user has pressed the *Yes* button.

CONST **no**

This value may be used as an element of the *form* set parameter of procedure *GetOK*. It indicates that the user has pressed the *No* button.

CONST **cancel**

This value may be used as an element of the *form* set parameter of procedure *GetOK*. It indicates that the user has pressed the *Cancel* button.

CONST **windows32s**

This is a possible value of variable *platform*. It indicates that BlackBox is running on Windows 3.1 (Win32s). This platform is not supported anymore.

CONST **windows95**

This is a possible value of variable *platform*. It indicates that BlackBox is running on Windows 95, Windows 98, Windows 98SE, or Windows 98 ME.

CONST **windowsNT3**

This is a possible value of variable *platform*. It indicates that BlackBox is running on Windows NT 3.x.

CONST **windowsNT4**

This is a possible value of variable *platform*. It indicates that BlackBox is running on Windows NT 4.x.

CONST **windows2000**

This is a possible value of variable *platform*. It indicates that BlackBox is running on Windows 2000 (formerly called Windows NT 5.0).

CONST **windows98**

This is a possible value of variable *platform*. It indicates that BlackBox is running on one of the Windows 98 flavors (original Windows 98, Windows 98 SE, or Windows 98 ME).

CONST **windowsXP**

This is a possible value of variable *platform*. It indicates that BlackBox is running on Windows XP.

CONST **windowsVista**

This is a possible value of variable *platform*. It indicates that BlackBox is running on Windows Vista.

CONST **macOS**

This is a possible value of variable *platform*. It indicates that BlackBox is running on Mac OS 7.x, 8.x, or 9.x. This platform is not supported anymore.

CONST **macOSX**

This is a possible value of variable *platform*. It indicates that BlackBox is running on Mac OS X. This platform is currently not supported.

CONST **linux**

This is a possible value of variable *platform*. It indicates that BlackBox is running on Linux. This platform is currently not supported.

CONST **tru64**

This is a possible value of variable *platform*. It indicates that BlackBox is running on Compaq Tru64 Unix.  This platform is currently not supported.

CONST **firstPos**

This value may be used in calls to *Tree* variables. It indicates that the first child of a node is requested.

CONST **lastPos**

This value may be used in calls to *Tree* variables. It indicates that the last child of a node is requested.

CONST **persistent**

This value may be used in calls to *SetLanguage* variables. It indicates that the setting is to be stored in a persistent registry and used again when BlackBox is started the next time.

CONST **nonPersistent**

This value may be used in calls to *SetLanguage* variables. It indicates that the setting is not to be stored in a persistent registry and thus will not affect BlackBox the next time it is started.

TYPE **String**

String type for various names to be displayed for the user, or to be entered by the user.

TYPE **List**

A list type defines a sub range of the *INTEGER* type, and an item name (a string) for each element of this range. All valid names can be enumerated by indexing from *0* upwards until *len - 1*.

**index**: INTEGER    index >= -1  &  index < len

Currently selected item of the list. If *index = -1* then no element of the list is selected, which may happen e.g. if *len = 0*.

**len**-: INTEGER    len >= 0

Number of elements in the list.

PROCEDURE (VAR l: List) **SetLen** (len: INTEGER)

NEW

Makes sure that the list has at least *len* elements available. If *len > l.len*, then the size of the existing list is increased as much as necessary. Note that *SetItem* also increases the list size if necessary, so *SetLen* is strictly necessary only to shorten the list. *SetLen* should be called when the size of the list to be constructed is known in advance, to avoid unnecessary allocations and copying in *SetItem*.

If *len > l.len*, the existing *l.len* elements are not affected by *SetLen*.

Pre

len >= 0    20

Post

l.len = len

PROCEDURE (VAR l: List) **SetItem** (index: INTEGER; IN item: String)

NEW

Given an index *index*, the corresponding item name is set, or overwritten if it had been set earlier. If *index >= l.len* then the length is increased as much as necessary

Pre

index >= 0    20

item # ""    21

Post

index <l.len'

    l.len = l.len'

index >= l.len'

    l.len = index + 1

PROCEDURE (VAR l: List) **GetItem** (index: INTEGER; OUT item: String)

NEW

Given an index *index*, the corresponding item name is returned. If *index* is outside of the valid index range, the empty string "" is returned.

Post

name # "" iff index is in 0 .. l.len - 1

PROCEDURE (VAR l: List) **SetResources** (IN key: ARRAY OF CHAR)

NEW

Set up the item list according to entries in a string resource file. For example, *key = "#System:colors"* would build up an item list (red, green, blue), assuming that resource file *System/Rsrc/Strings* contains the entries

    key[0]    red

    key[1]    green

    key[2]    blue

Pre

key # ""    20

TYPE **Selection**

A selection is similar to a *List*, except that not only one value can be represented, but between 0 and an arbitrary number of values instead, i.e., a selection is a potentially large set of integers. In this context, the term "list" denotes all selectable elements, not only the selected ones.

**len**-: INTEGER    len >= 0

Number of elements in the list.

PROCEDURE (VAR s: Selection) **SetLen** (len: INTEGER)

NEW

Makes sure that the list has at least *len* elements available. If *len > l.len*, then the size of the existing list is increased as much as necessary. Note that *SetItem* also increases the list size if necessary, so *SetLen* is strictly necessary only to shorten the list. *SetLen* should be called when the size of the list to be constructed is known in advance, to avoid unnecessary allocations and copying in *SetItem*.

If *len > l.len*, the existing *l.len* elements are not affected by *SetLen*.

Pre

len >= 0    20

Post

s.len = len

PROCEDURE (VAR s: Selection) **SetItem** (index: INTEGER; IN item: String)

NEW

Given an index *index*, the corresponding item name is set, or overwritten if it had been set earlier. If *index >= s.len* then the length is increased as much as necessary

Pre

index >= 0    20

item # ""    21

Post

index <s.len'

    s.len = s.len'

index >= s.len'

    s.len = index + 1

PROCEDURE (VAR s: Selection) **GetItem** (index: INTEGER; OUT item: String)

NEW

Given an index *index*, the corresponding item name is returned. If *index* is outside of the valid index range, the empty string "" is returned.

Post

name # "" iff index is in 0 .. s.len - 1

PROCEDURE (VAR s: Selection) **SetResources** (IN key: ARRAY OF CHAR)

NEW

Set up the item list according to entries in a string resource file. For example, *key = "#System:colors"* would build up an item list (red, green, blue), assuming that resource file *System/Rsrc/Strings* contains the entries

    key[0]    red

    key[1]    green

    key[2]    blue

Pre

key # ""    20

PROCEDURE (VAR s: Selection) **Incl** (from, to: INTEGER)

NEW

Include the range *[from..to]* intersected with *[0..s.len - 1]* into the selection. If *from > to*, this is regarded as an empty range.

PROCEDURE (VAR s: Selection) **Excl** (from, to: INTEGER)

NEW

Exclude the range *[from..to]* intersected with *[0..s.len - 1]* into the selection. If *from > to*, this is regarded as an empty range.

PROCEDURE (VAR s: Selection) **In** (index: INTEGER): BOOLEAN

NEW

Determine whether element *index* is in the selection. If *index* is outside of the range *[0..s.len-1]*, then *FALSE* is returned.

TYPE **Combo**

A combo is similar to a *List*, except that it also accepts other values than the predefined ones of a list. Typically, a combo is represented on the screen as a combo box control. Such a control is a mixture of a list box or popup box (where one of the listed values can be chosen) and a text field (in which non-standard values can be typed in).

**item**: String

Current value of the combo.

**len**-: INTEGER    len >= 0

Number of elements in the list.

PROCEDURE (VAR c: Combo) **SetLen** (len: INTEGER)

NEW

Makes sure that the list has at least *len* elements available. If *len > l.len*, then the size of the existing list is increased as much as necessary. Note that *SetItem* also increases the list size if necessary, so *SetLen* is strictly necessary only to shorten the list. *SetLen* should be called when the size of the list to be constructed is known in advance, to avoid unnecessary allocations and copying in *SetItem*.

If *len > l.len*, the existing *l.len* elements are not affected by *SetLen*.

Pre

len >= 0    20

Post

c.len = len

PROCEDURE (VAR c: Combo) **SetItem** (index: INTEGER; IN item: String)

NEW

Given an index *index*, the corresponding item name is set, or overwritten if it had been set earlier. If *index >= c.len* then the length is increased as much as necessary

Pre

index >= 0    20

item # ""    21

Post

index <c.len'

    c.len = c.len'

index >= c.len'

    c.len = index + 1

PROCEDURE (VAR c: Combo) **GetItem** (index: INTEGER; OUT item: String)

NEW

Given an index *index*, the corresponding item name is returned. If *index* is outside of the valid index range, the empty string "" is returned.

Post

name # "" iff index is in 0 .. c.len - 1

PROCEDURE (VAR c: Combo) **SetResources** (IN key: ARRAY OF CHAR)

NEW

Set up the item list according to entries in a string resource file. For example, *key = "#System:colors"* would build up an item list (red, green, blue), assuming that resource file *System/Rsrc/Strings* contains the entries

    key[0]    red

    key[1]    green

    key[2]    blue

Pre

key # ""    20

TYPE **TreeNode**

Holds information about a node in a *Tree*. A *TreeNode* is part of one and only one *Tree*.

PROCEDURE (tn: TreeNode) **SetName** (name: String)

NEW

Sets the name of *tn*. This is the text that is displayed when a *Tree* is bound to a tree control.

PROCEDURE (tn: TreeNode) **GetName** (OUT name: String)

NEW

Retrieves the name of *tn*.

PROCEDURE (tn: TreeNode) **SetData** (data: ANYPTR), NEW;

Associates some data with node *tn*. This can be used to associate some application defined data with each node in a *Tree*.

PROCEDURE (tn: TreeNode) **Data** (): ANYPTR

NEW

Returns the data associated with the *tn* by an earlier call to *SetData*. Returns *NIL* if no call to *SetData* has been made.

PROCEDURE (tn: TreeNode) **NofChildren** (): INTEGER

NEW

Returns the number of immediate children to *tn*, i.e. all nodes, *n*, in tree, *t*, such that *t.Parent(n) = tn*.

PROCEDURE (tn: TreeNode) **SetExpansion** (expanded: BOOLEAN)

NEW

Marks *tn* as expanded or collapsed. When the tree is displayed in a Tree Control the node corresponding to tn will be expanded or collapsed according to the value of *expanded*.

PROCEDURE (tn: TreeNode) **IsExpanded** (): BOOLEAN

NEW

Returns *TRUE* if *tn* has been expanded by a Tree Control or by an explicit call to *SetExpansion*. Otherwise *FALSE* is returned.

PROCEDURE (tn: TreeNode) **ViewAsFolder** (isFolder: BOOLEAN)

NEW

When a Tree Control has the option "Folder Icons" set, it automatically displays nodes that have children as folders. If node *tn* should be viewed as a folder even if it has no children, *tn.ViewAsFolder(TRUE)* should be called. If node *tn* has children it will be viewed as a folder even if *tn.ViewAsFolder(FALSE)* is called. *ViewAsFolder* only provides a way to make leafs look like folders, not the other way around.

PROCEDURE (tn: TreeNode) **IsFolder** (): BOOLEAN

NEW

Returns *TRUE* if *tn* has children or if *tn.ViewAsFolder(TRUE)* has been called, otherwise it returns *FALSE*.



TYPE **Tree**

Defines a tree structure for storing *TreeNodes*. Normally a *Tree* is bound to a Tree Control in the user interface. Each tree can have several roots. It is possible to navigate up and down in the tree as well as between siblings. All operations on a tree *t* that require a TreeNode *tn* have the precondition that *tn* was created using *t.NewChild* and that *tn* is still part of *t,* i.e., *tn* is a node in *t* and not in any other tree and *tn* has not been deleted from *t*.

*Note*:Tree controls look different under Windows NT and other Windows versions. The background of a tree control is not set to gray when the control is disabled or read only under Windows NT.

[See <u>Platform Specific Issues</u>](../../Dev/Docu/P-S-I.odc.md) for more information.

PROCEDURE (VAR t: Tree) **NofNodes** (): INTEGER

NEW

Returns the total number of nodes in the tree.

Post

Returned value is greater than or equal to 0.

PROCEDURE (VAR t: Tree) **NofRoots** (): INTEGER

NEW

The total number of roots in the tree. A node, *tn*, is a root if *tn.Parent() = NIL*.

Post

Returned value is greater than or equal to 0.

PROCEDURE (VAR t: Tree) **NewChild** (parent: TreeNode; pos: INTEGER; name: String): TreeNode

NEW

Creates a new node in a tree. The new node is inserted at positions *pos* among the children of *parent*. If parent has no children or *pos = firstPos* then the new node is inserted as the first child of *parent*. If *parent* has fewer children than the value of *pos *or *pos = lastPos*, then the new node is inserted as the last child of *parent*. If *parent* is NIL then the new node is added as a new root in the tree at position *pos*.

Pre

(pos >= 0) OR (pos = firstPos) OR (pos = lastPos)

Post

t.NofNodes() = t.NofNodes()' + 1

PROCEDURE (VAR t: Tree) **Delete** (node: TreeNode): INTEGER

NEW

Removes *node* and all its children from the tree.

Pre

node # NIL

Post

t.NofNodes < t.NofNodes()'

PROCEDURE (VAR t: Tree) **DeleteAll**

NEW

Removes all nodes from the tree.

Post

t.NofNodes() = 0

t.NofRoots() = 0

PROCEDURE (VAR t: Tree) **Move** (node, parent: TreeNode; pos: INTEGER)

NEW

Moves a node in a tree from its current place to the place specified by *parent *and *pos*. The interpretation of *parent* and *pos* is the same as in *NewChild*.

Pre

node # NIL

(pos >= 0) OR (pos = firstPos) OR (pos = lastPos)

Post

t.NofNodes() = t.NofNodes()'

PROCEDURE (VAR t: Tree) **Parent** (node: TreeNode): TreeNode

NEW

Returns the parent node of *node*. If *node* is a root then *NIL* is returned.

Pre

node # NIL



PROCEDURE (VAR t: Tree) **Child** (node: TreeNode; pos: INTEGER): TreeNode

NEW

Returns the child at position *pos* of *node*. If *node* is *NIL* the root at position *pos* is returned. The constants *firstPos* and *lastPos* can be used to retrieve the first and last child of a node. If *node* has no children or if it has fewer children than the value of *pos*, then *NIL* is returned.

Pre

(pos >= 0) OR (pos = firstPos) OR (pos = lastPos)

PROCEDURE (VAR t: Tree) **Next** (node: TreeNode): TreeNode

NEW

Returns the next node at the same level and with the same parent as *node* i.e. If *node* is at position *pos* then the returned node is at position *pos + 1*. If *node* is the last child of its parent then *NIL* is returned.

Pre

node # NIL

PROCEDURE (VAR t: Tree) **Prev** (node: TreeNode): TreeNode

NEW

Returns the previous node at the same level and with the same parent as *node*. If *node* is at position *pos* then the returned node is at position *pos - 1*. If *node* is the first child of its parent then *NIL* is returned.

Pre

node # NIL

PROCEDURE (VAR t: Tree) **Select** (node: TreeNode)

NEW

Makes *node* become the selected node in the tree. If *node* is NIL then there is no selection in the tree.

PROCEDURE (VAR t: Tree) **Selected** (): TreeNode

NEW

Returns the selected node in a tree. If no node is currently selected then *NIL* is returned.

TYPE **Color**

Type for colors.

**val**: INTEGER

Current color value (in the same format as *Ports.Color*).

TYPE **Currency**

Type for money values.

**val: **LONGINT

The fixed-point value of the currency. The true value is *val / 10^scale*.

**scale: **INTEGER    scale > 0

Scale factor for *val*. For example, *val = 12475* and *scale = 2* is the representation for *124.75*. If the currency denotes US dollars, then *scale = 2* means that values can be displayed and entered with cent precision. A value of *3* would increase precision to 1/10 of a cent.

TYPE **Par**

Values of this parameter type are used to set up the names of menu items, and to disable or check menu items. A procedure of type *GuardProc* has a variable parameter of type *Par*.

**disabled**: BOOLEAN

Initially set to *FALSE*, this field can be set to *TRUE* by guard commands, to disable a menu item or a control.

**checked**: BOOLEAN

Initially set to *FALSE*, this field can be set to *TRUE* to show a check mark for a menu item.

**undef**: BOOLEAN

Initially set to *FALSE*, this field can be set to *TRUE* to set the *undef* state of a control.

**readOnly**: BOOLEAN

Initially set to *FALSE*, this field can be set to *TRUE* to set the *readOnly* state of a control.

**label**: String

For menu items or controls which show different labels depending on the current context, the current string can be deposited here.

TYPE **GuardProc** = PROCEDURE (VAR par: Par)

Menu guard or control guard commands must have this signature (or the extended version described below). They can set the fields of the *par* parameter to suitable values. Guard commands are called to determine the current state (in particular to find out whether the item is currently enabled) of a menu item or a control.

For menu items, the guard commands are specified in the respective subsystem's */Rsrc/Menus* text, or in *System/Rsrc/Menus*. Menu guard commands are called after the user clicks in the menu bar, and before the menu appears.

For controls, the guard commands are specified in the inspector dialog which allows to set the various control properties. Control guard commands are called after the user interactively changed the state of a control, or after a program calls the procedure *Update* or *UpdateList *(or one of the other update procedures).

Note that when the user clicks in a menu bar, possibly all menu guard commands may be executed. After the contents of an interactor has been changed and *Update* or *UpdateList *(or one of the other update procedures) has been called, all control guards are executed. This means two things. First, a guard command must be efficient. And second, the module which contains the guard is loaded as soon as the guard is evaluated for the first time. In this respect, menu commands are a certain pitfall during development: when a module has been unloaded, it is reloaded as soon as the user tries to execute a menu command.

Guard commands may only modify fields of their *par* parameters, they must not modify any other state of the system, e.g., global variables. This means that the evaluation of a guard is similar to a function call without side-effects. Avoiding side-effects is particularly important since guards may be called by the framework at relatively unpredictable times.

An extended version of *GuardProc* can be used as an alternative, with the following signature:

    PROCEDURE (n: INTEGER; VAR par: Par)

An actual parameter for *n* must be a constant.

TYPE **NotifierProc** = PROCEDURE (op, from, to: INTEGER)

A notifier procedure must have one of the following signatures:

    PROCEDURE (op, from, to: INTEGER)

    PROCEDURE (n, op, from, to: INTEGER)

Through calls of notification procedures, an application can be notified of manipulations of a control. *op* determines the kind of manipulation:

*op = pressed*: A mouse-down event has occurred.

*op = released*: A mouse-up event has occurred.

*op = changed*: The value of a control (not bound to a *SET* or a *Selection*) has been changed.

*op = included*: Range *[from..to]* has been included in a *SET* or a *Selection*. It wasn't included before.

*op = excluded*: Range *[from..to]* has been excluded from a *SET* or a *Selection*. It was included before.

*op = set*: Range *[from..to]* has been set in a *SET* or a *Selection* after clearing the previous selection.

An actual parameter for *n* must be a constant.

TYPE **LangNotifier** = POINTER TO ABSTRACT RECORD  END;

Objects of this type can be registered and unregistered using *RegisterLangNotifier* and *RemoveLangNotifier*.

PROCEDURE (n: LangNotifier) **Notify**-

NEW, ABSTRACT

This method is called for all registered *LangNotifiers* whenever the language is changed. The order in which the language notifiers are called is undefined.

VAR **metricSystem**: BOOLEAN

This variable indicates whether sizes should be measured in metric units or in inches.

VAR **showsStatus**: BOOLEAN

Indicates whether status messages are currently displayed. If *showsStatus = FALSE*, the procedures *ShowParamStatus* and *ShowStatus* will have no visible effect.

VAR **version**: INTEGER

Indicates the current major version of BlackBox.

    10 = version 1.0

    11 = version 1.1

    12 = version 1.2

    13 = version 1.3

    14 = version 1.4

    15 = version 1.5

VAR **platform**: INTEGER

Indicates on which host operating system the application is running. The currently supported platforms are: *windows95*, *windowsNT3*, *windowsNT4*, *windows2000*.

VAR **appName**: ARRAY 32 OF CHAR

Gives the name of the application program which is currently running; the default is "BlackBox".

VAR **language-**: Language

Current language in ISO 639 codes. See *SetLanguage* for more information about language support.

VAR **user**: ARRAY 32 OF CHAR

Login name of current user. Currently not used.

VAR **thickCaret**: BOOLEAN

Determines whether the text subsystem uses a Word-like thick caret or a normal thin caret.

VAR **caretPeriod**: INTEGER

Determines the blinking period that the text subsystem uses for caret blinking. The period is given in ticks (1/1000 second). The default is 500, i.e., half a second.

VAR **commandLinePars**: String

Command line parameters that have been passed when starting BlackBox. Variable *commandLinePars* contains the string entered on the command line following the /PAR option. The string can be specified enclosed in either single or double quotes. Quotes may be omitted if no white space is contained in the string. If no /PAR option is present on the command line, *commandLinePars* contains the empty string.

Examples:

/PAR test

/PAR "parameter string"

/PAR 'A string containing a " can be entered like this'

PROCEDURE **Update** (IN x: ANYREC)

This procedure should be called after one or several fields of the interactor *x* have been modified by a program (it is called automatically when a field has been modified interactively via a control). It causes all controls which are bound to fields of this interactor to be updated, and then guards are evaluated.

PROCEDURE **UpdateList** (IN x: ANYREC)

For list-structured controls (list boxes, selection boxes, combo boxes, tree controls), the lists are re-created. For efficiency reasons, this is not done after a call to *Update*.

Note that *UpdateList* also includes the functionality of *Update*, thus for efficiency reasons you shouldn't call *UpdateList(rec); Update(rec)*.

PROCEDURE **UpdateBool** (VAR x: BOOLEAN)

Similar to *Update*, except that it accepts a BOOLEAN parameter.

PROCEDURE **UpdateByte** (VAR x: BYTE)

Similar to *Update*, except that it accepts a BYTE parameter.

PROCEDURE **UpdateChar** (VAR x: CHAR)

Similar to *Update*, except that it accepts a CHAR parameter.

PROCEDURE **UpdateInt** (VAR x: INTEGER)

Similar to *Update*, except that it accepts an INTEGER parameter.

PROCEDURE **UpdateLInt** (VAR x: LONGINT)

Similar to *Update*, except that it accepts a LONGINT parameter.

PROCEDURE **UpdateReal** (VAR x: REAL)

Similar to *Update*, except that it accepts a REAL parameter.

PROCEDURE **UpdateSChar** (VAR x: SHORTCHAR)

Similar to *Update*, except that it accepts a SHORTCHAR parameter.

PROCEDURE **UpdateSInt** (VAR x: SHORTINT)

Similar to *Update*, except that it accepts a SHORTINT parameter.

PROCEDURE **UpdateSReal** (VAR x: SHORTREAL)

Similar to *Update*, except that it accepts a SHORTREAL parameter.

PROCEDURE **UpdateSString** (IN x: ARRAY OF SHORTCHAR)

Similar to *Update*, except that it accepts an ARRAY OF SHORTCHAR parameter.

PROCEDURE **UpdateSet** (VAR x: SET)

Similar to *Update*, except that it accepts a SET parameter.

PROCEDURE **UpdateString** (IN x: ARRAY OF CHAR)

Similar to *Update*, except that it accepts an ARRAY OF CHAR parameter.

PROCEDURE **MapParamString **(in, p0, p1, p2: ARRAY OF CHAR; OUT out: ARRAY OF CHAR)

Translates string *in* into string *out*. Strings of the form "#Subsystem:message" are translated if there is a corresponding "Strings" resource file for this subsystem (in the subsystem's "Rsrc" directory). Otherwise, the "#Subsystem:" prefix is stripped away, if there is no resource file.

As an example, "#System:Cancel" may be translated to "Cancel" in the USA, and to "Abbrechen" in Germany; or to "Cancel" if the resource file or the appropriate entry is missing.

Three additional input parameters can be spliced into the *in* parameter. These parameters are inserted where "^0", "^1", or "^2" occur in *in*. The parameters are not mapped, but merely substituted.

*MapParamString* allows to remove country- and language-specific strings from a program source text, while at the same time providing a default string in the program source text such that the program always works, even if string resources are missing.

PROCEDURE **MapString** (in: ARRAY OF CHAR; OUT out: ARRAY OF CHAR)

This is a simplified version of *MapParamString* which has no additional input parameters.

Except for performance, equivalent to:

    MapParamString(in, "", "", "", out)

PROCEDURE **RegisterLangNotifier** (notifier: LangNotifier)

Adds *notifier* to the list of *LangNotifiers* to be called every time the language is changed.

Pre

notifier # NIL    20

PROCEDURE **RemoveLangNotifier** (notifier: LangNotifier)

Removes *notifier* from the list of *LangNotifiers* called every time the language is changed.

Pre

notifier # NIL, 20

PROCEDURE **SetLanguage** (lang: ARRAY OF CHAR; persistent: BOOLEAN)

Sets the current language to *lang*, specified in ISO 639 code. This indicates that String resources are not read from the Rsrc-directory directly but from a subdirectory within the Rsrc-directory with the same name as the language code. It also sets the value of the global variable *language*.

For example, if *lang* = "de" the String resources are read from the directory "Rsrc/de". If no such directory exists or the requested resource does not exist then the resources are read from the normal Rsrc-directory. An empty string implies that no particular language has been selected and resources are read from the Rsrc-directory.

The *persistent* parameter indicates whether the setting is being used again when BlackBox is started the next time. *persistent* = *nonPersistent* causes a non-permanent change, i.e. the change is effective for this particular instance of BlackBox only. With *persistent* = *persistent* the change is registered in a registry and the language will be set immediately when BlackBox starts up next time.

For information on how to do software that needs to be notified whenever the value of the global variable *language* is changing, see *RegisterLangNotifier*.

Pre

(lang = "") OR (LEN(lang$) = 2)    20

PROCEDURE **ResetLanguage**

Resets the current language to the value of the last persistent setting. See also *SetLanguage*.

PROCEDURE **ShowParamMsg** (IN str, p0, p1, p2: ARRAY OF CHAR)

Presents *str* as a message to the user. The string *str* is mapped. The additional input parameters *p0*, *p1*, and *p2* are not mapped. This procedure is used to present urgent messages to the user, typically alerting the user that some action has failed. It shouldn't be used for casual success messages. If a log window is present it is assumed that the user prefers these kind of messages in the log. Therefore the message is printed in the log if one exists, otherwise the message is displayed in a separately opened dialog box.

Pre

str # NIL    20

PROCEDURE **ShowMsg** PROCEDURE (IN str: ARRAY OF CHAR)

This is a simplified version of *ShowParamMsg* which has no additional input parameters.

Except for performance, equivalent to:

    ShowParamMsg(str, "", "", "")

Pre

str # NIL    20

PROCEDURE **ShowParamStatus** (IN str, p0, p1, p2: ARRAY OF CHAR)

Presents *str* as a message to the user. The string *str* is mapped. The additional input parameters *p0*, *p1*, and *p2* are not mapped. In contrast to *ShowParamMsg*, *ShowParamStatus* is used for shorter-lived and less urgent messages; e.g., messages produced and updated during a lengthy process. This procedure should not be used for vital messages, because on some platforms there may be no status area to display status messages, or the message mechanism may be switched off. These conditions are indicated by the global variable *showsStatus*.

PROCEDURE **ShowStatus** PROCEDURE (IN str: ARRAY OF CHAR)

This is a simplified version of *ShowParamStatus* which has no additional input parameters.

Except for performance, equivalent to:

    ShowParamStatus(str, "", "", "")

PROCEDURE **FlushMappings**

String mappings are cached in internal tables for efficiency reasons. This procedure flushes all string mapping tables. This forces a reload of these tables from the string resource files as soon as the mappings are performed again.

PROCEDURE **GetOK** (IN str, p0, p1, p2: ARRAY OF CHAR; form: SET; OUT res: INTEGER)

Modal dialog

Presents a mapped string, with the optional parameters *p0* to *p2*, in a modal dialog box. *form* indicates the set of buttons of the dialog box. Only meaningful combinations are allowed:

    {ok}

    {ok, cancel}

    {yes, no}

    {yes, no, cancel}

*res* indicates which button has been pressed by the user.

Pre

((yes IN form) = (no IN form))  &  ((yes IN form) # (ok IN form))    20

Post

res IN form

PROCEDURE **GetIntSpec** (defType: Files.Type; VAR loc: Files.Locator; OUT name: Files.Name)

Modal dialog

Ask the user for a file specification *(loc, name)*. *defType* indicates which file type is desired ("" stands for any file type; other types are platform-specific, e.g., "txt" for Windows Ascii files or "TEXT" for Mac OS Ascii files). *loc # NIL* indicates a valid file specification.

Mac OS: *loc* is ignored.

Pre

defType = ""  OR  defType is legal type name on this platform    20

PROCEDURE **GetExtSpec** (defName: Files.Name; defType: Files.Type; VAR loc: Files.Locator;

                                                OUT name: Files.Name)

Modal dialog

Ask the user for a file specification for externalizing a file. *defName* is the default name presented to the user. *defType* is the file type which should be used as default type. *loc # NIL* indicates a valid file specification.

Mac OS: *loc* is ignored.

Pre

defType = ""  OR  defType is legal type name on this platform    20

PROCEDURE **GetColor** (in: INTEGER; OUT out: LONGINT; OUT set: BOOLEAN)

Modal dialog

Ask the user for a color. *in* is the default color presented to the user.

PROCEDURE **Call** (IN cmd, errorMsg: ARRAY OF CHAR; OUT res: INTEGER)

*Call* executes a sequence of BlackBox commands denoted by *cmd*. If the corresponding modules are not yet loaded, *Call* tries to load them. If some error occurs, command execution terminates and *res* is returned with a value # 0. If *errorMsg = ""*, *Call* does not display error messages. If *errorMsg # ""*, *Call* displays *errorMsg* in case of an error, appended with a short description of the particular error having occurred.

The syntax for commands with parameters is explained in the documentation of module *StdInterpreter*.

PROCEDURE **Beep**

Emit a short beep sound.

PROCEDURE **Notify**

Used internally.

