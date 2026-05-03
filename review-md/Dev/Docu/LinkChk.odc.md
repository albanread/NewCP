**DevLinkChk**

DEFINITION DevLinkChk;

    IMPORT Dialog;

    CONST oneSubsystem = 0; globalSubsystem = 1; allSubsystems = 2;

    VAR

        par: RECORD

            scope: INTEGER;

            subsystem: ARRAY 9 OF CHAR

        END;

    PROCEDURE Check (subsystem: ARRAY OF CHAR; scope: INTEGER; check: BOOLEAN);

    PROCEDURE CheckLinks;

    PROCEDURE ListLinks;

    PROCEDURE Open (path, file: ARRAY OF CHAR; pos: INTEGER);

    PROCEDURE SubsystemGuard (VAR p: Dialog.Par);

    PROCEDURE CommandGuard (VAR p: Dialog.Par);

    PROCEDURE SubsystemNotifier (op, from, to: INTEGER);

END DevLinkChk.

This tool module allows to check whether a hyperlink (-> StdLinks) points to an existing BlackBox document. this allows to find most "stale" links, i.e., links to non-existing files.

Menu command:

    "Check Links..."    ""    "StdCmds.OpenAuxDialog('Dev/Rsrc/LinkChk', 'Check Links')"    ""

CONST **oneSubsystem , globalSubsystem, allSubsystems**

Checking can occur in one specific subsystem (more exactly, in the files contained in the subsystem's *Docu* and *Mod* subdirectories); in the global *Docu* and *Mod* directories, or in these directories plus the *Docu* and *Mod* directories of all subsystems.

VAR **par**: RECORD

Interactor for *CheckLinks* and *ListLinks*. It defines what documents these commands operate on.

**scope**: INTEGER    scope IN {oneSubsystem, globalSubsyste, allSubsystems}

The scope in which checking will occur, i.e., the directory or directories whose *Docu* and *Mod* subdirectories will be searched for files with links.

**subsystem**: ARRAY 9 OF CHAR    valid iff scope = oneSubsystem

Subsystem name. Only legal if *scope = oneSubsystem*.

PROCEDURE **Check** (subsystem: ARRAY OF CHAR; scope: INTEGER; check: BOOLEAN)

Used internally.

PROCEDURE **CheckLinks**

Guard: CommandGuard

Check all links in the scope defined by the *par* interactor. Checking means that it is tested whether the target file of the following link-commands exists:

    StdCmds.OpenMask

    StdCmds.OpenBrowser

    StdCmds.OpenDoc

    StdCmds.OpenAuxDialog

Links that are stale are listed in a report text. For every stale link, the report contains one link that directly opens the culpable text and scrolls to the offending link view (using the *Open* command below).

PROCEDURE **ListLinks**

Guard: CommandGuard

Lists all links in the scope defined by the par interactor. Links are listed in a report text. For every link, the report contains one link that directly opens the relevant text and scrolls to the link view (using the *Open* command below).

PROCEDURE **Open** (path, file: ARRAY OF CHAR; pos: INTEGER)

Used internally. The procedure opens a file at location *path* and name *file*; scrolls to position *pos*; and selects the range *[pos .. pos+1[*.

PROCEDURE **SubsystemGuard** (VAR p: Dialog.Par)

PROCEDURE **CommandGuard** (VAR p: Dialog.Par)

PROCEDURE **SubsystemNotifier** (op, from, to: INTEGER)

Various guards and notifiers used for dialog *Dev/Rsrc/LinkChk*.

