**Appendix B: Differences between Pascal and Component Pascal**

**Eliminated Features**

**ꀢ Subrange types**

Use a standard integer type instead.

**ꀢ Enumeration types**

Use integer constants instead.

**ꀢ Arbitrary array ranges**

Arrays are always defined over the integer range 0..max-1.

Example

    A = ARRAY 16 OF INTEGER    (* legal indices are in the range 0..15 *)

**ꀢ No general sets**

Type SET denotes the integer set which may include the elements 0..31.

**ꀢ No explicit DISPOSE**

Memory is reclaimed automatically by the garbage collector. Instead of calling DISPOSE, simply set the variable to NIL.



**ꀢ No variant records**

Use record extension instead.

**ꀢ No packed structures**

 Use SHORTCHAR or BYTE types for byte-sized values.

**ꀢ No GOTO**

**ꀢ No PRED and SUCC standard functions**

Use DEC or INC on integer values instead.

**ꀢ No built-in input/output facilities**

No file types. I/O is provided by library routines.

**Changed Features**

**ꀢ Standard procedure ENTIER instead of ROUND**

**ꀢ Syntax for REAL constants**

3.0E+4 but not 3.0e+4

**ꀢ Syntax for pointer type declarations**

P = POINTER TO R

instead of

P = ^R

**ꀢ Syntax for case statement**

"|" instead of ";" as case separator.

ELSE clause.

Example

    CASE i * 3 - 1 OF

      0: StdLog.String("zero")

    | 1..9: StdLog.String("one to nine")

    | 10, 20: StdLog.String("ten or twenty")

    ELSE StdLog.String("something else")

    END

**ꀢ Procedure name must be repeated**

Example

    PROCEDURE DrawDot (x, y: INTEGER);

    BEGIN

    END DrawDot;

**ꀢ Case is significant**

Small letters are distinguished from capital letters.

Example    "proc" is not the same as "Proc".

**ꀢ String syntax**

String literals are either enclosed between " or between '. There cannot be both single and double quotes in one string. String literals of length one are assignment-compatible to character variables.

Examples

    "That's great"    'Write "hello world" to the screen'

    ch := "x"

    ch := 'x'

**ꀢ Comments**

Comments are enclosed between (* and *) and may be nested.

**ꀢ Set brackets**

Set constants are given between { and } instead of [ and ].

Example    {0..2, 4, j..2 * k}

**ꀢ Function syntax**

Use keyword PROCEDURE for functions also, instead of FUNCTION.

Procedures with a return value always have a (possibly empty) parameter list in their declarations and in calls to them.

The function result is returned explicitly by a RETURN statement, instead of an assignment to the function name.

Example

    PROCEDURE Fun (): INTEGER;

    BEGIN

        RETURN 5

    END Fun;

     instead of

    FUNCTION Fun: INTEGER;

    BEGIN

        Fun := 5

    END;

    n := Fun()   instead of   n := Fun

**ꀢ Declarations**

The sequence of declarations is

{ ConstDecl | TypeDecl | VarDecl} {ProcDecl | ForwardDecl}

instead of

[ConstDecl] [TypeDecl] [VarDecl] {ProcDecl}.

Forward declarations are necessary if a procedure is used before it is defined.

Example

    PROCEDURE ^ Proc;

    instead of

    PROCEDURE Proc; FORWARD;

**ꀢ Procedure types**

Procedures may not only be passed to parameters, but also to procedure-typed variables.

Example

    TYPE P = PROCEDURE (x, y: INTEGER);

    VAR v: P;

    v := DrawDot;    (* assign *)

    v(3, 5);    (* call DrawDot(3, 5) *)

**ꀢ Explicit END instead of compound statement**

BEGIN can only occur before a statement sequence, but not in it. IF, WHILE, and LOOP are always terminated by END.

**ꀢ WITH statement**

A WITH statement is a regional type guard, it does not imply a hidden variable and does not open a new scope.

See language reference for more details.

**ꀢ ELSIF**

IF statements can have several branches.

Example

    IF name = "top" THEN

        StdLog.Int(0)

    ELSIF name = "bottom" THEN

        StdLog.Int(1)

    ELSIF name = " charm" THEN

        StdLog.Int(2)

    ELSIF name = "beauty" THEN

        StdLog.Int(3)

    ELSE

        StdLog.String("strange")

    END

**ꀢ BY instead of only DOWNTO in FOR**

FOR loops may use any constant value as increment or decrement.

Example

    FOR i := 15 TO 0 BY -1 DO StdLog.Int(i, 0) END

**ꀢ Boolean expressions use short-circuit evaluation**

A Boolean expression terminates as soon as its result can be determined.

Example

    The following expression does not cause a run-time error when p = NIL:

    IF (p # NIL) & (p.name = "quark") THEN

**ꀢ Constant expressions**

In constant declarations, not only literals, but also constant expressions are allowed.

    Example

        CONST

            zero = ORD("0");

            one = zero + 1;

**ꀢ Different operators**

# is used instead of <> for inequality test.

& is used instead of AND for logical conjunctions.

~ is used instead of NOT for logical negation.

**ꀢ Explicit conversion to included type with SHORT**

Type inclusion for numeric types allows to assign values of an included type to an including type. To assign in the other direction, the standard procedure SHORT must be used.

Example

    int := shortint;

    shortint := SHORT(int)

**New Features**

**ꀢ Hexadecimal numbers and characters**

Example

    100H    (* decimal 256 *)

    0DX    (* carriage return *)

**ꀢ Additional numeric types**

LONGINT, SHORTINT, BYTE, SHORTREAL have been added.

**ꀢ Symmetric set difference**

Sets can be subtracted.

**ꀢ New standard procedures**

The new standard procedures INC, DEC, INCL, EXCL, SIZE, ASH, HALT, ASSERT, LEN, LSH, MAX, MIN, BITS, CAP, ENTIER, LONG and SHORT have been added.

**ꀢ LOOP with EXIT**

There is a new loop statement with an explicit exit statement. See language report for more details.

**ꀢ ARRAY OF CHAR can be compared**

Character arrays can be compared with the =, #, <, >, <=, and >= operators.

**ꀢ Open arrays, multidimensional arrays**

Arrays without predefined sizes can be defined, possibly with several dimensions.

Examples

    VAR a: POINTER TO ARRAY OF CHAR;

    NEW(a, 16)

    PROCEDURE ScalarProduct (a, b: ARRAY OF REAL; VAR c: ARRAY OF REAL);

    TYPE Matrix = ARRAY OF ARRAY OF REAL;

    PROCEDURE VectorProduct (a, b: ARRAY OF REAL; VAR c: Matrix);

**ꀢ Pointer dereferencing is optional**

The dereferencing operator ^ can be omitted.

Example

    root.next.value := 5

    instead of

    root^.next^.value := 5

**ꀢ Modules**

Modules are the units of compilation, of information hiding, and of loading. Information hiding is one of the main features of object-oriented programming. Various levels of information hiding are possible: complete hiding, read-only / implement-only export, full export.

See language report for more details.

**ꀢ Type extension**

Record types (pointer types) can be extended, thus providing for polymorphism. Polymorphism is one of the main features of object-oriented programming.

**ꀢ Methods**

Procedures can be bound to record types (pointer types), thus providing late binding. Late binding is one of the main features of object-oriented programming. Such procedures are also called *methods*.

**ꀢ String operator**

The string (sequence of characters) that is contained in an array of character can be selected by using the $-selector.

**ꀢ Record attributes**

Records are non-extensible by default, but may be marked as EXTENSIBLE, ABSTRACT, or LIMITED.

**ꀢ Method attributes**

Methods are non-extensible by default, but may be marked as EXTENSIBLE, ABSTRACT, or EMTPY. Newly introduced methods must be marked as NEW.

