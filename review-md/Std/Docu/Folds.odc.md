**StdFolds**

DEFINITION StdFolds;

    IMPORT TextModels, Views, Dialog;

    CONST expanded = FALSE; collapsed = TRUE;

    TYPE

        Label = ARRAY 32 OF CHAR;

        Fold = POINTER TO RECORD (Views.View)

            leftSide-, collapsed-: BOOLEAN;

            label-: Label;

            (fold: Fold) Flip, NEW;

            (fold: Fold) FlipNested, NEW;

            (fold: Fold) HiddenText (): TextModels.Model, NEW;

            (fold: Fold) MatchingFold (): Fold, NEW

        END;

        Directory = POINTER TO ABSTRACT RECORD

            (d: Directory) New (collapsed: BOOLEAN; label: Label;

                                        hiddenText: TextModels.Model): Fold, NEW, ABSTRACT

        END;

    VAR

        foldData: RECORD

            nested, all: BOOLEAN;

            findLabel, newLabel: Label

        END;

        dir-, stdDir-: Directory;

    PROCEDURE CollapseFolds (text: TextModels.Model; nested: BOOLEAN; IN label: ARRAY OF CHAR);

    PROCEDURE ExpandFolds (text: TextModels.Model; nested: BOOLEAN; IN label: ARRAY OF CHAR);

    PROCEDURE Collapse;

    PROCEDURE Expand;

    PROCEDURE ZoomOut;

    PROCEDURE ZoomIn;

    PROCEDURE Overlaps (text: TextModels.Model; beg, end: INTEGER): BOOLEAN;

    PROCEDURE Insert (text: TextModels.Model; label: Label; beg, end: INTEGER; collapsed: BOOLEAN);

    PROCEDURE CreateGuard (VAR par: Dialog.Par);

    PROCEDURE Create (state: INTEGER);

    PROCEDURE SetDir (d: Directory);

    PROCEDURE FindFirstFold;

    PROCEDURE FindNextFold;

    PROCEDURE CollapseLabel;

    PROCEDURE ExpandLabel;

    PROCEDURE SetLabel;

    PROCEDURE SetLabelGuard (VAR p: Dialog.Par);

    PROCEDURE FindLabelGuard (VAR par: Dialog.Par);

END StdFolds.

Fold views, also called folds, are views that always appear in pairs. They are only meaningful when embedded in texts. A pair, called the left and right fold, brackets a stretch of text and represent a second piece of text that is hidden. By clicking at a fold with the mouse, the stretch of text between the left and right fold is replaced with the hidden text, and the text that originally appeared between the folds becomes the hidden text. Clicking a second time at the same fold restores the original state. Because the primary use of folds is to hide longer stretches of text and replace them with a usually much shorter placeholder, a fold is said to be either in expanded or collapsed state. Try it!

    text between collapsed folds

Folds can be nested, but the stretch between one pair of folds must not partially overlap another. Hierarchically nested folds are often used in program texts. By hiding a sequence of statements and writing a short comment between the collapsed folds, the resulting program text can be explored interactively in a top-down manner. Try it with this example!

    PROCEDURE Enter (id: INTEGER; name: TextMappers.String; value: REAL);

    enter a new value into the list

Instead of clicking manually through a deep hierarchy of folds, you can hold down the modifier key while clicking at the fold. This expands or collapses a fold and all nested folds.

The following menu entries operate on folds. *Create Fold* in menu *Tools* inserts a new pair of folds in the focus text. The current text selection in the focus text will be bracketed by the newly inserted folds. *Expand All* in menu *Tools* expands all folds in the focus text. Similarly, *Collapse All* collapses all folds in the focus text.

*Fold...* in menu *Tools* opens a property inspector that lets you manipulate the folds in the focus text. The inspector looks like this:

*Find First* searches in the focus text for the first occurence of a fold. If *All* is checked then the the very first fold in the text is selected. If *All* is not checked then *Find First* searches for a fold which has a label equal to the search criteria specified in the *Find* field. *Find Next* finds the next fold using the same criteria as *Find First*.

*Collapse* collapses all folds in the focus text which comply to the search critera specified by *All* and *Find*. If *Nested* is checked the folds are searched for contained folds and these are also collapsed.

*Expand* expands folds using *All*, *Find* and *Nested* in the same way as *Collapse*.

When a fold is found or when a fold in the focus text is selected by the user its label is displayd in the field next to the *Set Label* button. The label can be changed by typing a new label into the field and clicking on *Set Label*.

To describe the pre- and postconditions of procedures exported from *StdFolds*, we use the following pseudo-procedures.

Pos(f)    designates the position of the fold view f in its hosting text.

0 <= Pos(f) < text.Length().

Stretch(p1, p2)     stands for the text stretch [p1, p2) if p1 < p2 holds, or if p2 <= p1, then it denotes the text stretch [p2, p1).

Flipped(f)     denotes the fold f in its dual, "flipped" state.

f.state = collapsed <=> Flipped(f).state = expanded.

Typical menu commands (typically in the *Tools* menu, some commands may be omitted in the standard distribution):

**MENU**

    "Create Fold"    ""    "StdFolds.Create(1)"    "StdFolds.CreateGuard"

    "Expand All"    ""    "StdFolds.Expand"    ""

    "Collapse All"    ""    "StdFolds.Collapse"    ""

    "Fold..."    ""    "StdCmds.OpenToolDialog('Std/Rsrc/Folds', 'Zoom')"    ""

**END**

CONST **collapsed, expanded**

Possible values of field *Fold.collapsed*.

TYPE **Label**

String type for label property values.

TYPE **Fold (Views.View)**

View type of a fold.

**leftSide**-: BOOLEAN

Determines whether the view is the left or right element of a pair.

**collapsed**-: BOOLEAN

Determines whether the fold view currently is in expanded or collapsed state.

**label**-: Label

A string indicating the label property of the fold. If the fold has no label associated with it, *label = ""* holds.

PROCEDURE (fold: Fold) **Flip**

NEW

Changes the state of fold. The text stretch S between *fold* and *fold.MatchingFold()* is replaced by the text *fold.HiddenText()*. The stretch S will be the new hidden text.

Pre

fold # NIL    20

Post

(fold.MatchingFold() # NIL) & fold.collapsed'

    ~fold.collapsed

(fold.MatchingFold() # NIL) & ~fold.collapsed'

    fold.collapsed

fold.MatchingFold() = NIL

    no effect

PROCEDURE (fold: Fold) **FlipNested**

NEW

Changes the state of fold, and all fold views f between *fold* and *fold.MatchingFold()* for which *f.state = fold.state*.

Pre

fold # NIL    20

Post

(fold.MatchingFold() # NIL) & fold.collapsed'

    ~fold.collapsed

    For all folds f between fold and fold.MatchingFold(): ~f.collapsed

(fold.MatchingFold() # NIL) & ~fold.collapsed'

    fold.collapsed

    For all folds f between fold and fold.MatchingFold(): f.collapsed

fold.MatchingFold() = NIL

    no effect

PROCEDURE (fold: Fold) **HiddenText** (): TextModels.Model

NEW

Returns the text stretch that is currently hidden by the pair *(fold, fold.MatchingFold())*. The text should not be modified. If the hidden text stretch is of length zero, *NIL* is returned.

Pre

fold # NIL    20

Post

fold.MatchingFold() # NIL

    [ let p1 := Pos(Flipped(fold)), p2 := Pos(Flipped(fold).MatchingFold() ]

    ABS(p2 - p1) = 1  =>  result = NIL

    ABS(p2 - p1) > 1  =>  result is Stretch(p1, p2)

fold.MatchingFold() = NIL

    result = NIL

PROCEDURE (fold: Fold) **MatchingFold** (): Fold

NEW

Returns the matching fold view to *fold*, or *NIL* if none can be found.

Pre

fold # NIL    20

Post

~(fold.context IS TextModels.Context)

    result = NIL

fold.context IS TextModels.Context

    (fold.kind = left) & (result.kind = right) & Pos(fold) < Pos(result)

            & (For all folds f with Pos(fold) < Pos(f) < Pos(result):

                    (f.MatchingFold() # NIL) & (Pos(fold) < Pos(f.MatchingFold() < Pos(result))

    OR (fold.kind = right) & (result.kind = left) & (Pos(fold) > Pos(result)

            & (For all folds f with Pos(result) < Pos(f) < Pos(fold):

                    (f.Matching # NIL) & (Pos(result) < Pos(f.MatchingFold() < Pos(fold))

    OR result = NIL

TYPE **Directory**

ABSTRACT

Directory for fold views.

PROCEDURE (d: Directory) **New** (collapsed: BOOLEAN; IN label: Label;

                                                    hiddenText: TextModels.Model): Fold

NEW, ABSTRACT

Create a new fold view in state *collapsed*, with *label*, and with the text *hiddenText*. If *hiddenText* is *NIL*, it is a right fold, otherwise it is a left fold.

Post

result.leftSide = (hiddenText # NIL)

result.collapsed = collapsed

result.label = label

result.HiddenText() = hiddenText

PROCEDURE **CollapseFolds** (text: TextModels.Model; nested: BOOLEAN;

                                                    IN label: ARRAY OF CHAR)

If nested holds and *label = ""*, all folds f in text with *~f.collapsed* are flipped. If *~nested*, only the outermost folds in the nesting hierarchy are flipped. If *label # "*", only folds with *f.label = label* are flipped.

Pre

text # NIL    20

PROCEDURE **ExpandFolds** (text: TextModels.Model; nested: BOOLEAN;

                                                IN label: ARRAY OF CHAR)

If nested holds and *label = ""*, all folds f in text with *f.collapsed* are flipped. If *~nested*, only the outermost folds in the nesting hierarchy are flipped. If *label # ""*, only folds with *f.label = label* are flipped.

Pre

20        text # NIL

PROCEDURE **Collapse**

Guard: TextCmds.FocusGuard

Collapse all fold views in the focus text.

PROCEDURE **Expand**

Guard: TextCmds.FocusGuard

Expand all fold views in the focus text.

PROCEDURE **ZoomOut**

Guard: TextCmds.FocusGuard

Collapse all outermost expanded fold views in the focus text.

PROCEDURE **ZoomIn**

Guard: TextCmds.FocusGuard

Expand all outermost collapsed views in the focus text.

PROCEDURE **Overlaps** (text: TextModels.Model; beg, end: INTEGER): BOOLEAN

Returns *TRUE* if the text stretch *[beg, end)* in text partially overlaps a pair *(f, f.MatchingFold())* of fold views.

Pre

text # NIL    20

(beg >= 0) & (end <= text.Length()) & (beg <= end)    21

PROCEDURE **Insert** (text: TextModels.Model; IN label: Label; beg, end: INTEGER; collapsed: BOOLEAN)

Inserts a new pair (f1, f2) of fold views into text. The new pair will bracket the text stretch *[beg, end)*. Flag *collapsed* determines whether the fold is inserted in collapsed or in expanded state.

Pre

text # NIL    20

(beg >= 0) & (end <= text.Length()) & (beg <= end)    21

Post

the text stretch [beg, end) does not partially overlap a pair of folds

    Pos(f1) = beg

    Pos(f2) = end+1

    f1.MatchingFold() = f2

    f1.collapsed = f2.collapsed

    f1.HiddenText() = NIL

the text stretch [beg, end) partially overlaps a pair of folds

    nothing is done

PROCEDURE **CreateGuard** (VAR par: Dialog.Par)

Sets *par.disabled* to *TRUE* when the text selection in the current focus text partially overlaps a pair *(f, Matching(f))* of fold views. *par.disabled*  is also set to *TRUE* if the focus text is shown in browser mode or mask mode, that is if the text cannot be modified.

PROCEDURE **Create** (state: INTEGER)

If *CreateGuard* holds, creates a a new pair of fold views and inserts them into the focus text, bracketing the current text selection or the caret position. Calls *Insert(FocusText, selBeg, selEnd, state)*.

*state = 0* generates a collapsed fold, *state = 1* generates an expanded fold.

Pre

state IN {0, 1)

Post

state = 0

    result.collapsed

state = 1

    ~result.collapsed

PROCEDURE **SetDir** (d: Directory)

Set directory.

The following variables and procedures are only used for the property inspector of the *StdFolds.View*:

VAR

    foldData: RECORD

        nested, all: BOOLEAN;

        findLabel, newLabel: Label

    END;

PROCEDURE FindFirstFold;

PROCEDURE FindNextFold;

PROCEDURE CollapseLabel;

PROCEDURE ExpandLabel;

PROCEDURE SetLabel;

PROCEDURE SetLabelGuard (VAR p: Dialog.Par);

PROCEDURE FindLabelGuard (VAR par: Dialog.Par);

