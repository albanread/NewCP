**FormGen**

DEFINITION FormGen;

    IMPORT Dialog;

    VAR

        new: RECORD

            link: Dialog.String

        END;

    PROCEDURE Create;

    PROCEDURE Empty;

    PROCEDURE CreateGuard (VAR par: Dialog.Par);

END FormGen.

*FormGen* is a generator for a form layout. It takes the name of an interactor variable (any exported record) as input and creates a default layout for the fields of the interactor. The following list describes the mapping from field types to controls.

BYTE, SHORTINT, INTEGER    Field

SHORTREAL, REAL    Field

ARRAY OF CHAR    Field

BOOLEAN    CheckBox

Dates.Date    DateField

Dates.Time    TimeField

Dialog.Color    ColorField

Dialog.Currency    Field

Dialog.List    ListBox

Dialog.Selection    SelectionBox

Dialog.Combo    ComboBox

Alternatively, a dialog for all interactors of a module can be generated, by entering the module name into the dialog box field. In this case, a group box is generated for every exported record variable, and a command button for every exported parameterless procedure.

A possible menu using the above commands:

**MENU** "Form"

    "&New Form..."    ""    "StdCmds.OpenAuxDialog('Form/Rsrc/Gen', 'New Form')"    ""

END

VAR **new**

Interactor for the form generator dialog.

**link**: Dialog.String

The name of the interactor for which a default layout should be generated. The name must be an identifier qualified with the module name, e.g., "FormGen.new", or only a module name, e.g., "FormGen".

PROCEDURE **Create**

Guard: CreateGuard

Create a default layout for the interactor specified in *link*.

PROCEDURE **Empty**

Create an empty form layout.

PROCEDURE **CreateGuard**

Guard for *Create*.

