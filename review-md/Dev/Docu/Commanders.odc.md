**DevCommanders**

DEFINITION DevCommanders;

    TYPE

        Par = POINTER TO RECORD

            text: TextModels.Model;

            beg, end: INTEGER

        END;

    VAR

        par: Par;

    PROCEDURE Deposit;

    PROCEDURE DepositEnd;

    ... plus some private items ...

END DevCommanders.

A commander is a view which interprets and executes the command or command sequence written behind the commander. It only operates when embedded in a text.

Commanders can be useful during development; e.g., they may be embedded directly in the source code of a program. They are not intended for use in end-user applications, due to their non-standard user interface experience.

Typical menu:

"Insert Commander"    ""    "DevCommanders.Deposit; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

"Insert EndCommander"    ""    "DevCommanders.DepositEnd; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

TYPE **Par**

Parameter context of a command's execution. This is used with commands such as

    *DevCompiler.CompileThis ObxViews1 ObxViews2*

**text**: TextModels.Model

The text containing the activated commander and command.

**beg**: INTEGER

Begin of the parameters to the command, that is, the text position immediately behind the last character of the command itself.

**end**: INTEGER

End of the parameters to the command. This is either the end of the text or the position of the next commander or *EndView* in the text.

VAR **par-**: Par    par # NIL exactly during the currently executed command

A command can get access to the text, and thus to its context, via this variable during the execution of the command.

PROCEDURE **Deposit**

Deposit command for commanders.

PROCEDURE **DepositEnd**

Deposit command for a *DevCommanders.EndView*. Marks the end of the command for the preceding commander. If no end view is present the commander reads until another commander is found or until the text ends.

This module contains several other items which are used internally.

