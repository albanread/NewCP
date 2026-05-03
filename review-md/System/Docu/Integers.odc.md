**Integers**

DEFINITION Integers;

    IMPORT Files;

    TYPE Integer = POINTER;

    PROCEDURE Abs (x: Integer): Integer;

    PROCEDURE Compare (x, y: Integer): INTEGER;

    PROCEDURE ConvertFromString (IN s: ARRAY OF CHAR; OUT x: Integer);

    PROCEDURE ConvertToString (x: Integer; OUT s: ARRAY OF CHAR);

    PROCEDURE Difference (x, y: Integer): Integer;

    PROCEDURE Digits10Of (x: Integer): INTEGER;

    PROCEDURE Entier (x: REAL): Integer;

    PROCEDURE Externalize (w: Files.Writer; x: Integer);

    PROCEDURE Float (x: Integer): REAL;

    PROCEDURE GCD (x, y: Integer): Integer;

    PROCEDURE Internalize (r: Files.Reader; OUT x: Integer);

    PROCEDURE Long (x: LONGINT): Integer;

    PROCEDURE Power (x: Integer; exp: INTEGER): Integer;

    PROCEDURE Product (x, y: Integer): Integer;

    PROCEDURE QuoRem (x, y: Integer; OUT quo, rem: Integer);

    PROCEDURE Quotient (x, y: Integer): Integer;

    PROCEDURE Remainder (x, y: Integer): Integer;

    PROCEDURE Short (x: Integer): LONGINT;

    PROCEDURE Sign (x: Integer): INTEGER;

    PROCEDURE Sum (x, y: Integer): Integer;

    PROCEDURE ThisDigit10 (x: Integer; exp10: INTEGER): CHAR;

END Integers.

Module *Integer* implements an abstract data type *Integer* to represent arbitrary precision integer numbers. It also offers the most important arithmetical operations on such numbers and a variety of conversion operations. The arithmetical operations include summation, multiplication, powering, computations of quotients, remainders, and greatest common divisors.

With respect to assignment and procedure parameters, variables of type *Integers.Integer* can be used in the same way as variables of the numerical types built into the language. Of course, to perform operations with such variables, special procedures have to be called. The "operators" +, -, *, DIV, MOD, etc. cannot be used . The same holds for comparisons.

Note: Though the language syntax allows two variables of type *Integers.Integer* to be compared using the "="-operator, the result is not what you would expect. Instead of values, pointers to objects representing the values are compared. Thus, the comparison may yield the result *FALSE*, although the values of the variables are equal. Hence, use the *Integers.Compare*-function instead of the "="-operator.

The individual values are represented by objects on the heap rather than the stack. Clients of module *Integers* will use references to these objects only. Values of existing objects cannot be changed (they are immutable). Thus, copying such values is completely safe; it is not possible to inadvertantly change a value via an alias pointer.

The space needed to represent a small integer is that of the minimum object size of your Component Pascal implementation. To represent a large value, the required memory is proportional to the number of decimal digits in the value. Each of the operations offered by module *Integers* allocates memory necessary to represent its result, but no extra memory will be allocated beyond that (space for intermediate results is allocated on the stack).

Examples of client modules:

[<u>ObxFact docu</u>](../../Obx/Docu/Fact.odc.md)    calculate factorials

[<u>ObxFract docu</u>](../../Obx/Docu/RatCalc.odc.md)    calculatior/simplifier for rational numbers

TYPE **Integer**

Opaque

Opaque type to represent integers of arbitrary size. Values of this type are allocated on the heap.  The required memory size depends on the number of decimals to be represented. The objects are immutable.

PROCEDURE **Long** (x: LONGINT): Integer

*Long* generates a new *Integer* from a *LONGINT* variable.

PROCEDURE **Entier** (x: REAL): Integer

*Entier* generates a new *Integer* from a *REAL* variable. *Entier* rounds similar to the *ENTIER*-function of the Component Pascal programming language; both implement the floor-function.

PROCEDURE **Short** (x: Integer): LONGINT

*Short* converts a value of type *Integer* into a *LONGINT*.

Pre

MIN(LONGINT) <= x <= MAX(LONGINT)

PROCEDURE **Float** (x: Integer): REAL

*Float* converts a value of type *Integer* into a *REAL*.

Pre

MIN(REAL) <= x <= MAX(REAL)

PROCEDURE **Sum** (x, y: Integer): Integer

PROCEDURE **Difference** (x, y: Integer): Integer

PROCEDURE **Product** (x, y: Integer): Integer

PROCEDURE **Quotient** (x, y: Integer): Integer

PROCEDURE **Remainder** (x, y: Integer): Integer

PROCEDURE **QuoRem** (x, y: Integer; VAR quo, rem: Integer)

PROCEDURE **Power** (x: Integer; exp: INTEGER): Integer

PROCEDURE **GCD** (x, y: Integer): Integer

PROCEDURE **Abs** (x: Integer): Integer

The arithmetic operations *Sum*, *Difference*, *Product*, *Quotient*, *Remainder*, *Power*, *GCD* (= greatest common divisor), and *Abs* (= absolute value) are defined as to be expected. In particular, *Quotient* and *Remainder* are defined according the Component Pascal rules for *DIV* and *MOD*. If both quotient and remainder need to be computed, for performance reasons the procedure *QuoRem* should be called instead of the individual functions *Quotient* and *Remainder*. *Power* requires the exponent to be non-negative.

Pre (Quotient, Remainder, QuoRem)

y # 0

Pre (Power)

exp >= 0

PROCEDURE **Compare** (x, y: Integer): INTEGER

Compares the values of *x* and *y*. With this function, all comparison relations can be built: to compute the value of (*x op y*) write (*Compare(x, y) op 0*), where *op* is one of =, #, <, <=, >, >=.

Post

x < y

    result < 0

x = y

    result = 0

x > y

    result > 0

PROCEDURE **Sign** (x: Integer): INTEGER

The sign of *x*.

Post

x > 0

    result = 1

x = 0

    result = 0

x < 0

    result = -1

PROCEDURE **Digits10Of** (x: Integer): INTEGER

The number of decimal digits needed to represent *x*.

Exception: for *x = 0* the result is the value 0.

PROCEDURE **ThisDigit10** (x: Integer; exp10: INTEGER): CHAR

This *Digit10* returns a single decimal digit as a character.

Pre

exp10 >= 0    20

Post

"0" <= result <= "9"

exp10 >= Digits10Of(x)

    result = "0"

PROCEDURE **ConvertFromString** (IN s: ARRAY OF CHAR; OUT x: Integer)

PROCEDURE **ConvertToString** (x: Integer; OUT s: ARRAY OF CHAR)

*ConvertFromString* and *ConvertToString* are used to read an *Integer* from a string resp. to write an *Integer* to a string. *ConvertToString* requires that the string is long enough to represent the *Integer*.

Pre (ConvertToString)

(Sign(x) >= 0) & (LEN(s) >= Digits10Of(x) + 1)

OR

(Sign(x) < 0) & (LEN(s) >= Digits10Of(x) + 2)

PROCEDURE **Internalize** (r: Files.Reader; OUT x: Integer)

PROCEDURE **Externalize** (w: Files.Writer; x: Integer)

*Internalize* and *Externalize* are used to read from resp. to write to files.

