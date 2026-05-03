**StdHeaders**

DEFINITION StdHeaders;

    IMPORT Dialog, Views, Fonts, Properties;

    CONST alternate = 0; number = 1; head = 2; foot = 3; showFoot = 4;

    TYPE

        Prop = POINTER TO RECORD (Properties.Property)

            alternate: BOOLEAN;

            showFoot: BOOLEAN;

            number: NumberInfo;

            head, foot: Banner

        END;

        Banner = RECORD

            left, right: ARRAY 128 OF CHAR;

            gap: INTEGER

        END;

        NumberInfo = RECORD

            new: BOOLEAN;

            first: INTEGER

        END;

    VAR

        dialog: RECORD

            alternate, showFoot: BOOLEAN;

            number: NumberInfo;

            head, foot: Banner

        END;

    PROCEDURE New (p: Prop; f: Fonts.Font): Views.View;

    PROCEDURE Deposit;

    PROCEDURE InitDialog;

    PROCEDURE NewNumberGuard (VAR par: Dialog.Par);

    PROCEDURE AlternateGuard (VAR par: Dialog.Par);

    PROCEDURE HeaderGuard (VAR par: Dialog.Par);

    PROCEDURE Set;

END StdHeaders.

This module implements views that can be embedded in BlackBox texts, to provide page headers and/or footers upon printing.

Typical menu command:

    "Insert Header"    ""    "StdHeaders.Deposit; StdCmds.PasteView"    "TextCmds.FocusGuard"

A header in a text is invisible, except if *Text -> Show Marks* has been executed. Selecting a header view and then executing *Edit -> Show Properties...* (or double-clicking on the view) opens a property sheet that allows to set the following properties:

- "alternate": if checked, this option causes left and right pages to be distinguished, so that they can have different headers and footers.

- "Show Foot": if checked, this option makes the header view display the footer, otherwise it displays the header (when it is visible at all).

- "Start with new page number": if checked, the user can enter the page number for the page in which the view is embedded. It allows to change page numbering within a single document.

- "Right Head", "Right Foot", "Left Head", "Left Foot": these are the properties of the right / left headers / footers. They are (one-line) strings. Optionally, these strings may contain macros, as described below.

- "Head gap", "Foot gap": determines the distance between header/footer and the text area, in points.

- "Font...": allows to set the font that is used for the headers and footers.

The following macros can be used in the header/footer strings:

    &p - replaced by current page number as arabic numeral

    &r - replaced by current page number as roman numeral

    &R - replaced by current page number as capital roman numeral

    &a - replaced by current page number as alphanumeric character

    &A - replaced by current page number as capital alphanumeric character

    &d - replaced by printing date

    &t - replaced by printing time

    && - replaced by & character

    &; - specifies split point

    &f - filename with path/title

These macros are evaluated upon printing (or upon display, when the header view is visible). The macro "&;" indicates a split point, which is a position in the string where the string can be broken apart to allow for a better layout. For example, the string "Left margin&;Right margin" allows the string part "Left margin" to be positioned at the left side of the page, and the string "Right margin" at the right side of the page. The split point becomes a kind of "elastic spring" that pushes the left and right string parts apart.

CONST **alternate, number, head, foot, showFoot**

Property numbers for the properties that are supported by header views.

TYPE **Prop (Properties.Property)**

Property descriptor for header views.

**alternate**: BOOLEAN

Determines whether left and right pages have their own headers/footers.

**showFoot**: BOOLEAN

Determines whether a visible header view shows the header or the footer.

**number**: NumberInfo

Determines the way in which page numbers are shown.

**head, foot**: Banner

Determines the way in which headers and footers are shown.

TYPE **Banner**

Determines the header or footer of a page (or pair of pages if "alternate" is chosen).

**left, right**: ARRAY 128 OF CHAR

Strings for the left and right header/footer. Supports macros as described at the beginning of this text.

**gap**: INTEGER    gap >= 0

Distance between header/footer and the body of the text.

TYPE **NumberInfo**

Determines the way pages are numbered.

**new**: BOOLEAN

Determines whether page numbering starts anew, using the page number *first*.

**first**: INTEGER    first >= 0

The starting page number that is used for the page in which the header view is embedded.

VAR **dialog**

Interactor for the property sheet.

**alternate**: BOOLEAN

Determines whether left and right pages have their own headers/footers.

**showFoot**: BOOLEAN

Determines whether a visible header view shows the header or the footer.

**number**: NumberInfo

Determines the way in which page numbers are shown.

**head, foot**: Banner

Determines the way in which headers and footers are shown.

PROCEDURE **New** (p: Prop; f: Fonts.Font): Views.View

Create a new header view with the given properties.

PROCEDURE **Deposit**

Deposit command for header views.

The following procedures are used for the property sheet (*Std/Rsrc/Headers*).

PROCEDURE **InitDialog**

PROCEDURE **NewNumberGuard** (VAR par: Dialog.Par)

PROCEDURE **AlternateGuard** (VAR par: Dialog.Par)

PROCEDURE **HeaderGuard** (VAR par: Dialog.Par)

PROCEDURE **Set**

