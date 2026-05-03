**StdClocks**

DEFINITION StdClocks;

    IMPORT Views;

    PROCEDURE Deposit;

    PROCEDURE New (): Views.View;

END StdClocks.

This module implements a simple clock view.

Typical menu command:

    "Insert Clock"    ""    "StdClocks.Deposit; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

PROCEDURE **New** (): Views.View

Factory function for standard clocks.

PROCEDURE **Deposit**

Deposit command for standard clocks.

