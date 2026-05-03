**StdLinks**

DEFINITION StdLinks;

    IMPORT Dialog, Views;

    TYPE

        Link = POINTER TO RECORD (Views.View)

            leftSide-: BOOLEAN;

            (v: Link) GetCmd (OUT cmd: ARRAY OF CHAR), NEW

        END;

        Target = POINTER TO RECORD (Views.View)

            leftSide-: BOOLEAN;

            (t: Target) GetIdent (OUT ident: ARRAY OF CHAR), NEW

        END;

        Directory = POINTER TO ABSTRACT RECORD

            (d: Directory) NewLink (IN cmd: ARRAY OF CHAR): Link, NEW, ABSTRACT;

            (d: Directory) NewTarget (IN ident: ARRAY OF CHAR): Target, NEW, ABSTRACT

        END;

    VAR

        par-: Link;

        dir-, stdDir-: Directory;

    PROCEDURE CreateGuard (VAR par: Dialog.Par);

    PROCEDURE CreateLink;

    PROCEDURE CreateTarget;

    PROCEDURE ShowTarget (ident: ARRAY OF CHAR);

    PROCEDURE SetDir (d: Directory);

END StdLinks.

Link views, also called links, are views that always appear in pairs. They are only meaningful when embedded in texts. A pair brackets a stretch of text and contains a command. This an example of a link:

    [<u>A link to the user manual</u>](../../System/Docu/User-Man.odc.md)

If you do not see anything special to the left and right of the above text stretch, use *Text->Show Marks* to make the views visible.

If you click on either the left or right link view, the command associated with the link is executed. Moreover, the entire text stretch between the link pair is active also. The mouse cursor changes its shape when it points to the active stretch.

If you hold down the *modifier* key when clicking on one of the link stretch, the two views are transformed into a textual form:

    <StdCmds.OpenBrowser('System/Docu/User-Man', 'User Manual')><u>A link to the user manual</u><>

The syntax is "<" command sequence ">" text stretch "<" ">". The command sequence usually consists of a *StdCmds.OpenBrowser* command. However, any command may be used, e.g., like in the following specification:

    <Dialog.Beep; Dialog.Beep><u>beep beep</u><>

To turn this specification into an active text stretch, select it (from and including the first "<", to and including the last ">") and then execute *Tools->Create Link*.

You may have noticed that the name "link" thus just denotes the most typical use of link views. They are not inherently specialized for text linking. The behavior is completely determined by the command sequence associated with them.

In order to use link views for hypertext linking, it must be possible to use link commands which open a particular piece of text. The standard command for this purpose is *StdCmds.OpenBrowser*. For example, the command *"StdCmds.OpenBrowser('Obx/Docu/Sys-Map', 'Map to the Obx Subsystem')"* opens the *Obx* map text in browser mode.

Sometimes it is useful to have a command which, as a reaction to activating it via a link view, scrolls the text in which the link view is embedded, to a certain target position. This target position can be marked with a pair of target views, which are created in a similar way as link views. The command to scroll to a certain target view is *ShowTarget*. To determine which target to show, a target contains an arbitrary identifier name.

For example, the link view with the specification

    <StdLinks.ShowTarget('first target')><u>show target</u><>

creates a link to a target given by the specification

    <first target>this is the first target<>

**MENU**

    "Create Link"    ""    "StdLinks.CreateLink"    "StdLinks.CreateGuard"

    "Create Target"    ""    "StdLinks.CreateTarget"    "StdLinks.CreateGuard"

**END**

TYPE **Link (Views.View)**

View type for links.

**leftSide**-: BOOLEAN

Tells whether it is a left or a right view.

PROCEDURE (v: Link) **GetCmd** (OUT cmd: ARRAY OF CHAR)

NEW

Returns the link's command.

Post

leftSide = (cmd # "")

TYPE **Target (Views.View)**

View type for targets.

**leftSide**-: BOOLEAN

Tells whether it is a left or a right view.

PROCEDURE (t: Target) **GetIdent** (OUT ident: ARRAY OF CHAR)

NEW

Returns the target's identifier.

Post

leftSide = (ident # "")

TYPE **Directory**

ABSTRACT

Directory type for link/target views.

PROCEDURE (d: Directory) **NewLink** (IN cmd: ARRAY OF CHAR): Link

NEW, ABSTRACT

Returns a new link view with *cmd* as command string. It is a left view if *cmd # ""*, otherwise a right view.

PROCEDURE (d: Directory) **NewTarget** (IN ident: ARRAY OF CHAR): Target

NEW, ABSTRACT

Returns a new target view with *ident* as identifier string. It is a left view if *ident # ""*, otherwise a right view.

VAR **par-**: Link    par # NIL exactly during the currently executed link command

A command in a link can get access to its (left) link view, and thus to its context, via this variable during the execution of the command.

VAR **dir-, stdDir-**: Directory

Link/target directories.

PROCEDURE **CreateGuard** (VAR par: Dialog.Par)

Menu guard procedure used for *CreateLink* and *CreateTarget*. *par.disabled* remains FALSE (i.e., the menu entry is not disabled) if the following holds. The focus view is a text view and the current text selection exactly covers a stretch of text with the syntax: "<" text ">" text "<>".

PROCEDURE **CreateLink**

Insert a link into the focus text. To create a link, a piece of text with the following syntax must be selected: "<" *command sequence* ">" *arbitrary text* "<>". The *command sequence* must not contain a ">" character. The stretch "<" *command sequence* ">" is replaced with the left link view. The stretch "<>" is replaced with the right link view. The link views can be hidden/shown with *Text->Show Marks* and *Text->Hide Marks*, resp. To edit the command sequence of a link, click on one of the link views with the modifier key pressed. This replaces the views with the original text.

PROCEDURE **CreateTarget**

Insert a target into the focus text. To create a target, a piece of text with the following syntax must be selected: "<" *target identifier* ">" *arbitrary text* "<>". The target identifier must not contain a ">" character. The stretch "<" *target identifier* ">" will be replaced with the left target view. The stretch "<>" will be replaced with the right target view. Target views can be hidden/shown with *Text->Show Marks* and *Text->Hide Marks*, resp. To edit the target identifier of an existing target, click on one of the views with the modifier key pressed. This replaces the views with the original text.

PROCEDURE **ShowTarget** (ident: ARRAY OF CHAR)

Searches the first target view in the focus text whose target identifier equals *ident*. If one is found, the text is scrolled such that the target view is shown on the first line, and the text stretch between the left and right target views is selected. (Note: if the text is opened in mask mode, the selection is not visible.)

PROCEDURE **SetDir** (d: Directory)

Sets the link/target view directory.

