**DevMarkers**

DEFINITION DevMarkers;

    PROCEDURE NextError;

    PROCEDURE ToggleCurrent;

    PROCEDURE UnmarkErrors;

    ... plus some private items ...

END DevMarkers.

Error markers indicate compiler errors in-place in the compiled text.

This module contains several other items which are used internally.

Possible menu:

**MENU**

    "Unmar&k Errors"    ""    "DevMarkers.UnmarkErrors"    "TextCmds.FocusGuard"

    "Next E&rror"    ""    "DevMarkers.NextError"    "TextCmds.FocusGuard"

    "To&ggle Error Mark"    ""    "DevMarkers.ToggleCurrent"    "TextCmds.FocusGuard"

**END**

PROCEDURE **NextError**

Guard: TextCmds.FocusGuard

Move caret forward after the next error marker. If there is none, the text is scrolled to the beginning.

PROCEDURE **ToggleCurrent**

Guard: TextCmds.FocusGuard

Toggle the state of the marker before the caret.

PROCEDURE **UnmarkErrors**

Guard: TextCmds.FocusGuard

Removes all error markers.

