**TextCmds**

DEFINITION TextCmds;

    IMPORT Dialog;

    VAR

        find: RECORD

            find: ARRAY 256 OF CHAR;

            replace: ARRAY 256 OF CHAR;

            ignoreCase, wordBeginsWith, wordEndsWith, reverseOrientation: BOOLEAN

        END;

        ruler: RECORD

            pageBreaks: RECORD

                notInside, joinPara: BOOLEAN

            END

        END;

    PROCEDURE InitFindDialog;

    PROCEDURE SetNormalOrientation;

    PROCEDURE SetReverseOrientation;

    PROCEDURE FindFirst (option: ARRAY OF CHAR);

    PROCEDURE FindAgain (option: ARRAY OF CHAR);

    PROCEDURE Replace (option: ARRAY OF CHAR);

    PROCEDURE ReplaceAll (option: ARRAY OF CHAR);

    PROCEDURE ReplaceAndFindNext (option: ARRAY OF CHAR);

    PROCEDURE InitRulerDialog;

    PROCEDURE SetRuler;

    PROCEDURE InsertDigitSpace;

    PROCEDURE InsertNBHyphen;

    PROCEDURE InsertNBSpace;

    PROCEDURE InsertParagraph;

    PROCEDURE InsertRuler;

    PROCEDURE InsertSoftHyphen;

    PROCEDURE ShiftLeft;

    PROCEDURE ShiftRight;

    PROCEDURE Subscript;

    PROCEDURE Superscript;

    PROCEDURE MakeDefaultAttributes;

    PROCEDURE MakeDefaultRuler;

    PROCEDURE ListAlienViews;

    PROCEDURE ToggleMarks;

    PROCEDURE HideMarks;

    PROCEDURE ShowMarks;

    PROCEDURE EditGuard (VAR par: Dialog.Par);

    PROCEDURE FocusGuard (VAR par: Dialog.Par);

    PROCEDURE SelectionGuard (VAR par: Dialog.Par);

    PROCEDURE EditSelectionGuard (VAR par: Dialog.Par);

    PROCEDURE SingletonGuard (VAR par: Dialog.Par);

    PROCEDURE FindGuard (VAR par: Dialog.Par);

    PROCEDURE FindAgainGuard (VAR par: Dialog.Par);

    PROCEDURE ReplaceGuard (VAR par: Dialog.Par);

    PROCEDURE ReplaceAllGuard (VAR par: Dialog.Par);

    PROCEDURE MakeDefaultRulerGuard (VAR par: Dialog.Par);

    PROCEDURE ToggleMarksGuard (VAR par: Dialog.Par);

END TextCmds.

Command package for text views. A possible menu using commands from this package:

**MENU **"Text" ("TextViews.View")

    "&Find / Replace..."    "F"    "TextCmds.InitFindDialog; StdCmds.OpenToolDialog('Text/Rsrc/Cmds', 'Find / Replace')"    ""

    "Find &Again"    "F3"    "TextCmds.InitFindDialog; TextCmds.FindAgain('~I~B~E~R')"        "TextCmds.FindAgainGuard"

    "Find &Previous"    "*F3"    "TextCmds.InitFindDialog; TextCmds.FindAgain('~I~B~ER')"        "TextCmds.FindAgainGuard"

    "Find &Previous"    "F2"    "TextCmds.InitFindDialog; TextCmds.FindAgain('~I~B~ER')"        "TextCmds.FindAgainGuard"

    "Find First"    "F4"    "TextCmds.InitFindDialog; TextCmds.FindFirst('~I~B~E~R')"        "TextCmds.FindAgainGuard"

    "Find Last"    "*F4"    "TextCmds.InitFindDialog; TextCmds.FindFirst('~I~B~ER')"        "TextCmds.FindAgainGuard"

    **SEPARATOR**

    "Shift &Left"        ""    "TextCmds.ShiftLeft"    "TextCmds.EditGuard"

    "Shift &Right"        ""    "TextCmds.ShiftRight"    "TextCmds.EditGuard"

    "Su&perscript"        ""    "TextCmds.Superscript"    "TextCmds.SelectionGuard"

    "Su&bscript"        ""    "TextCmds.Subscript"    "TextCmds.SelectionGuard"

    **SEPARATOR**

    "&Insert Paragraph"        ""    "TextCmds.InsertParagraph"    "StdCmds.PasteCharGuard"

    "Insert R&uler"        ""    "TextCmds.InsertRuler"    "StdCmds.PasteViewGuard"

    "Insert &Soft-Hyphen"        ""    "TextCmds.InsertSoftHyphen"    "StdCmds.PasteCharGuard"

    "Insert &Non-Brk Hyphen"        ""    "TextCmds.InsertNBHyphen"    "StdCmds.PasteCharGuard"

    "Insert N&on-Brk Space"        ""    "TextCmds.InsertNBSpace"    "StdCmds.PasteCharGuard"

    "Insert &Digit Space"        ""    "TextCmds.InsertDigitSpace"    "StdCmds.PasteCharGuard"

    "Toggle &Marks"        ""    "TextCmds.ToggleMarks"    "TextCmds.ToggleMarksGuard"

    **SEPARATOR**

    "Make Default Attributes"        ""    "TextCmds.MakeDefaultAttributes"    "TextCmds.SelectionGuard"

    "Make Default Ruler"        ""    "TextCmds.MakeDefaultRuler"    "TextCmds.MakeDefaultRulerGuard"

**END**

VAR **find**: RECORD

This is the interactor for the *Find & Replace* dialog. It allows to specify several options: *ignoreCase* makes searching insensitive to case, *wordBeginsWith* restricts searching to the beginning of words, and *wordEndsWith* restricts searching to the end of words. *wordBeginsWith* and *wordEndsWith* together restrict searching to whole words. *reverseOrientation* reverses the search direction: instead of searching towards the end of the text, a search progresses towards the beginning of the text.

**find**: ARRAY 256 OF CHAR

The search string.

**replace**: ARRAY 256 OF CHAR

The replacement string.

**ignoreCase**: BOOLEAN

Determines whether searching should consider or ignore the case of a letter (small/caps).

**wordBeginsWith, wordEndsWith**: BOOLEAN

Determine whether searching should be restricted to word beginnings, word endings, or both. The latter case means that a word must match exactly.

**reverseOrientation**: BOOLEAN

Determines the search orientation, normal (forward) or reverse (backward).

This flag is considered by all search operations.

VAR **ruler**: RECORD

This is the interactor for the dialog popped up by rulers to set properties that are not normally controlled interactively.

**pageBreaks**: RECORD

The two options presently supported by the ruler interactor both affect the page breaking strategy used by text setters. These two options are grouped into the *pageBreaks* subrecord.

**notInside**: BOOLEAN

If set, this option excludes page breaks anywhere in the text following this ruler and before the next following ruler or the end of the text. However, the text will be broken across pages anyway if it doesn't fit on a single page.

**joinPara**: BOOLEAN

If set, this option prevents a page break between the text controlled by this ruler and the text controlled by the next ruler. (If there is no next ruler, this option has no effect.) As with the *notInside* option, if the text controlled by this option does not fit onto a single page, it will be broken across pages anyway.

PROCEDURE **InitFindDialog**

This initialization command sets up the *find.find* interactor field with the current text selection; if there is no current selection, *find.find* is left unmodified. Whenever *InitFindDialog* actually modifies *find.find*, it also resets the search options *find.ignoreCase*, *find.wordBeginsWith*, and *find.wordEndsWith* to FALSE. It is useful to call this command before calling one of the searching commands (*FindFirst, FindAgain*) or opening the *Find & Replace* dialog.

PROCEDURE **SetNormalOrientation**

Resets *reverseOrientation* to FALSE.

PROCEDURE **SetReverseOrientation**

Sets *reverseOrientation* to TRUE.

PROCEDURE **FindFirst** (option: ARRAY OF CHAR)

Guard: FindGuard

Searches for the first occurrence of the string *find.find* in the focused text. If the string is not found, a beep is emitted. The *ignoreCase*, *wordBeginsWith*, *wordEndsWith*, and *reverseOrientation* mode flags are taken from the *find* interactor. These interactor-determined modes can be overridden by using the *option* parameter. (*option* can be left empty if no overriding of the interactor modes is required.) The *option* string is interpreted according to the following simple syntax:

    *option* = { [~] (i | b | e | r) }

where the letters *i, b, e, *and* r* set the mode flags *ignoreCase*, *wordBeginsWith*, *wordEndsWith*, and *reverseOrientation,* respectively. (The case of these letters is ignored.) If a letter is preceded by a tilde (~), then the corresponding mode flag is reset. For example, the following option string requests case-insensitive search of a pattern that begins a word but doesn't have to end a word, where the search is to be performed from the current position backwards, that is, towards the beginning of the document:

    "ib~er"

PROCEDURE **FindAgain** (option: ARRAY OF CHAR)

Guard: FindAgainGuard

This command searches for the string *find.find*, starting from the end of the selection. If there is no selection, it searches for the string starting from the caret position. If there is no caret either, it starts from the beginning of the focused text. The *ignoreCase*, *wordBeginsWith*, *wordEndsWith*, and *reverseOrientation* mode flags are used as explained for the *FindFirst* command.

PROCEDURE **Replace** (option: ARRAY OF CHAR)

Guard: ReplaceGuard

Replace the previously found occurrence of the search string (*find.find*) by a replacement string (*find.replace*). The *ignoreCase*, *wordBeginsWith*, *wordEndsWith*, and *reverseOrientation* mode flags are used as explained for the *FindFirst* command.

PROCEDURE **ReplaceAll** (option: ARRAY OF CHAR)

Guard: ReplaceGuard

Replace all search strings (*find.find*) by a replacement string (*find.replace*), either in the entire focus text, or, if the focus text contains a selection, just in the selected range. The *ignoreCase*, *wordBeginsWith *and *wordEndsWith* mode flags are used as explained for the *FindFirst* command.

PROCEDURE **ReplaceAndFindNext** (option: ARRAY OF CHAR)

Guard: ReplaceGuard

Replace the previously found occurrence of the search string (*find.find*) by a replacement string (*find.replace*). Afterwards, try to find the next occurrence of the search string. The *ignoreCase*, *wordBeginsWith*, *wordEndsWith*, and *reverseOrientation* mode flags are used as explained for the *FindFirst* command.

PROCEDURE **InitRulerDialog**

This initialization command sets the *ruler* interactor fields *pageBreak.notInside* and *pageBreak.joinPara* to match the settings of the currently selected or focused ruler. The fields remain unchanged if there is no ruler selected or focused. It is useful to call this command before opening the auxiliary ruler dialog.

PROCEDURE **SetRuler**

Guard:

If there is a ruler selected or focused, change its *notInside* and *joinPara* options to the values set in the *ruler* interactor; otherwise do nothing.

PROCEDURE **InsertDigitSpace**

Guard: PasteCharGuard

Pastes a digit space, i.e., a space which in most fonts has the same width as a digit. Whether a digit space has this defined width depends on the font used; some fonts have digits of varying widths. Also, some font designers chose to set the digit space to half the space of a digit. (*Compatibility note:* in some older versions of the text system it was recommended practice to use digit spaces to simulate right alignment of numbers. Use right-aligning tab stops instead.)

PROCEDURE **InsertNBHyphen**

Guard: PasteCharGuard

Pastes a non-breaking hyphen, i.e., a hyphen which will not be used to break a word.

PROCEDURE **InsertNBSpace**

Guard: PasteCharGuard

Pastes a non-breaking space, i.e., a space which will not be used to break a word.

PROCEDURE **InsertParagraph**

Guard: PasteCharGuard

Pastes a paragraph character, i.e., a character indicating the beginning of a new paragraph. Regular line breaks do not start new paragraphs; for example, the first line indentation setting of a ruler only affects the first line of a paragraph introduced by a paragraph character.

PROCEDURE **InsertRuler**

Guard: PasteViewGuard

Pastes a new ruler, which is set up the same way as the ruler in the same text closest above. If there is no ruler above, the values of the default ruler are taken. (See command *MakeDefaultRuler* below.)

PROCEDURE **InsertSoftHyphen**

Guard: PasteCharGuard

Pastes a soft hyphen, i.e., a hyphen which only becomes visible if it is used to break a word.

PROCEDURE **ShiftLeft**

Guard: SelectionGuard

This command removes one *tab* character from the white space at the beginning of each line spanned by the current selection.

PROCEDURE **ShiftRight**

Guard: SelectionGuard

This command inserts one *tab* character from the white space at the beginning of each line spanned by the current selection.

PROCEDURE **Subscript**

Guard: SelectionGuard

This command moves the selected text down vertically, into a subscript position.

PROCEDURE **Superscript**

Guard: SelectionGuard

This command moves the selected text up vertically, into a superscript position.

PROCEDURE **MakeDefaultAttributes**

Guard: SelectionGuard

Sets the focus text's default attributes to the ones of the current selection in this text.

PROCEDURE **MakeDefaultRuler**

Guard: SingletonGuard

Sets the focus text's default ruler values to the ones of the currently selected ruler in this text.

PROCEDURE **ListAlienViews**

Guard: FocusGuard

Opens a text containing the list of alien views contained in this text.

PROCEDURE **ToggleMarks**

Guard: ToggleMarksGuard

This command makes text rulers and paragraph characters visible if they aren't, and hides them if they are.

PROCEDURE **HideMarks**

Guard: FocusGuard

This command hides text rulers and paragraph characters if they are visible.

PROCEDURE **ShowMarks**

Guard: FocusGuard

This command makes text rulers and paragraph characters visible if they aren't.

PROCEDURE **EditGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current focus is not a text view or doesn't allow editing, i.e., doesn't allow for setting a caret.

PROCEDURE **FocusGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current focus is not a text view.

PROCEDURE **SelectionGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current focus is not a text view or doesn't contain a selection.

PROCEDURE **EditSelectionGuard** (VAR par: Dialog.Par)

Same as *SelectionGuard*, except that the text view that contains the selection must be in an editable mode (modes of *->Containers*).

PROCEDURE **SingletonGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current focus is not a text view or doesn't contain a singleton selection, i.e., a selection of a single embedded view.

PROCEDURE **FindGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current focus is not a text view or if the *find.find* interactor holds an empty string, i.e., no search target is set.

PROCEDURE **FindAgainGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current focus is not a text view or if the *find.find* interactor holds an empty string *and* the focused text doesn't contain a selection.

PROCEDURE **ReplaceGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current focus is not a text view or if the *find.find* interactor holds an empty string (replacement target cannot possibly match) or if the focused text doesn't allow editing or if the focused text doesn't contain a selection (nothing to replace).

PROCEDURE **ReplaceAllGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current focus is not a text view or if the *find.find* interactor holds an empty string (replacement target cannot possibly match) or if the focused text doesn't allow editing. Otherwise, this guard sets the label depending on whether the focused text contains a selection ("Replace all in selection") or not ("Replace all in text").

PROCEDURE **MakeDefaultRulerGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current focus is not a text view or doesn't contain a selected ruler.

PROCEDURE **ToggleMarksGuard** (VAR par: Dialog.Par)

This guard disables the current menu item if the current focus isn't a text view. Furthermore it sets up the correct string of the item (*Show Marks* / *Hide Marks*).

