**Overview by Example: ObxControls**

A change of an interactor field (i.e., a field of a globally declared record variable) may affect not only controls which are linked to this field, but others as well. For example, a command button may be disabled as long as a text field is empty, and become enabled when something is typed into the text field.

**Guard commands**

Such state changes of controls, which are the results of state changes in an interactor, are handled by *guard commands*. A guard is a command which may disable a control, may denote it as undefined, or may make it read-only. It does not have other side effects; in particular, it doesn't invoke an arbitrary action or change the state of an interactor. For that purpose, notifiers are used.

**Notifier commands**

A notifier is an optional command that can be bound to a control, using the control property editor (-> DevInspector). For example, a notifier command may write something into the status bar of a window when the mouse button is pressed, and clear the status bar when the mouse button is released again.

    It is more typical, however, that a notifier changes the state of the interactor to which its control is linked; i.e., to change one or more of the interactor fields to which its control is *not* linked.

     In this way, the change of one interactor field's value may cause a change of another field's value, via the former's notifier.

**Example**

The example illustrates guards and notifiers, as well as command button, radio button, and list box controls.

    Imagine something whose size should be controlled via a dialog, with more skilled users getting more degrees of freedom. For example, this may be a dialog for a game, which allows the user to choose a skill level. A novice user only gets a default amount of money to invest in an economics simulation game, a more experienced user additionally may choose among three predefined choices, while a guru may enter any value he or she wants to use. In order to keep our example simple and to concentrate on the control behaviors, nothing as complex as a simulation game is implemented. Instead, the initial size of a square view can be controlled by the user, in a more or less flexible way that depends on the chosen skill level.

    In our example, the skill level is implement as the *class* field of the *data* interactor in module *ObxControls*. It is an integer variable, to which a text entry field may be linked, or more appropriately, a number of radio buttons. The radio buttons are labeled *beginner*, *advanced*, *expert*, and *guru*. Depending on the currently selected class, the interactor's *list* field is adapted: for a beginner, the list box should be disabled, because there is no choice (the default is taken). For an advanced player, a choice between "small", "medium", and "large" are presented in addition to the "default" size. Obviously, the list box must be enabled if such a choice exists. "expert" players get even more choices, namely "tiny" and "huge". Note that if these choices appear, the list box becomes too small to show all choices simultaneously. As a consequence, the scroll bar of the list box becomes enabled.

    "guru" players have even more freedom, they can type in the desired size numerically, in a text entry field which is read-only for all other skill levels. The command button *Cancel* closes the dialog without doing anything, the *Open* button starts the game, and the *OK* button starts the game and immediately closes the dialog.

Since we are mainly interested in how to implement the described behavior, we don't actually implement a game. Instead, the "game" merely consists in opening a new view, whose size is determined by the size chosen in the dialog (default, large, small, etc.)

With the *New Form...* menu command, a new dialog box for the *ObxControls.data* interactor can be created automatically. The layout of this dialog can be edited interactively, and the properties of the various controls can be set using the control property editor. They should be set up as in the table below:

Control    Link    Label    Guard    Notifier    Level

Radio Button    ObxControls.data.class    &Beginner        ObxControls.ClassNotify    0

Radio Button    ObxControls.data.class    &Advanced        ObxControls.ClassNotify    1

Radio Button    ObxControls.data.class    &Expert        ObxControls.ClassNotify    2

Radio Button    ObxControls.data.class    &Guru        ObxControls.ClassNotify    3

List Box    ObxControls.data.list        ObxControls.ListGuard    ObxControls.ListNotify

Text Field    ObxControls.data.width        ObxControls.WidthGuard

Command Button    StdCmds.CloseDialog;

    ObxControls.Open    OK

Command Button    ObxControls.Open    &Open

 "StdCmds.OpenAuxDialog('Obx/Rsrc/Controls', 'ObxControls Demo')"

[<u>ObxControls  sources</u>](../Mod/Controls.odc.md)

