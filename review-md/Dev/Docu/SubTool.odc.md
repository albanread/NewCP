**DevSubTool**

DEFINITION DevSubTool;

    CONST

        textCmds = 0; formCmds = 1; otherCmds = 2;

        simpleView = 3; standardView = 4; complexView = 5;

        wrapper = 6; specialContainer = 7; generalContainer = 8;

    VAR

        create: RECORD

            subsystem: ARRAY 9 OF CHAR;

            kind: INTEGER;

            Create: PROCEDURE

        END;

END DevSubTool.

Module *DevSubTool* provides a code generator (sometimes such a tool is called a "wizard" or "expert") which creates source code skeletons for typical view implementations. Theses skeletons are extended with your own code pieces and then compiled.

*DevSubTool* supports several kinds of projects, from simple text commands to general containers. The source document(s) are always created in the form of a new subsystem, i.e., as a subdirectory with the generic subsystem structure (*Sym*, *Code*, *Docu*, *Mod* subdirectories).

Note that the tool uses template texts which are stored in the *Dev/Rsrc/New* directory. Studying these texts, in particular the more complex model/view/commands templates, can be worthwile to learn more about typical BlackBox design and code patterns.

Typical command:

    "Create Subsystem..."    ""    "StdCmds.OpenToolDialog('Dev/Rsrc/SubTool', 'Create Subsystem')"    ""

CONST **textCmds**

This value can be assigned to *create.kind*, to create a command package for text commands, i.e., a module which imports the standard *Text* subsystem and enhances it with its own exported commands or interactors.

CONST **formCmds**

This value can be assigned to *create.kind*, to create a command package for form commands, i.e., a module which imports the standard *Form* subsystem and enhances it with its own exported commands or interactors.

CONST **otherCmds**

This value can be assigned to *create.kind*, to create a command package for arbitrary commands, i.e., a module which enhances BlackBox with its own exported commands or interactors.

CONST **simpleView**

This value can be assigned to *create.kind*, to create a view implementation for a simple view which has no model. The view and its commands are packaged into one module. The view is not exported.

CONST **standardView**

This value can be assigned to *create.kind*, to create a view implementation for a view with a model. The model, view, and its commands are packaged into one module. Model and view are not exported.

CONST **complexView**

This value can be assigned to *trans.kind*, to create a view implementation for a view with a model. The model, view, and its commands are packaged into one module each. Model and view are exported as definition types, concrete implementations are created via directory objects.

*This category is currently not supported.*

CONST **wrapper**

This value can be assigned to *trans.kind*, to create a wrapper implementation for wrapping an arbitrary view. The wrapper view and its commands are packaged into one module. The wrapper view is not exported.

*This category is currently not supported.*

CONST **specialContainer**

Creates a container with a static layout and no intrinsic contents, possibly for containing only views of a particular type. The container view and its commands are packaged into one module. The container view is not exported.

CONST **generalContainer**

Creates a container view with dynamic layout, possibly some intrinsic contents, and able to contain any view type. The model, view, controller, and its commands are packaged into one module each. Model and view are exported as definition types, concrete implementations are created via directory objects. Model, view, and controller are extensions of their base types in module *Containers*.

*This category is currently not supported.*

VAR **trans**

Interactor for the translation dialog.

**subsystem**: ARRAY 9 OF CHAR

Name of the subsystem to be translated. The name must be a legal subsystem name, between 3 to 8 characters in length, and start with a capital letter.

**kind**: INTEGER    kind IN {textCmds..generalContainer}

Kind of program to generate.

**Create**: PROCEDURE

Creation command. As input, a legal subsystem name must be entered. As a result, a new subsystem directory is created.

The *Dev/Rsrc/New* directory contains a number of template documents. *Create* translates some of these documents (depending on *kind*) by replacing all strings with the strikeout attribute (like here: strikeout) by the subsystem name. The template files are then deleted.

