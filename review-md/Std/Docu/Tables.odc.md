**StdTables**

DEFINITION StdTables;

    IMPORT

        Ports, Dialog, Views, Properties, Controls;

    CONST

        line = 0DX;

        deselect = -1; select = -2; changed = -3;

        layoutEditable = 0; dataEditable = 1; selectionStyle = 2;

        noSelect = 0; cellSelect = 1; rowSelect = 2; colSelect = 3; crossSelect = 4;

    TYPE

        Table = RECORD

            rows-, cols-: INTEGER;

            (VAR tab: Table) SetSize (rows, cols: INTEGER), NEW;

            (VAR tab: Table) SetItem (row, col: INTEGER; item: Dialog.String), NEW;

            (VAR tab: Table) GetItem (row, col: INTEGER; OUT item: Dialog.String), NEW;

            (VAR tab: Table) SetLabel (col: INTEGER; label: Dialog.String), NEW;

            (VAR tab: Table) GetLabel (col: INTEGER; OUT label: Dialog.String), NEW;

            (VAR tab: Table) HasSelection (): BOOLEAN, NEW;

            (VAR tab: Table) GetSelection (OUT row, col: INTEGER), NEW;

            (VAR tab: Table) Select (row, col: INTEGER), NEW;

            (VAR tab: Table) Deselect, NEW;

            (VAR tab: Table) SetAttr (l, t, r, b: INTEGER; style: SET; weight: INTEGER; color: Ports.Color), NEW;

            (VAR tab: Table) GetAttr (row, col: INTEGER;

                                        OUT style: SET; OUT weight: INTEGER; OUT color: Ports.Color), NEW

        END;

        Prop = POINTER TO RECORD (Properties.Property)

            layoutEditable, dataEditable: BOOLEAN;

            selectionStyle: INTEGER;

            (p: Prop) IntersectWith (q: Properties.Property; OUT equal: BOOLEAN)

        END;

        Directory = POINTER TO ABSTRACT RECORD

            (d: Directory) NewControl (p: Controls.Prop): Views.View, ABSTRACT, NEW

        END;

    VAR

        dir-, stdDir-: Directory;

        text: Dialog.String;

        dlg: RECORD

            layoutEditable, dataEditable: BOOLEAN;

            selectionStyle: Dialog.List

        END;

    PROCEDURE InitDialog;

    PROCEDURE Set;

    PROCEDURE Guard (idx: INTEGER; VAR par: Dialog.Par);

    PROCEDURE Notifier (idx, op, from, to: INTEGER);

    PROCEDURE SetDir (d: Directory);

    PROCEDURE DepositControl;

END StdTables.

Module *StdTables* implements a simple tabular control ("grid view"). To allow clients to create new table controls, a directory object is exported (*StdTables.dir*). As interactor, type *StdTables.Table* is provided.

Typical menu command:

    "Insert Table"    ""    "StdTables.DepositControl; StdCmds.PasteView"    "StdCmds.PasteViewGuard"



The property editor for the StdTables is the same as for normal controls. The fields, Link, Label, Guard and Notifier works in the same way as for other controls. In addition to the property editor, a special dialog for tables is offered, where one can choose the visual feedback of the selection in a table. Furthermore one can specify, whether the layout should be editable (column width), and whether the data in the cells should be editable.

CONST **line**

New line character for multiple line labels.

CONST **deselect**

Notifier op-code. Indicates that the cell at position row = *from* and column = *to* has been deselected.

CONST **select**

Notifier op-code. Indicates that the user has selected a cell at position row = *from* and column = *to.*

CONST **changed**

Notifier op-code. Indicates that the user has changed the contents of a cell in the table at position row = *from* and column = *to.*

CONST **layoutEditable**

Element of a control property's valid set. Determines, whether the layout editable property is valid.

CONST **dataEditable**

Element of a control property's valid set. Determines, whether the data editable property is valid.

CONST **selectionStyle**

Element of a control property's valid set. Determines, whether the selection style property is valid.

CONST **noSelect, cellSelect, rowSelect, colSelect, crossSelect**

Selection style property constants.

TYPE **Table**

Interactor for table controls.

**rows**-, **cols**-: INTEGER        (rows >= 0) & (cols >= 0)

Number of rows and columns of the table.

PROCEDURE (VAR tab: Table) **SetSize** (rows, cols: INTEGER), NEW

Set size of the table.

Pre

20    (rows >= 0) & (cols >= 0)

21    ((cols > 0) OR ((cols = 0) & (rows = 0))

PROCEDURE (VAR tab: Table) **SetItem** (row, col: INTEGER; item: Dialog.String), NEW

Set contents of cell in row *row* and column *col* to *item*.

Pre

20    SetSize must have been called before

PROCEDURE (VAR tab: Table) **GetItem** (row, col: INTEGER; OUT item: Dialog.String), NEW

Get contents of cell in row *row* and column *col*.

Pre

20    SetSize must have been called before

PROCEDURE (VAR tab: Table) **SetLabel** (col: INTEGER; label: Dialog.String), NEW

Set contents of label of column *col*. For each occurence of the character *StdTables.line* in *label* a new line is created.

Pre

20    SetSize must have been called before

PROCEDURE (VAR tab: Table) **GetLabel** (col: INTEGER; OUT label: Dialog.String), NEW

Get contents of label of column *col*.

Pre

20    SetSize must have been called before

PROCEDURE (VAR tab: Table) **HasSelection** (): BOOLEAN, NEW

Returns, whether a cell in the table is currently selected.

PROCEDURE (VAR tab: Table) **GetSelection** (OUT row, col: INTEGER), NEW

Get coordinates (row and column) of the selected table cell.

Pre

20    SetSize must have been called before

21    tab.HasSelection()

PROCEDURE (VAR tab: Table) **Select** (row, col: INTEGER), NEW

Set selection in table to row *row* and column *col*.

Pre

20    SetSize must have been called before

Post

tab.HasSelection() = TRUE

PROCEDURE (VAR tab: Table) **Deselect**, NEW

Remove current selection in table, if any.

Pre

20    SetSize must have been called before

Post

tab.HasSelection() = FALSE

PROCEDURE (VAR tab: Table) **SetAttr** (l, t, r, b: INTEGER; style: SET; weight: INTEGER; color: Ports.Color), NEW

Sets the the attributes for a range of cells. *style* and *weight* affects the font for the cell and *color* is the color of the text in the cell. (For explanations of the parameters *style* and *weight*, see [<u>Fonts.Font</u>](../../System/Docu/Fonts.odc.md).) The range is indicated by *l* and *t* being the column and row for the top left cell in the range, and *r* and *b* being the column and row of the bottom right cell in the range.

PROCEDURE (VAR tab: Table) **GetAttr** (row, col: INTEGER; OUT style: SET; OUT weight: INTEGER; OUT color: Ports.Color), NEW

Retrieves the current attribute values for the cell at position *row, col* in the table.

Pre

20    SetSize must have been called before

TYPE **Prop**

Table specific properties.

**layoutEditable**, **dataEditable**: BOOLEAN

Column width is editable by the user; data in table cells is editable by the user.

**selectionStyle**: INTEGER

Visual feedback of selections:

noSelect:     no visual feedback

cellSelect:     selected cell is high-lighted

rowSelect:     all cells in selected row are high-lighted

colSelect:     all cells in selected column are high-lighted

crossSelect:    all cells in selected row and column are high-lighted

PROCEDURE (p: Prop) **IntersectWith** (q: Properties.Property; OUT equal: BOOLEAN)

Intersect table properties *p* with another property record *q*. Iff both are equal, *equal* is set to TRUE.

TYPE **Directory**

New controls can be created using the directory object *StdTables.dir*.

PROCEDURE (d: Directory) **NewControl** (p: Controls.Prop): Views.View, ABSTRACT, NEW

Create a new table control using properties *p*.

VAR **dir**-, **stdDir**-: Directory        dir # NIL, stdDir # NIL, stable stdDir = d

Directory and standard directory objects for table controls.

VAR **text**: Dialog.String        valid only during the editing of a cell

Contents of the cell currently being edited.

VAR **dlg**: RECORD

Interactor for table property dialog.

PROCEDURE **InitDialog**

Init table property dialog.

PROCEDURE **Set**

Set properties as selected in table property dialog.

PROCEDURE **Guard** (idx: INTEGER; VAR par: Dialog.Par)

Guard for table property dialog.

CASE idx OF

0: Layout Editable checkbox

1: Data Editable checkbox

2: Selection Style listbox

END

PROCEDURE **Notifier** (idx, op, from, to: INTEGER)

Notifier for table property dialog.

CASE idx OF

0: Layout Editable checkbox

1: Data Editable checkbox

2: Selection Style listbox

END

PROCEDURE **SetDir** (d: Directory)

Set directory object.

Pre

d # NIL    20

Post

stdDir' = NIL

    stdDir = d

dir = d

PROCEDURE **DepositControl**

Deposit command for table controls.

