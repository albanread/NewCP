**StdScrollers**

DEFINITION StdScrollers;

    IMPORT Dialog, Properties;

    CONST

        horBar = 0; verBar = 1; horHide = 2; verHide = 3; width = 4; height = 5; savePos = 7; showBorder = 6;

    TYPE

        Prop = POINTER TO RECORD (Properties.Property)

            horBar, verBar, horHide, verHide: BOOLEAN;

            width, height: INTEGER;

            showBorder, savePos: BOOLEAN

        END;

    VAR

        dialog: RECORD

            horizontal, vertical: RECORD

                mode: INTEGER;

                adapt: BOOLEAN;

                size: REAL

            END;

            showBorder, savePos: BOOLEAN

        END;

    PROCEDURE AddScroller;

    PROCEDURE RemoveScroller;

    PROCEDURE InitDialog;

    PROCEDURE Set;

    PROCEDURE DialogGuard (VAR par: Dialog.Par);

    PROCEDURE HeightGuard (VAR par: Dialog.Par);

    PROCEDURE WidthGuard (VAR par: Dialog.Par);

    PROCEDURE HorAdaptGuard (VAR par: Dialog.Par);

    PROCEDURE VerAdaptGuard (VAR par: Dialog.Par);

END StdScrollers.

Module *StdScrollers* provides a wrapper view which can be wrapped around any other view to provide it with a horizontal and/or vertical scrollbar. For example, a text view in a form may be wrapped in a scroller in order to allow the scrolling of text that doesn't completely fit in the text view.

CONST **horBar, verBar, horHide, verHide, width, height, savePos, showBorder**

Property elements of the *Prop* descriptor.

TYPE **Prop** (Properties.Property)

Properties describing the attributes of a scroller view.

**horBar**: BOOLEAN

Is there a horizontal scrollbar?

**verBar**: BOOLEAN

Is there a vertical scrollbar?

**horHide**: BOOLEAN    horHide -> horBar

Only valid if *horBar*. This leaves three legal possibilities:

~horBar                    scrollbar is never visible

horBar  &  horHide    scrollbar is only visible when needed (appears or disappears automatically)

horBar  &  ~horHide    scrollbar is always visible

**verHide**: BOOLEAN    verHide -> verBar

Only valid if *verBar*. This leaves three legal possibilities:

~verBar                    scrollbar is never visible

verBar  &  verHide    scrollbar is only visible when needed (appears or disappears automatically)

verBar  &  ~verHide    scrollbar is always visible

**width, height**: INTEGER    width >= 0  &  height >= 0    [units]

Size of the wrapped view.

A value of 0 means that the wrapped view automatically adapts its size to the scroller (wrapper) view. Other values are fixed sizes.

**showBorder**: BOOLEAN

Display a border around the wrapped view.

**savePos**: BOOLEAN

Save the current scroll position when the wrapper is saved to disk, and makes scrolling undoable.

VAR

**dialog**: RECORD

Interactor for setting the scroller properties of a singleton selection.

**horizontal, vertical**: RECORD

Descriptor of scrollbar behavior for both dimensions.

    **mode**: INTEGER    mode IN {0, 1, 2}

        0: never a scrollbar

        1: automatic scrollbar

        2: always a scrollbar

    **adapt**: BOOLEAN

        adapt: set size (width or height) to 0

        ~adapt: retain a fixed size

    **size**: REAL

        size in cm (*Dialog.metricSystem*) or in inches (*~Dialog.metricSystem*)

**showBorder, savePos**: BOOLEAN

Values according to property descriptor.

PROCEDURE **AddScroller**

Guard: StdCmds.SingletonGuard

Wraps a scroller around the selected view.

PROCEDURE **RemoveScroller**

Guard: StdCmds.SingletonGuard

Removes the scroller which is selected.

PROCEDURE **InitDialog**

Initializes variable *dialog* according to the selected scroller view.

PROCEDURE **Set**

Applies the newly defined properties to the selected scroller view.

PROCEDURE **DialogGuard** (VAR par: Dialog.Par)

PROCEDURE **HeightGuard** (VAR par: Dialog.Par)

PROCEDURE **WidthGuard** (VAR par: Dialog.Par)

PROCEDURE **HorAdaptGuard** (VAR par: Dialog.Par)

PROCEDURE **VerAdaptGuard** (VAR par: Dialog.Par)

Various guards for the *Std/Rsrc/Scrollers* dialog box.

