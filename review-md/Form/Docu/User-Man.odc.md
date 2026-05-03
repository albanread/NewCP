**Form Subsystem**

**User Manual**

**Contents**

[<u>1 Creating and Saving Forms</u>](#Form Creation)

[<u>2 Basic Editing</u>](#Basic Editing)

[<u>3 Navigation Keys</u>](#Navigation Keys)

[<u>4 Drag & Drop</u>](#Drag & Drop)

[<u>5 Drag & Pick</u>](#Drag & Pick)

[<u>6 Layout and Mask Modes</u>](#Layout and)

[<u>7 Controls and Interactors</u>](#Controls and)

[<u>8 Control Properties</u>](#Control Properties)

The form subsystem implements a simple form editor, which can be used to create dialog boxes or data entry  masks.

<a id="Form Creation"></a>**1 Creating and Saving Forms**

The command *Controls->New Form...* creates dialog boxes and other forms that match exported global variables. All such forms are *non-modal*. Almost all dialog boxes in BlackBox are non-modal, as long as the conventions for the underlying platform permit.

To get started, enter "TextCmds.find" into the *Link* field of the *Controls->New Form...* dialog box. By clicking the default button, a dialog box is automatically created which has fields *Find* and *Replace*, and buttons like *Find Next* and *Replace All*. These fields and buttons directly match the record fields of variable *find* in module *TextCmds*. To verify this, select the string "TextCmds.find", and execute command *Info->Interface*. The browser will display the definition of the record type.

The size of a form view can be adjusted by selecting the whole document (*Edit->Select Document*) and then dragging the graphical handles.

The dialog box created by *Controls->New Form...* exhibits a simple default arrangement of controls (e.g., buttons, edit fields), shown as an editable layout. The controls may be re-arranged, renamed, or otherwise modified using the menus *Layout* and *Controls*. The former menu appears automatically whenever a form becomes focused.

Instead of creating an initial layout out of an existing global variable, form creation may start with an empty form (click on the *New Form* dialog box's *Empty* button), and then successively insert new controls via the *Controls* menu. Later, such controls may be linked to suitable global variables.

An edited dialog box can be saved by saving the window of the form layout. The BlackBox Component Framework saves dialog boxes in the standard document format. By convention, BlackBox forms are saved in the appropriate *Rsrc* directory (-> Subsystems and Global Modules). For example, the standard *Text->Find & Replace* dialog box is stored in *Text/Rsrc/Find*.

<a id="Basic Editing"></a>**2 Basic Editing**

A view can be inserted into a form layout by invoking one of the *Insert Xyz* commands of menu *Controls*, e.g., the command *Controls->Insert Command Button*. Arbitrary other views could also be inserted, or copied from the clipboard or via drag & drop.

Views in a form can be selected. If one single view is selected ( a "singleton"), it shows resize handles that can be manipulated to change the view's size. If several views are selected, they show a selection mark like singletons do, but no resize handles.

A view can be selected either by clicking in it, or by dragging over an area which intersects the view(s). If the shift key is pressed during selection, the newly touched view(s) are toggled, i.e., a selected view is deselected and vice versa. If the shift key is not pressed during selection, any existing selection is removed prior to creating the new selection. Consequently, a simple click outside of any view removes the existing selection and does not create a new one. Pressing *esc* achieves the same effect.

Some attributes of selected views can be modified using the *Attributes* menu (and the *Fonts* menu under the Mac OS). In this way, the typeface, style, and size of the labels of many controls can be changed.

The command *Controls->Insert Group Box* works like the other insert commands, with one exception: if there is a selection, the group box is placed and sized such that it forms a bounding box around the selected views.

Mac OS:

Changing the size of a control's label has no effect, it always remains 12 point.

A selection can be cut, copied, and pasted using the clipboard. It can be deleted with the *backspace* or *delete* keys.

Menu *Layout* contains a variety of commands which are useful for editing a form layout. The *Align Left*, *Align Right*, *Align Top*, and *Align Bottom* commands align the left, right, top, or bottom borders of all selected views. Alignment occurs to the view which lies furthest in the alignment direction. For example, *Align Left* finds the selected view whose left border lies furthest left, and then moves all other views (without resizing them) so that their left borders are aligned to the same x-coordinate.

*Align To Row* aligns all selected views in vertical direction, so that all their vertical centers come to lie on the same y-coordinate, which is the average of the old y-coordinates. Similarly, *Align To Column* aligns all selected view in the horizontal direction, so that they form a single column.

The dialog box *Set Grid...* allows to set the grid. Most commands which move or resize a view round the view's borders to this grid. The dialog box allows you to choose between a grid based on millimeters or on 1/16 inches. The grid resolution allows you to specify how many grid values exist for these units (for one millimeter, or for one sixteenth of an inch). A higher value means preciser placement is possible. Roughly every centimeter or half an inch, dotted lines are drawn on the grid, as visual aids for editing.

*Select Off-Grid Views* selects all views of which at least one of its four borders is not aligned to the form's grid. *Force To Grid* shifts and resizes them by a minimal distance so that they are aligned again.

Conceptually the views are arranged in z-order, i.e., each view is either "above" or "below" another view. Normally, views in a form should not overlap, and thus this z-ordering has no immediate effect. But the same ordering is used for moving the focus between controls, by using the *tab* key. The focus is moved from the bottom-most towards the top-most view when tabbing through a dialog box. The commands *Set First/Back* and *Set Last/Front* allow to change the z-order of a selected singleton. *Sort Views* sorts the z-orders of all views in a form such that a view further to the left and top is reached before another view (which is further "up" in the hierarchy). This command can be applied after a layout has been edited, to update the z-order to make it intuitive again.

*Recalc Focus Size* operates on a focused form layout view. It calculates the bounding box of the views which are contained in the form, offsets this bounding box by a small margin, and then sets the form view's width and height to this size. This is more convenient than using *Edit->Select Document* and then resizing the view manually.

The commands *Set Default Button* and *Set Cancel Button* make a selected button into a default or a cancel button. A default button reacts to the input of a *return* or *enter* key as if the mouse had been pressed inside; a cancel button reacts to the input of an *esc* key as if the mouse had been pressed inside.

Like other views that have separate models, a form view can be opened in several windows simultaneously.

<a id="Navigation Keys"></a>**3 Navigation Keys**

Arrow keys can be used to move a selection to the left, right, upwards, or downwards by one point. If the *modifier* key is pressed before the arrow key, the selection moves by a larger distance.

<a id="Drag &amp; Drop"></a>**4 Drag & Drop**

When clicking into a selection, and moving the cursor around, the selected views can be moved around accordingly. Moving occurs on a grid, the minimal distance is the same as when using the *modifier* key together with an arrow key.

By holding down the *modifier* key when the mouse button is released at the end of dragging, copies of the selected views are dropped, and become selected.

Drag & Drop also works across windows, and even between different containers. If a singleton is dragged & dropped into another kind of container (e.g., a text container), then a copy of this view is dropped. If a whole selection is dropped, the selected views are wrapped into a form view and this form view is dropped to the other container.

<a id="Drag &amp; Pick"></a>**5 Drag & Pick**

Drag & Pick is also supported in forms. In forms, another view's size can be picked up. This is very convenient for layout editing. For example, select several controls which have different sizes, hold down the *alt* key (Windows) / *command* key (Mac OS), and then drag to another view. When you release the mouse over this view, all selected views will be made the same size as this one.

<a id="Layout and"></a>**6 Layout and Mask Modes**

Forms can be used in two different modes. Normally, a form is used in *layout mode*, i.e. its layout can be freely edited. The views (typically controls) embedded in a form however, *cannot* be edited directly, because they can only be selected, but not focused. For layout editing, it would be very inconvenient if a click in one of the form's controls would focus it, instead of only selecting it. Forms are saved in layout mode, and opened in document windows.

For using a form as a dialog box or data entry mask, a form can be opened in *mask mode* instead, in an auxiliary window (data entry mask) or in a tool window (dialog box for the manipulation of a document beneath the dialog box). A form in mask mode cannot be modified. However, its embedded views may be focused, e.g. a text entry field may be focused, or a button may be clicked. In contrast to layout mode, embedded views cannot be selected, resized, or moved around.

In mask mode, no focus border is shown around the currently focused view. In layout mode, a grid is shown.

When a form in layout mode is focus, another window can be opened in mask mode, by using either *Controls->Open As Tool Dialog* or *Controls->Open As Aux Dialog*. In this way, a form can be tried out while it is still being edited. Layout changes in the layout mode form are immediately reflected in the other window, and input in the other window is immediately reflected in the layout.

Forms need not be saved in mask mode. They are saved in layout mode, and thus can be opened, edited, and saved again via the normal *File* menu commands. A data entry mask is opened via the *StdCmds.OpenAuxDialog* command; and a dialog box is opened via the *StdCmds.OpenToolDialog* command. These commands open a form document into an auxiliary/tool window and force it into mask mode. For example, the following commands show the difference between the two modes by opening the same (layout mode) form in the two possible ways:

 "StdCmds.OpenAuxDialog('Form/Rsrc/Gen', 'New Form')"

 "StdCmds.OpenToolDialog('Text/Rsrc/Cmds', 'Find & Replace')"

 "StdCmds.OpenDoc('Form/Rsrc/Gen')"

 "StdCmds.OpenDoc('Text/Rsrc/Cmds')"

The latter two commands correspond to the *File->Open* command and allow editing.

The difference between auxiliary and tool dialog boxes is that auxiliary dialog boxes are self-contained, e.g. dialog boxes to set up configuration parameters or data entry masks. Tool dialog boxes on the other hand operate on windows below them, e.g. the *Find & Replace* dialog box operates on the text beneath the dialog box.

These *OpenAuxDialog/OpenToolDialog* commands accept *portable path names* as input. A portable path name is a string which denotes a file in a machine-independent way. It uses the "/" character as separator, i.e. like in Unix or the World-Wide Web.

These commands are usually applied in menu items, for example the following command sequence:

    "TextCmds.InitFindDialog; StdCmds.OpenToolDialog('Text/Rsrc/Find', 'Find & Replace')"

In this command sequence, the first command initializes the *TextCmds.find* interactor with the currently selected text stretch. The second command opens a dialog box with the *Text/Rsrc/Find* file's form, whose controls are linked to the interactor's fields upon opening of the dialog box.

See also modules [<u>FormGen</u>](Gen.odc.md), [<u>TextCmds</u>](../../Text/Docu/Cmds.odc.md), and [<u>StdCmds</u>](../../Std/Docu/Cmds.odc.md).

<a id="Controls and"></a>**7 Controls and Interactors**

Controls are specialized views. Like every view, any control can be inserted into any general container, be it a text, a spreadsheet, or whatever other container is available. However, most controls are put into forms.

Each control can be linked to a variable, more exactly to the field of a globally declared record variable, a so-called *interactor*. When the control is allocated (newly created or read from a document), BlackBox tries to link the control to its variable, using the advanced meta-programming capabilities of the BlackBox Component Framework core. In this way, the link between control and variable can be built up automatically when a dialog box is created or loaded, and correct linking (which includes correct typing) can be guaranteed even after a dialog box layout had been edited. The separation of interactor from controls makes it possible to hide many user-interface details from a program, e.g. the layout of a dialog box or other "cosmetic" properties of controls.

The BlackBox Component Framework provides *command buttons*, *check boxes*, *radio buttons*, *fields*, *captions, list boxes*, *selection boxes*, *combo boxes, date, time, color, up/down fields*, and *groups* as standard controls.

<a id="Control Properties"></a>**8 Control Properties**

Controls have various *properties*, e.g. the label displayed in a button, or the interactor field to which the control is linked.  The *inspector* is a tool used to inspect and to modify the properties of standard controls. To open the inspector dialog box on a control, first select the control, and then execute *Edit->Object Properties...* (Windows) / *Edit->Part Info* (Mac OS).

To learn more about the inspector, see the documentation for module [<u>DevInspector</u>](../../Dev/Docu/Inspector.odc.md).

For more information on the *Form* subsystem's programming interface, consult the on-line documentation of the modules *FormModels*, *FormViews*, *FormControllers*, *FormGen*, *FormCmds*, and *Controls*. Note that most of these modules are distributed in source form also, and thus serve as an example of a complex subsystem. Simpler examples are given in the *Obx* subsystem, in particular the examples *ObxAddress0*, *ObxAddress1*, *ObxAddress2*, *ObxOrders*, *ObxControls*, and *ObxDialog*. A tutorial on the form subsystem is given in [<u>Chapter 4</u>](../../Docu/Tut-4.odc.md) of the accompanying book on component software and the BlackBox Component Framework.

