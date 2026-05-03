**TextMappers**

DEFINITION TextMappers;

    IMPORT Views, TextModels;

    CONST

        returnCtrlChars = 1; returnQualIdents = 2; returnViews = 3;

        interpretBools = 4; interpretSets = 5;

        maskViews = 6;

        char = 1; string = 3; int = 4; real = 5;

        bool = 6; set = 7; view = 8; tab = 9; line = 10; para = 11;

        lint = 16;

        eot = 30; invalid = 31;

        charCode = -1; decimal = 10; hexadecimal = -2;

        hideBase = FALSE; showBase = TRUE;

    TYPE

        String = ARRAY 256 OF CHAR;

        Scanner = RECORD

            opts-: SET;

            rider-: TextModels.Reader;

            type: INTEGER;

            start, lines, paras: INTEGER;

            char: CHAR;

            int, base: INTEGER;

            lint: LONGINT;

            real: REAL;

            bool: BOOLEAN;

            set: SET;

            len: INTEGER;

            string: String;

            view: Views.View; w, h: INTEGER;

            (VAR s: Scanner) ConnectTo (text: TextModels.Model), NEW;

            (VAR s: Scanner) SetPos (pos: INTEGER), NEW;

            (VAR s: Scanner) SetOpts (opts: SET), NEW;

            (VAR s: Scanner) Pos (): INTEGER, NEW;

            (VAR s: Scanner) Skip (OUT ch: CHAR), NEW;

            (VAR s: Scanner) Scan, NEW

        END;

        Formatter = RECORD

            rider-: TextModels.Writer;

            (VAR f: Formatter) ConnectTo (text: TextModels.Model), NEW;

            (VAR f: Formatter) SetPos (pos: INTEGER), NEW;

            (VAR f: Formatter) Pos (): INTEGER, NEW;

            (VAR f: Formatter) WriteChar (x: CHAR), NEW;

            (VAR f: Formatter) WriteInt (x: LONGINT), NEW;

            (VAR f: Formatter) WriteSString (x: ARRAY OF SHORTCHAR), NEW;

            (VAR f: Formatter) WriteString (x: ARRAY OF CHAR), NEW;

            (VAR f: Formatter) WriteReal (x: REAL), NEW;

            (VAR f: Formatter) WriteBool (x: BOOLEAN), NEW;

            (VAR f: Formatter) WriteSet (x: SET), NEW;

            (VAR f: Formatter) WriteTab, NEW;

            (VAR f: Formatter) WriteLn, NEW;

            (VAR f: Formatter) WritePara, NEW;

            (VAR f: Formatter) WriteView (v: Views.View), NEW;

            (VAR f: Formatter) WriteIntForm (x: LONGINT;

                base, minWidth: INTEGER; fillCh: CHAR; showBase: BOOLEAN), NEW;

            (VAR f: Formatter) WriteRealForm (x: REAL;

                precision, minW, expW: INTEGER; fillCh: CHAR), NEW;

            (VAR f: Formatter) WriteViewForm (v: Views.View; w, h: INTEGER), NEW;

            (VAR f: Formatter) WriteParamMsg (msg, p0, p1, p2: ARRAY OF CHAR), NEW;

            (VAR f: Formatter) WriteMsg (msg: ARRAY OF CHAR), NEW

        END;

    PROCEDURE IsQualIdent (IN s: ARRAY OF CHAR): BOOLEAN;

    PROCEDURE ScanQualIdent (VAR s: Scanner; OUT x: ARRAY OF CHAR; OUT done: BOOLEAN);

END TextMappers.

*TextMappers* are mappers that use text riders to scan and format structured text.

CONST **returnCtrlChars**

Option element

Possible element of *Scanner.opts*. If present, the scanner will return *tab*, *line*, and *para* characters; otherwise these control characters are treated as white space and read over.

CONST **returnQualIdents**

Option element

Possible element of *Scanner.opts*. If present, the scanner will return "qualified identifiers" as a single string; otherwise, the name and period parts of the qualified identifier will be returned individually. (A qualified string, as defined by the language Component Pascal follows the syntax *name ["." name]*.)

CONST **returnViews**

Option element

Possible element of *Scanner.opts*. If present, the scanner will return embedded views; otherwise these are treated as white space and read over.

CONST **interpretBools**

Option element

Possible element of *Scanner.opts*. If present, the scanner will recognize Boolean truth values "$TRUE" and "$FALSE", as output by the formatter when writing Boolean values; otherwise "$", "TRUE", and "FALSE" are returned individually, without interpretation.

CONST **interpretSets**

Option element

Possible element of *Scanner.opts*. If present, the scanner will recognize set values: sets of integers in the range MIN(SET) .. MAX(SET) as defined by the language Component Pascal; otherwise "{", ".", and enclosed integers will be returned individually. (The syntax of set values is *{" integer [".." integer "]" { "," integer [".." integer] "}*.)

CONST **maskViews**

Option element

Possible element of *scanner.opts*. If present, the scanner will try to interpret a view as a character code, if the view has a preferred character code. Otherwise, the view is returned.

CONST **char**

Possible value of *scanner.type*, signalling that a plain character has been scanned. A character is returned in this class if it does not form a valid first character of any of the structured scan types below.

CONST **string**

Possible value of *scanner.type*, signalling that a string has been scanned.

CONST **int**

Possible value of *scanner.type*, signalling that an integer has been scanned.

CONST **real**

Possible value of *scanner.type*, signalling that a real has been scanned.

CONST **bool**

Possible value of *scanner.type*, signalling that a Boolean has been scanned.

CONST **set**

Possible value of *scanner.type*, signalling that a set has been scanned.

CONST **view**

Possible value of *scanner.type*, signalling that an embedded view has been scanned.

CONST **tab**

Possible value of *scanner.type*, signalling that a *tab* character has been scanned.

CONST **line**

Possible value of *scanner.type*, signalling that a *line* character has been scanned.

CONST **para**

Possible value of *scanner.type*, signalling that a *para* character has been scanned.

CONST **lint**

Possible value of *scanner.type*, signalling that a longint has been scanned.

CONST **eot**

Possible value of *scanner.type*, signalling that the most recent call to *Scan* hit the end of the text.

CONST **invalid**

Possible value of *scanner.type*, signalling that the most recent call to *Scan* encountered a syntactically ill formed sequence.

CONST **charCode**

Possible value for parameter *base* of *formatter.WriteIntForm*, asking for formatting integers following the syntax of Component Pascal numerical character literals. (For example, 0DX is the code for *line*, and 37X the code for "7".)

CONST **decimal**

Possible value for parameter *base* of *formatter.WriteIntForm*, asking for formatting integers as decimal literals.

CONST **hexadecimal**

Possible value for parameter *base* of *formatter.WriteIntForm*, asking for formatting integers as hexadecimal literals.

CONST **hideBase**

Possible value for parameter *showBase* of *formatter.WriteIntForm*, asking for suppression of the base indicator.

CONST **showBase**

Possible value for parameter *showBase* of *formatter.WriteIntForm*, asking for output of the base indicator.

TYPE **String**

Strings of characters as detectable by scanners.

TYPE **Scanner**

Scanners are connectable to texts. They allow to scan the sequence of characters and embedded views which form a text for recognized structured subsequences (symbols). The various symbols that a scanner can recognize are defined in terms of scan types (cf. the constants above).

**opts**-: SET

The scanning options, drawn from the set {*returnCtrlChars*, *returnQualIdents*, *returnViews*, *interpretBools*, *interpretSets*, *maskViews*}.

**rider**-: TextModels.Reader

The rider connecting the scanner to the text. The rider state is used by the scanner as a single element look-ahead buffer. A sequence of *rider.Read* or *rider.ReadPrev*, or positioning the rider followed by *rider.Read* are all legal manipulations of that look-ahead state.

**type**: INTEGER

Type of symbol scanned most recently. One of *char*, *string*, *int*, *real*, *bool*, *set*, *view*, *tab*, *line*, *para*, *lint, eot*, or *invalid*.

**start**: INTEGER

Starting position of the symbol scanned most recently. Set by *scanner.Scan* after skipping initial white space.

**lines**, **paras**: INTEGER

Number of lines (*line* characters) and paragraphs (*para* characters) passed by the scanner since being connected. Updated by *scanner.Skip* (called initially in *scanner.Scan*) when skipping white space.

**char**: CHAR    valid if type = char

Character scanned most recently. The string representation of the scanned character is available in *string *after scanning.

**int**: INTEGER    valid iff type = int

Integer scanned most recently. The string representation of the scanned integer is available in *(len*, *string)* after scanning.

**base**: INTEGER    valid iff type IN {int, lint}

The base that was used to format the most recently scanned integer or longint.

**lint**: LONGINT    valid iff type IN {int, lint}

Longint scanned most recently. The string representation of the scanned longint is available in *(len*, *string)* after scanning.

**real**: REAL    valid iff type = real

Real scanned most recently.

**bool**: BOOLEAN    valid iff type = bool

Boolean scanned most recently. The string representation of the scanned Boolean is available in *string* after scanning.

**set**: SET    valid iff type = set

Set scanned most recently.

**len**: INTEGER    valid iff type IN {string, int, lint}

Length of *string* field.

**string**: String    valid iff type IN {string, int, lint, bool, char}

String of characters scanned most recently. To force a number to be scanned as a string, it must be enclosed in a pair of (double or single) quotes (e.g., if it starts with digits and thus would otherwise be interpreted as a number).

**view**: Views.View; w, h: INTEGER    valid iff type = view

View scanned most recently, and its width and height.

PROCEDURE (VAR s: Scanner) **ConnectTo** (text: TextModels.Model)

Disconnect the scanner from the text it was connected to previously (if any), and connect the scanner to the given text (if any).

Post

text = NIL

    s.rider = NIL

text # NIL

    s.rider.Base() = text

    s.Pos() = 0

    s.opts = {}

PROCEDURE (VAR s: Scanner) **Pos** (): INTEGER

Current position of the scanner's look-ahead rider.

Pre

s.rider # NIL    (not explicitly checked)

Post

result = s.rider.Pos()

PROCEDURE (VAR s: Scanner) **SetPos** (pos: INTEGER)

Reposition the scanner.

Pre

s.rider # NIL    (not explicitly checked)

preconditions of s.rider.SetPos

Post

s.Pos() = pos

s.start = pos

s.lines = 0

s.paras = 0

s.type = invalid

PROCEDURE (VAR s: Scanner) **SetOpts** (opts: SET)

Set scanning options.

Post

s.opts = opts

PROCEDURE (VAR s: Scanner) **Skip** (VAR ch: CHAR)

Skip white space, as specified by the scanning options picked from {*returnCtrlChars*, *returnViews*}.

Pre

s.rider # NIL    (not explicitly checked)

Post

~s.rider.eot

    s.start = s.rider.Pos() - 1

s.rider.eot

    s.start = s.rider.Base().Length()

    s.type = eot

PROCEDURE (VAR s: Scanner) **Scan**

Scan the text for the next symbol as specified by the scanning options.

Pre

s.rider # NIL    (not explicitly checked)

Post

s.rider.eot  OR  s.Pos() = s.start + Length(symbol) + 1

TYPE **Formatter**

Formatters connectable to texts in order to write formatted entities to the text.

**rider**-: TextModels.Writer

The rider connecting the formatter to the text.

PROCEDURE (VAR f: Formatter) **ConnectTo** (text: TextModels.Model)

Disconnect the formatter from the text it was previously connected to (if any), and connect it to the given text (if any).

Post

text = NIL

    f.rider = NIL

text # NIL

    f.rider # NIL

    f.rider.Base() = text

    f.Pos() = text.Length()

PROCEDURE (VAR f: Formatter) **Pos** (): INTEGER

Position of the formatter.

Pre

f.rider # NIL    (not explicitly checked)

PROCEDURE (VAR f: Formatter) **SetPos** (pos: INTEGER)

Reposition the formatter.

Pre

f.rider # NIL    (not explicitly checked)

Post

f.Pos() = pos

PROCEDURE (VAR f: Formatter) **WriteChar** (x: CHAR)

Write character *x*. For control characters the numerical literal form enclosed in spaces is written.

Pre

f.rider # NIL    (not explicitly checked)

Post

x >= " "  &  x # 7FX

    character written as is

x < " "  OR  x = 7FX

    " " code(x) " " written

PROCEDURE (VAR f: Formatter) **WriteInt** (x: LONGINT)

Write integer in default format.

Except for performance, equivalent to:

    f.WriteIntForm(x, decimal, 0, TextModels.digitspace, showBase)

PROCEDURE (VAR f: Formatter) **WriteReal** (x: REAL)

Write real in default format.

Except for performance, equivalent to:

    f.WriteRealForm(x, 7, 0, 0, TextModels.digitspace)

PROCEDURE (VAR f: Formatter) **WriteString** (x: ARRAY OF CHAR)

Write string of characters.

Except for performance, equivalent to:

    VAR i: INTEGER;

    i := 0; WHILE x[i] # 0X DO f.WriteChar(x[i]); INC(i) END

PROCEDURE (VAR f: Formatter) **WriteSString** (x: ARRAY OF SHORTCHAR)

Write string of short characters.

Except for performance, equivalent to:

    VAR i: INTEGER;

    i := 0; WHILE x[i] # 0 DO f.WriteChar(x[i]); INC(i) END

PROCEDURE (VAR f: Formatter) **WriteBool** (x: BOOLEAN)

Write Boolean.

Except for performance, equivalent to:

    IF x THEN f.WriteString("$TRUE") ELSE f.WriteString("$FALSE") END

PROCEDURE (VAR f: Formatter) **WriteSet** (x: SET)

Write set.

Except for performance, equivalent to:

    VAR i: INTEGER;

    f.WriteChar("{"); i := MIN(SET);

    WHILE x # {} DO

        IF i IN x THEN

            f.WriteInt(i); EXCL(x, i);

            IF (i + 2 <= MAX(SET)) & (i + 1 IN x) & (i + 2 IN x) THEN

                f.WriteString("..");

                x := x - {i + 1, i + 2}; INC(i, 3);

                WHILE (i <= MAX(SET)) & (i IN x) DO EXCL(x, i); INC(i) END;

                f.WriteInt(i - 1)

            END;

            IF x # {} THEN f.WriteString(", ") END

        END;

        INC(i)

    END;

    f.WriteChar("}")

PROCEDURE (VAR f: Formatter) **WriteTab**

Write *tab* character.

Except for performance, equivalent to:

    f.rider.WriteChar(TextModels.tab)

PROCEDURE (VAR f: Formatter) **WriteLn**

Write *line* character.

Except for performance, equivalent to:

    f.rider.WriteChar(TextModels.line)

PROCEDURE (VAR f: Formatter) **WritePara**

Write *para* character.

Except for performance, equivalent to:

    f.rider.WriteChar(TextModels.para)

PROCEDURE (VAR f: Formatter) **WriteView** (v: Views.View)

Embed view.

Except for performance, equivalent to:

    f.WriteViewForm(v, Views.undefined, Views.undefined)

Pre

v # NIL    20

v.context = NIL    21

PROCEDURE (VAR f: Formatter) **WriteIntForm** (x: LONGINT; base, minWidth: INTEGER;

                                                                                        fillCh: CHAR; showBase: BOOLEAN)

Write integer *x*. The numeral string used to represent the number is relative to base *base*. The total representation form will at least have a width of *minWidth* characters, where padding (if required) takes place to the left using characters as specified by *fillCh*. If non-decimal, the base can be requested to form part of the representation using *showBase*. The special value *base* = *charCode* renders the base suffix "X", while *base* = *hexadecimal* renders the suffix "H". All other non-decimal bases are represented by a trailing "%" followed by the decimal numerical literal representing the base value itself. Non-decimal representations of negative integers are formed using a base-complement form of width *minWidth*. E.g., *x* = -3 renders for *base* = 16 and *minWidth* = 2 as "FD". For negative hexadecimal numbers, *fillCh* is ignored and "F" is used instead.

For more details, see also the description of *Strings.IntToStringForm*.

Pre

f.rider # NIL    (not explicitly checked)

(base = charCode) OR (base = hexadecimal) OR ((base >= 2) & (base <= 16))    20

minWidth >= 0    22

PROCEDURE (VAR f: Formatter) **WriteRealForm** (x: REAL; precision, minW,

                                                                                            expW: INTEGER; fillCh: CHAR)

Write real *x*. The numeral string used to represent the number is either in fixed point or in scientific format, according to *expW*. *precision* denotes the number of valid decimal places (usually 7 for short reals and 16 for reals). *minW* denotes the minimal length in characters. If necessary, preceding *fillCh* will be inserted. Numbers are always rounded to the last valid and visible digit.

*expW* > 0: exponential format (scientific) with at least *expW* digits in the exponent.

*expW* = 0: fixpoint or floatingpoint format, depending on *x*.

*expW* < 0: fixpoint format with *-expW* digits after the decimal point.

For more details, see also the description of *Strings.RealToStringForm*.

Pre

f.rider # NIL    (not explicitly checked)

precision > 0    20

0 <= minW    21

expW <= 3    22

PROCEDURE (VAR f: Formatter) **WriteViewForm** (v: Views.View; w, h: INTEGER)

Embed a view with width *w* and height *h*. *w* and *h* may have the value *Views.undefined*.

Pre

f.rider # NIL    (not explicitly checked)

v # NIL    20

v.context = NIL    21

PROCEDURE (VAR f: Formatter) **WriteParamMsg** (msg, p0, p1, p2: ARRAY OF CHAR)

Write a parameterized message string mapped by the *Dialog.MapParamString* facility. The resulting string is allowed to contain *line*, *para* and *tab* characters, all of which will be written as such.

Pre

f.rider # NIL    (not explicitly checked)

PROCEDURE (VAR f: Formatter) **WriteMsg** (msg: ARRAY OF CHAR)

Write a message string mapped by the *Dialog.MapParamString* facility.

Except for performance, equivalent to:

    f.WriteParamMsg(msg, "", "", "")

PROCEDURE **IsQualIdent** (VAR s: ARRAY OF CHAR): BOOLEAN

Test whether the string *s* fulfills the syntax of a Component Pascal qualident, i.e. *ident ["." ident]*.

PROCEDURE **ScanQualIdent** (VAR s: Scanner; VAR x: ARRAY OF CHAR;

                                                        VAR done: BOOLEAN)

Assuming that the scanner returned a string, check if the succeeding symbols can be consumed to scan a qualident. If the scanned string is not a qualident, the scanner is reset the position it had before the call to *ScanQualIdent*.

Post

s'.type = string

    IsQualIdent(s'.string)

        done = TRUE

        x = s'.string

        s = s'

    ~IsQualIdent(s'.string)

        s'.Scan.type = char  &  s'.Scan.char = "."

            s'.Scan.Scan.type = string  &  (s'.len + 1 + s'.Scan.Scan.len < LEN(x))

                done = TRUE

                x = s'.string + "." + s'.Scan.Scan.string

                s = s'.Scan.Scan

            s'.Scan.Scan.type # string  OR  (s'.len + 1 + s'.Scan.Scan.len >= LEN(x))

                done = FALSE

        s'.Scan.type # char  OR  s'.Scan.char # "."

            done = FALSE

s'.type # string

        done = FALSE

~done

    s = s'.SetPos(s'.start).Scan()

