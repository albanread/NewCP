**StdLog**

DEFINITION StdLog;

    IMPORT TextViews, Views, TextRulers, TextModels;

    CONST

        charCode = -1; decimal = 10; hexadecimal = -2;

        hideBase = FALSE; showBase = TRUE;

    VAR

        buf-: TextModels.Model;

        defruler-: TextRulers.Ruler;

        dir-: TextViews.Directory;

        text-: TextModels.Model;

    PROCEDURE Bool (x: BOOLEAN);

    PROCEDURE Char (ch: CHAR);

    PROCEDURE Int (i: INTEGER);

    PROCEDURE IntForm (x: LONGINT; base, minWidth: INTEGER; fillCh: CHAR; showBase: BOOLEAN);

    PROCEDURE Real (x: REAL);

    PROCEDURE RealForm (x: REAL; precision, minW, expW: INTEGER; fillCh: CHAR);

    PROCEDURE Set (x: SET);

    PROCEDURE String (IN str: ARRAY OF CHAR);

    PROCEDURE ParamMsg (IN msg, p0, p1, p2: ARRAY OF CHAR);

    PROCEDURE Msg (IN msg: ARRAY OF CHAR);

    PROCEDURE Tab;

    PROCEDURE Para;

    PROCEDURE Ln;

    PROCEDURE View (v: Views.View);

    PROCEDURE ViewForm (v: Views.View; w, h: INTEGER);

    PROCEDURE New;

    PROCEDURE Open;

    PROCEDURE Clear;

    PROCEDURE NewView (): TextViews.View;

    PROCEDURE SetDefaultRuler (ruler: TextRulers.Ruler);

    PROCEDURE SetDir (d: TextViews.Directory);

END StdLog.

Module *StdLog* provides a log text and procedures that simplify writing into the log text. Typically, log windows are only used during development, not for end user environments. The log window is opened by the following statement in procedure *Config.Setup*:

    Dialog.Call("StdLog.Open", "", res)

CONST **charCode**

Possible value for parameter *base* of *IntForm*, asking for formatting integers following the syntax of Component Pascal numerical character literals. (For example, 0DX is the code for *line*, and 37X the code for "7".)

CONST **decimal**

Possible value for parameter *base* of *IntForm*, asking for formatting integers as decimal literals.

CONST **hexadecimal**

Possible value for parameter *base* of *IntForm*, asking for formatting integers as hexadecimal literals.

CONST **hideBase**

Possible value for parameter *showBase* of *IntForm*, asking for suppression of the base indicator.

CONST **showBase**

Possible value for parameter *showBase* of *IntForm*, asking for output of the base indicator.

PROCEDURE **Bool** (x: BOOLEAN)

Writes a Boolean value to the log.

Except for performance, equivalent to:

    IF x THEN String("$TRUE") ELSE String("$FALSE") END

PROCEDURE **Char** (ch: CHAR)

Writes a character value to the log. For control characters the numerical literal form enclosed in spaces is written (e.g., " 9X " for a tab code).

Note that it is much more efficient to use *String* if more than one character needs to be written.

PROCEDURE **Int** (i: INTEGER)

Writes an integer value to the log.

Except for performance, equivalent to:

    IntForm(x, decimal, 0, digitspace, showBase)

where *digitspace* = 8FX.

PROCEDURE **IntForm** (x: LONGINT; base, minWidth: INTEGER; fillCh: CHAR; showBase: BOOLEAN)

Write integer *x*. The numeral string used to represent the number is relative to base *base*. The total representation form will at least have a width of *minWidth* characters, where padding (if required) takes place to the left using characters as specified by *fillCh*. If non-decimal, the base can be requested to form part of the representation using *showBase*. The special value *base* = *charCode* renders the base suffix "X", while *base* = *hexadecimal* renders the suffix "H". All other non-decimal bases are represented by a trailing "%" followed by the decimal numerical literal representing the base value itself. Non-decimal representations of negative integers are formed using a base-complement form of width *minWidth*. E.g., *x* = -3 renders for *base* = 16 and *minWidth* = 2 as "FD".

Pre

(base = charCode) OR (base = hexadecimal) OR ((base >= 2) & (base <= 16))    20

minWidth >= 0    22

PROCEDURE **Real** (x: REAL)

Writes a real value to the log.

Except for performance, equivalent to:

    WriteRealForm(x, 16, 0, 0, digitspace)

where *digitspace* = 8FX.

PROCEDURE **RealForm** (x: REAL; precision, minW, expW: INTEGER; fillCh: CHAR)

Write real *x*. The numeral string used to represent the number is either in fixed point or in scientific format, according to *expW*. *precision* denotes the number of valid decimal places (usually 7 for short reals and 16 for reals). *minW* denotes the minimal length in characters. If necessary, preceding *fillCh* will be inserted. Numbers are always rounded to the last valid and visible digit.

*expW* > 0: exponential format (scientific) with at least *expW* digits in the exponent.

*expW* = 0: fixpoint or floatingpoint format, depending on *x*.

*expW* < 0: fixpoint format with *-expW* digits after the decimal point.

For more details, see also the description of *Strings.RealToStringForm*.

Pre

precision > 0    20

0 <= minW    21

expW <= 3    22

PROCEDURE **Set** (x: SET)

Writes a set value to the log.

Except for performance, equivalent to:

    VAR i: INTEGER;

    Char("{"); i := MIN(SET);

    WHILE x # {} DO

        IF i IN x THEN

            Int(i); EXCL(x, i);

            IF (i + 2 <= MAX(SET)) & (i + 1 IN x) & (i + 2 IN x) THEN

                String("..");

                x := x - {i + 1, i + 2}; INC(i, 3);

                WHILE (i <= MAX(SET)) & (i IN x) DO EXCL(x, i); INC(i) END;

                Int(i - 1)

            END;

            IF x # {} THEN String(", ") END

        END;

        INC(i)

    END;

    Char("}")

PROCEDURE **String** (IN str: ARRAY OF CHAR)

Writes a string value to the log.

PROCEDURE **ParamMsg** (IN msg, p0, p1, p2: ARRAY OF CHAR)

Writes a parameterized message string value mapped by the *Dialog.MapParamString* facility to the log.

PROCEDURE **Msg** (IN msg: ARRAY OF CHAR)

Writes a message string value mapped by the *Dialog.MapParamString* facility to the log.

Except for performance, equivalent to:

    ParamMsg(msg, "", "", "")

PROCEDURE **Tab**

Writes a tab character (9X) to the log.

PROCEDURE **Para**

Writes a paragraph character (0EX) to the log. Afterwards, it makes sure that the current end of the log text is visible.

PROCEDURE **Ln**

Writes a carriage return character (0DX) to the log. Afterwards, it makes sure that the current end of the log text is visible.

PROCEDURE **View** (v: Views.View)

Writes a view to the log. The size of the view is determined by the text container in cooperation with the view itself.

PROCEDURE **ViewForm** (v: Views.View; w, h: INTEGER)

Writes a view to the log, and forces it to a particular size given in *w* and *h* in universal units.

PROCEDURE **New**

Used internally.

PROCEDURE **Open**

Opens a log window if none is open, otherwise it brings the log window to the top.

PROCEDURE **Clear**

Clears the log text.

PROCEDURE **NewView** (): TextViews.View

Used internally.

PROCEDURE **SetDefaultRuler** (ruler: TextRulers.Ruler)

Sets the default ruler.

Pre

ruler # NIL    20

Post

defruler = ruler

PROCEDURE **SetDir** (d: TextViews.Directory)

Set up the directory object for log text views.

Pre

d # NIL    20

Post

dir = d

VAR **buf**-: TextModels.Model

The buffer used internally for minimizing screen refreshes.

VAR **defruler**-: TextRulers.Ruler

The log text's default ruler.

VAR **dir**-: TextViews.Directory

Used internally.

VAR **text**-: TextModels.Model

The log text.

