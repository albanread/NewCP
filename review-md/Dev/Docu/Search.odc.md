**DevSearch**

DEFINITION DevSearch;

    PROCEDURE Compare;

    PROCEDURE SearchInDocu (opts: ARRAY OF CHAR);

    PROCEDURE SearchInSources;

    PROCEDURE SelectCaseInSens (pat: ARRAY OF CHAR);

    PROCEDURE SelectCaseSens (pat: ARRAY OF CHAR);

END DevSearch.

This tool provides global search facilities (in all subsystems' *Docu* or *Mod* directories) and a text comparison feature. The search engine cooperates with module *TextCmds*, in that it uses the latter's find & replace interactor. This means that after having found a string using one of the search commands, the text command "Find Again" can be used to conveniently find further occurrences of the same string in the same text.

Typical menu:

**MENU**

    "Search In Sources"    ""    "TextCmds.InitFindDialog; DevSearch.SearchInSources"    "TextCmds.SelectionGuard"

    "Search In Docu (Case Sensitive)"    ""    "TextCmds.InitFindDialog; DevSearch.SearchInDocu('s')"    "TextCmds.SelectionGuard"

    "Search In Docu (Case Insensitive)"    ""    "TextCmds.InitFindDialog; DevSearch.SearchInDocu('i')"    "TextCmds.SelectionGuard"

    "Compare Texts"    ""    "DevSearch.Compare"    "TextCmds.FocusGuard"

**END**

PROCEDURE **Compare**

Guard: top two windows are document windows

Perform a textual comparison of the two topmost windows' contents. The comparison starts at each window's current caret position, or alternatively, at the end of its selection. The next difference which is found is indicated by advancing the caret or selection to the found difference. White space (spaces, tabs, carriage returns) are ignored during comparison.

PROCEDURE **SearchInSources**

Guard: TextCmds.SelectionGuard

Search all available sources (in all subsystems) for the occurrence of the selected text pattern. Search is case-sensitive.

PROCEDURE **SearchInDocu** (opts: ARRAY OF CHAR)

Guard: TextCmds.SelectionGuard

Search all available documentation texts (in all subsystems and in *Manuals*) for the occurrence of the selected text pattern. If the string *opts* starts with an 's' or 'S' then the search is case sensitive, if it starts with an 'i' or 'I' the search is case-insensitive and in all other cases the value from the Find dialog is used.

PROCEDURE **SelectCaseInSens** (pat: ARRAY OF CHAR)

Sets up *TextCmds.find.find* with *pat*, calls *TextCmds.FindFirst* and opens the find dialog. This procedure is used by link views created with the above procedure *SearchInDocu *when the search was performed case-insensitive.

PROCEDURE **SelectCaseSens** (pat: ARRAY OF CHAR)

Sets up *TextCmds.find.find* with *pat* and calls *TextCmds.FindFirst*. This procedure is used by link views created with the above procedures *SearchInSource* and *SearchInDocu *when the search was performed case-sensitive.

