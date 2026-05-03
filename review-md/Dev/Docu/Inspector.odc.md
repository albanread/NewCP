**DevInspector**

DEFINITION DevInspector;

    IMPORT Dialog;

    VAR

        inspect: RECORD

            control-: Dialog.String;

            label: ARRAY 40 OF CHAR;

            link, guard, notifier: Dialog.String;

            level: INTEGER;

            opt0, opt1, opt2, opt3, opt4: BOOLEAN

        END;

    PROCEDURE GetNext;

    PROCEDURE Set;

    PROCEDURE InitDialog;

    PROCEDURE ControlGuard (VAR par: Dialog.Par);

    PROCEDURE GuardGuard (VAR par: Dialog.Par);

    PROCEDURE LabelGuard (VAR par: Dialog.Par);

    PROCEDURE LevelGuard (VAR par: Dialog.Par);

    PROCEDURE LinkGuard (VAR par: Dialog.Par);

    PROCEDURE OptGuard (opt: INTEGER; VAR par: Dialog.Par);

    PROCEDURE NotifierGuard (VAR par: Dialog.Par);

    PROCEDURE Notifier (idx, op, from, to: INTEGER);

END DevInspector.

The inspector makes it possible to inspect and modify properties of a control. Currently, various types of controls are supported: command buttons, check boxes, radio buttons, edit fields, date fields, time fields, color fields, up/down-fields list boxes, selection boxes, combo boxes, and groups.

The inspector is opened with the *Edit->Object Properties...* command (Windows) / *Edit->Part Info* command (Mac OS). It takes a singleton control as input (-> Controls). Starting the inspector is done indirectly by the framework, using the *StdCmds.ShowProp* command.

VAR **inspect**: RECORD

Interactor for a control view property dialog.

**control**-: Dialog.String

The control's name. This is the (possibly mapped) name of the control type.

**label**: ARRAY 40 OF CHAR

Label string of the control.

**link**: Dialog.String

Link to the interactor field, in the form *module.variable.field*.

**guard**: Dialog.String

Name of the control's guard command.

**notifier**: Dialog.String

Name of the control's notifier command.

**level**: INTEGER

Iff the value of a radio button is equal to level, the radio button is "on".

**opt0, opt1, opt2, opt3, opt3**: BOOLEAN

Various options which depend on the currently selected control. For example, a command button uses *opt0* and *opt1* to indicate the "default" or "cancel" properties.

PROCEDURE **GetNext**

Show the next control in this container. After the last control, *GetNext* wraps around to the first control.

PROCEDURE **Set**

Set the control's properties to the currently displayed values.

PROCEDURE **InitDialog**

Sets up the interactor according to the currently selected control.

PROCEDURE **ControlGuard** (VAR par: Dialog.Par)

PROCEDURE **GuardGuard** (VAR par: Dialog.Par)

PROCEDURE **LabelGuard** (VAR par: Dialog.Par)

PROCEDURE **LevelGuard** (VAR par: Dialog.Par)

PROCEDURE **LinkGuard** (VAR par: Dialog.Par)

PROCEDURE **OptGuard** (opt: INTEGER; VAR par: Dialog.Par)

PROCEDURE **NotifierGuard** (VAR par: Dialog.Par)

PROCEDURE **Notifier** (idx, op, from, to: INTEGER)

Various guards and notifiers used in the inspector dialog.

