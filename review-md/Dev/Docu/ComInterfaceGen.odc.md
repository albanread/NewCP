**DevComInterfaceGen**

DEFINITION DevComInterfaceGen;

    IMPORT Dialog;

    VAR

        dialog: RECORD

            library: Dialog.List;

            fileName: ARRAY 256 OF CHAR;

            modName: ARRAY 64 OF CHAR

        END;

    PROCEDURE Browse;

    PROCEDURE GenAutomationInterface;

    PROCEDURE InitDialog;

    PROCEDURE ListBoxNotifier (op, from, to: INTEGER);

    PROCEDURE TextFieldNotifier (op, from, to: INTEGER);

    ... plus some private items ...

END DevComInterfaceGen.

Module *DevComInterfaceGen* provides services to automatically generate COM interface modules. Interface modules are used by BlackBox to access COM components in several different ways.

One of them is Automation. Any application (server) supporting Automation can be controlled from within another application (controller). The interface of a server application, i.e., the objects provided by the application and the operations on these objects, are described in a COM type library. In order to write a controller in BlackBox, an interface module is used which contains a Component Pascal object for each COM object in the type library. These objects can be accessed like other Component Pascal objects in a convenient and typesafe way. They hide the details of the Automation standard. The *Ctl* subsystem includes several such Automation interface modules. (Basically for MS Office Automation servers.)

Module *DevComInterfaceGen* provides services to automatically generate Automation interfaces from the corresponding type libraries. In certain situations, however, the automatically generated interface module does not compile without errors. Here some minimal manual work is required. So far we have encountered three kinds of difficulties that may occur:

1. Formal parameter name is equal to parameter type name. Solution: Change the formal parameter name in the parameter list and procedure body.

2. Formal parameter name is equal to return type name. Solution: Change the formal parameter name in the parameter list and procedure body.

3. Constant name is equal to some type name. Solution: Change constant name.

A possible menu using commands from this module:

**MENU** "Automation"

    "Generate Automation Interface"    ""    "DevComInterfaceGen.InitDialog; StdCmds.OpenToolDialog('Dev/Rsrc/ComInterfaceGen', 'Generate Automation Interface')"    ""

**END**

VAR **dialog**: RECORD

The interactor for the *Generate Automation Interface* dialog.

**library**: Dialog.List

A list representing all type library entries that are registered in the system's registration database.

**fileName**: ARRAY 256 OF CHAR

Complete path name of the type library.

**modName**: ARRAY 64 OF CHAR

Module name of the Automation interface module to be generated from the corresponding type library.

PROCEDURE **Browse**

Open a standard open file dialog box, to let the user choose a type library or DLL file.

PROCEDURE **GenAutomationInterface**

Generate the interface module according to the parameters in *dialog*.

PROCEDURE **InitDialog**

Call this command before opening the *Generate Automation Interface* dialog. It initializes the *dialog* interactor.

PROCEDURE **ListBoxNotifier** (op, from, to: INTEGER)

PROCEDURE **TextFieldNotifier** (op, from, to: INTEGER)

Notifiers used by the *Generate Automation Interface* dialog.

