**Strings**

DEFINITION Strings;

    CONST

        charCode = -1; decimal = 10; hexadecimal = -2; roman = -3;

        digitspace = 8FX;

        hideBase = FALSE; showBase = TRUE;

    PROCEDURE Valid (IN s: ARRAY OF CHAR): BOOLEAN;

    PROCEDURE Extract (s: ARRAY OF CHAR; pos, len: INTEGER; OUT res: ARRAY OF CHAR);

    PROCEDURE Find (IN s: ARRAY OF CHAR; IN pat: ARRAY OF CHAR; start: INTEGER;

                                    OUT pos: INTEGER);

    PROCEDURE Replace (VAR s: ARRAY OF CHAR; pos, len: INTEGER; IN rep: ARRAY OF CHAR);

    PROCEDURE Lower (ch: CHAR): CHAR;

    PROCEDURE Upper (ch: CHAR): CHAR;

    PROCEDURE ToLower (in: ARRAY OF CHAR; OUT out: ARRAY OF CHAR);

    PROCEDURE ToUpper (in: ARRAY OF CHAR; OUT out: ARRAY OF CHAR);

    PROCEDURE IntToString (x: LONGINT; OUT s: ARRAY OF CHAR);

    PROCEDURE IntToStringForm (x: LONGINT; form, minWidth: INTEGER; fillCh: CHAR;

                                                    showBase: BOOLEAN; OUT s: ARRAY OF CHAR);

    PROCEDURE RealToString (x: REAL; OUT s: ARRAY OF CHAR);

    PROCEDURE RealToStringForm (x: REAL; precision, minW, expW: INTEGER; fillCh: CHAR;

                                                            OUT s: ARRAY OF CHAR);

    PROCEDURE StringToInt (IN s: ARRAY OF CHAR; OUT x, res: INTEGER);

    PROCEDURE StringToLInt (IN s: ARRAY OF CHAR; OUT x: LONGINT; res: INTEGER);

    PROCEDURE StringToReal (IN s: ARRAY OF CHAR; OUT x: REAL; OUT res: INTEGER);

END Strings.

Module *Strings* is a simple and small string library. Its goal is to provide a few string operations that are both often needed and complicated to implement, in particular routines for conversions between numbers and strings. The library is optimized for convenience, not for efficiency.

This tradeoff is apparent in that some operations, such as *Extract*, use value parameters instead of IN parameters. This allows to pass the same variable both for input and output purposes, which is often convenient (a variable should never be passed to several IN/OUT/VAR parameters simultaneously, since this may cause interference between them).

It is not a goal to provide operations for all possible circumstances, since string processing in different applications simply varies to much to make this practical. Often it is useful to write a few string operations fully tailored to a particular application, which is usually easy to do. Moreover, such custom string operations can be optimized for speed, which is not possible for too general routines.

Note that the language Component Pascal provides efficient built-in support for string assignment (implicitly or explicitly with the "$" operator), for string concatenation (with the "+" operator), and for counting the number of characters in a string (LEN(string$)).

CONST **charCode**

Possible value for parameter *form* of *IntToStringForm*, asking for formatting integers following the syntax of Component Pascal numerical character literals, e.g., *0DX* or *37X*.

CONST **decimal**

Possible value for parameter *form* of *IntToStringForm*, asking for formatting integers as decimal literals.

CONST **hexadecimal**

Possible value for parameter *form* of *IntToStringForm*, asking for formatting integers as hexadecimal literals.

CONST **roman**

Possible value for parameter *form* of *IntToStringForm*, asking for formatting integers as roman literals.

CONST **digitspace**

A digit space has the width of digit zero (0) which is equivalent to the width of all digits in most fonts, thus can be used for number formatting.

CONST **hideBase**, **showBase**

Possible values for parameter *showBase* of *IntToStringForm*, asking for showing / suppressing the base of the number format.

PROCEDURE  **Valid** (IN s: ARRAY OF CHAR): BOOLEAN

Returns *TRUE* if and only if the array *s* contains at least one string terminator *0X*.

Post

s contains a 0X character

    result = TRUE

s does not contain a 0X character

    result = FALSE

PROCEDURE **Upper** (ch: CHAR): CHAR

Conversion to uppercase characters. Handles the entire ISO Latin-1 character set. Character values that have no uppercase equivalent (and Unicodes outside of Latin-1) are returned unchanged.

PROCEDURE **Lower** (ch: CHAR): CHAR

Conversion to lowercase characters. Handles the entire ISO Latin-1 character set. Character values that have no lowercase equivalent (and Unicodes outside of Latin-1) are returned unchanged.

PROCEDURE **ToLower** (in: ARRAY OF CHAR; OUT out: ARRAY OF CHAR)

Converts string *in* to lowercase characters and returns the result in *out*. Handles the entire ISO Latin-1 character set. Character values that have no lowercase equivalent (and Unicodes outside of Latin-1) are unchanged. The same actual parameter may be passed for *in* and *out*.

Pre

Valid(in)    index trap

Post

Valid(out)

PROCEDURE **ToUpper** (in: ARRAY OF CHAR; OUT out: ARRAY OF CHAR)

Converts string *in*  to uppercase characters and returns the result in *out*. Handles the entire ISO Latin-1 character set. Character values that have no uppercase equivalent (and Unicodes outside of Latin-1) are unchanged. The same actual parameter may be passed for *in* and *out*.

Pre

Valid(in)    index trap

Post

Valid(out)

PROCEDURE **Extract** (s: ARRAY OF CHAR; pos, len: INTEGER; OUT res: ARRAY OF CHAR)

Extracts the stretch *[pos, MIN(pos+len, Len(s)))* from *s* and returns it in *res*. The result is truncated if *res* is not large enough. The same actual parameter may be passed for *s* and *res*.

Pre

len >= 0    20

pos >= 0    21

Valid(s)    (not checked)

Post

Valid(res)

LEN(res$) = MAX(MIN(len, LEN(s'$)-pos, LEN(res)-1), 0)

PROCEDURE **Replace** (VAR s: ARRAY OF CHAR; pos, len: INTEGER; IN rep: ARRAY OF CHAR)

Replaces the stretch *[pos, MIN(pos+len, Len(s)))* in *s* with the string in *rep*. The characters after the replaced stretch are moved if necessary. The result is truncated if *s* is not large enough.

Hint: if *len = 0* then *rep* is inserted in *s* at position *pos*. If *LEN(rep$) = 0* then the stretch *[pos, MIN(pos+len, LEN(s$)))* is deleted from *s*.

Pre

len >= 0    20

pos >= 0    21

Valid(s) & Valid(rep)    (not checked)

Post

Valid(s)

PROCEDURE **Find** (IN s: ARRAY OF CHAR; IN pat: ARRAY OF CHAR; start: INTEGER;

                                OUT pos: INTEGER);

Searches the first occurrence of the pattern *pat* in string *s* after position *start*. If the pattern is found, the position of the first character of the pattern in *s* is returned in *pos*. If the pattern is not found, *pos* is *-1*.

Pre

start >= 0    20

Valid(s) & Valid(pat)    (not checked)

Post

pattern found

    pos is start position of pat in s

pattern not found

    pos = -1

PROCEDURE  **IntToStringForm** (x: LONGINT; form, minWidth: INTEGER; fillCh: CHAR;

                                                        showBase: BOOLEAN; OUT s: ARRAY OF CHAR)

Convert integer* x* into string *s*. If *form* is *charCode* or *hexadecimal*, *x* is converted to a base 16 representation. The total representation will at least have a width of *minWidth* characters, where padding (if required) takes place to the left using characters as specified by *fillCh*.

If *showBase* is *TRUE*, a suffix character is appended to the number representation according to the number form. The value *form = charCode* renders the suffix "X", while *form = hexadecimal* renders the suffix "H". These values of *form* also represent negative integers using a base-complement form of width *minWidth*, i.e., for negative hexadecimal numbers, *fillCh* is ignored and "F" is used instead (both for *form = charCode* and *form = hexadecimal*). For *form* values in the range 2..16, base-complement representation is not supported.

E.g.

    *x* = -3, *form* = 16, *minWidth* = 4, *fillCh* = " " and *showBase* = FALSE renders a result of -3

    *x* = -3, *form* = *hexadecimal*, *minWidth* = 4, *fillCh* = " " and *showBase* = FALSE renders a result of FFFD

If *showBase* is *TRUE* and form is in the range 2..16, then the base is appended to the number, preceded by a "%" sign (e.g., "10111001%2").

If *form = roman *(roman numbers), then *showBase* is ignored.

The following conditions imply that *s* is large enough to hold the resulting string (Pre 23):

    *form = roman*: LEN(*s*) > MAX(*minWidth*, 15)

    (*form* = *charCode*) OR (*form* = *hexadecimal*) OR (*form* >= 2) & (*form* <= 16):

        LEN(*s*) > MAX(*minWidth*, 4 + <*number of digits*>)

        Where <*number of digits*> is 1 + Floor(Logbase(ABS(*x*))), if ABS(*x*) >= 1, and 1 otherwise.

Note that these values are non-tight upper bounds of the required string length. In individual cases, actual requirements might be lower but the given bounds guarantee compliance with the precondition.

Pre

(form = charCode) OR (form = hexadecimal) OR (form = roman) OR ((form >= 2) & (form <= 16))    20

(form # roman) OR (form = roman) & (x > 0) & (x < 3999)    21

minWidth >= 0    22

s is large enough to hold resulting string (see above)    23

Post

Valid(s)

PROCEDURE **IntToString** (x: LONGINT; OUT s: ARRAY OF CHAR)

Write integer in default format.

Except for performance, equivalent to:

    IntToStringForm(x, decimal, 0, digitspace, FALSE, s)

PROCEDURE **RealToStringForm** (x: REAL; precision, minW, expW: INTEGER;

                                                        fillCh: CHAR; OUT s: ARRAY OF CHAR)

Convert real* x* into string *s*. The string created to represent the number is either in fixed point or in scientific format, according to *expW*. *precision* denotes the number of valid decimal places (usually 7 for short reals and 16 for reals). *minW* denotes the minimal length in characters. If necessary, preceding *fillCh* will be inserted. Numbers are always rounded to the last valid and visible digit.

*expW* > 0: exponential format (scientific) with at least *expW* digits in the exponent.

*expW* = 0: fixpoint or floatingpoint format, depending on *x*.

*expW* < 0: fixpoint format with *-expW* digits after the decimal point.

The following conditions imply that *s* is large enough to hold the resulting string (Pre 23):

    (*x* = inf) OR (*x* = -inf) OR (*x* = nan): LEN(*s*) > MAX(*minW*, 4)

*    expW* >= 0: LEN(*s*) > MAX(*minW*, *precision* + 7)

*    expW* < 0: LEN(*s*) > MAX(*minW*, 3 - *expW *+ <*number of digits before the decimal point*>)

    Where the <*number of digits before the decimal point*> is 1 + Floor(Log10(ABS(*x*))), if ABS(*x*) >= 1,

    and 1 otherwise.

Note that these values are non-tight upper bounds of the required string length. In individual cases, actual requirements might be lower but the given bounds guarantee compliance with the precondition.

Pre

precision > 0    20

0 <= minW < LEN(s)    21

-LEN(s) < expW <= 3    22

s is large enough to hold resulting string (see above)    23

Pos

Valid(s)

PROCEDURE **RealToString** (x: REAL; OUT s: ARRAY OF CHAR)

Write real in default format.

Except for performance, equivalent to:

    RealToStringForm(x, 16, 0, 0, digitspace, s)

PROCEDURE **StringToInt** (IN s: ARRAY OF CHAR; OUT x, res: INTEGER)

PROCEDURE **StringToLInt** (IN s: ARRAY OF CHAR; OUT x: LONGINT; res: INTEGER)

Converts the number contained in string *s* into value *x*. Legal integer number representations follow the syntax given below. Possible result codes are *res = 1* for overflow, *res = 2* for syntax error.

Syntax:

  number = ( [ "+" | "-" ] dec | hex ) 0X .

  dec = digit { digit } .

  hex = hexdigit { hexdigit } ("H" | "X") .

  hexdigit = digit | "A" | "B" | "C" | "D" | "E" | "F" .

  digit = "0" |  "1" |  "2" |  "3" |  "4" |  "5" |  "6" |  "7" |  "8" |  "9" .

Post

s is legal integer number representation

    x is converted integer number

    res = 0

s is not a legal integer number representation

    res # 0

PROCEDURE **StringToReal** (IN s: ARRAY OF CHAR; OUT x: REAL; OUT res: INTEGER)

Converts string *s* given in fixed or scientific notation into value *x*. Possible result codes are *res = 1* for overflow, *res = 2* for syntax error.

Post

s is legal real number representation

    x is converted real number

    res = 0

s is not legal real number representation

    res # 0

