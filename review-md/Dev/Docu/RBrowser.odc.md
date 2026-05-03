**DevRBrowser**

DEFINITION DevRBrowser;

    PROCEDURE ShowRepository;

    PROCEDURE Update;

    PROCEDURE OpenFile (path, name: ARRAY OF CHAR);

    PROCEDURE ShowFiles (path: ARRAY OF CHAR);

END DevRBrowser.

This tool module allows to list all subsystems as folds (-> StdFolds). A fold contains links to its subsystem's symbol, code, source, and documentation files. It makes it easier to get an overview over the elements of a large BlackBox application, or over BlackBox itself.

For the resources of a subsystem, a *Rsrc* link is generated. Clicking this link creates a list of all documents in the *Rsrc* subdirectory.

For the documentation files, those which contain one or several dashes ("-") in their names are not treated as module documentations, but rather as auxiliary documentation files which are listed at the beginning. In particular, files with name *Sys-Map*, *User-Man*, and *Dev-Man* are extracted this way. They are the standard names for a map document which contains hyperlinks to the relevant files of the subsystems; the user manual for the subsystem (user in this context means a programmer); and the programmer's reference.

Menu command:

    "Repository"    ""    "DevRBrowser.ShowRepository"    ""

PROCEDURE **ShowRepository**

Create a report which lists all subsystems.

PROCEDURE **Update**

Update the report (used as command in the *update* link that is on top of the generated report).

PROCEDURE **OpenFile** (path, name: ARRAY OF CHAR)

Used internally. Opens the file at location *path* and with name *name*.

PROCEDURE **ShowFiles** (path: ARRAY OF CHAR)

Used internally. Lists all resource documents at location *path*.

