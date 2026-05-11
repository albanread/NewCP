# NewCP — A User's Guide
## Component Pascal for the Modern Workstation

*An introductory manual in the manner of N. Wirth and the
Akademgorodok school. Berichte des Instituts für Computersysteme,
nominal series.*

> **Drei Sprachen — Three Languages — Три языка.**
> The principal text is English. Definitions of the central
> notions are repeated in German (the language in which the
> Oberon family was first described, at ETH Zürich) and in
> Russian (in which the Kronos workstation, the Sokol compiler,
> and a substantial body of Pascal pedagogy were written).
> The repetition is not decorative. A definition that survives
> three translations is more likely to be a definition than a
> turn of phrase.

---

## 0. PROLOGUE — VORWORT — ПРЕДИСЛОВИЕ

**EN.** Component Pascal is a small language. Its definition fits
on roughly thirty pages of EBNF. From that small core arise
modules, separate compilation, garbage collection, extensible
records, dynamic dispatch, runtime type information, and an
integrated documentary medium. The aim of this manual is to make
that core available to the practising programmer without insisting
on the entire formal report.

**DE.** *Component Pascal ist eine kleine Sprache. Ihre Definition
ist in etwa dreißig Seiten erweiterter Backus-Naur-Form gefasst.
Aus diesem schlanken Kern entstehen Module, getrennte Übersetzung,
selbsttätige Speicherverwaltung, erweiterbare Verbunde, dynamische
Methodenbindung, Typinformation zur Laufzeit und eine eingebaute
dokumentarische Schicht. Zweck dieses Heftes ist es, dem Programmierer
den Kern zugänglich zu machen, ohne ihn auf den vollständigen
Sprachbericht zu verpflichten.*

**RU.** *Component Pascal — небольшой язык. Его определение
умещается на тридцати страницах расширенной формы Бэкуса—Наура. На
этом скромном основании построены модули, раздельная компиляция,
автоматическое управление памятью, расширяемые записи, динамическое
связывание методов, информация о типах во время исполнения и
встроенная документная среда. Цель настоящего руководства —
сделать этот фундамент доступным практикующему программисту, не
обязывая его к чтению полного формального отчёта.*

NewCP is the implementation of Component Pascal which the present
manual describes. It compiles to LLVM intermediate representation
and to native code; it loads, links, and executes modules within
one resident process; it provides an integrated graphical surface
(`iGui`); and it preserves the document, view, store, and
controller abstractions of the BlackBox Component Builder.

The manual is divided into twelve sections, an appendix
summarising the standard procedures, and a brief lexicon. The
practitioner is advised to read sections 1, 2, and 3 in order;
the remainder may be consulted as the need arises.

---

## 1. NOTATION — SYMBOLE — ОБОЗНАЧЕНИЯ

We employ the meta-notation of *extended Backus-Naur form*. A
production has the shape

```
    Term  =  Definition .
```

The definition may contain:

```
    A B            sequence: A immediately followed by B
    A | B          alternative: A or B
    [ A ]          optional: zero or one occurrence of A
    { A }          repetition: zero or more occurrences of A
    ( A )          grouping
    "if"           a terminal symbol, written verbatim
```

Identifiers in the meta-language denote non-terminals;
identifiers within quotation marks denote terminals.

> **DE.** *Diese Notation entspricht der erweiterten Backus-Naur-Form,
> wie sie für Pascal, Modula-2 und Oberon eingeführt wurde.*
>
> **RU.** *Употребляется расширенная форма Бэкуса—Наура, принятая в
> работах Н. Вирта по Паскалю, Модуле-2 и Оберону.*

---

## 2. THE VOCABULARY — DAS WORTSCHATZ — СЛОВАРЬ

### 2.1 Identifiers and Reserved Words

An *identifier* is a letter followed by any number of letters or
decimal digits. Identifiers are case-sensitive: `Point` and
`point` are distinct.

```
    ident   =  letter { letter | digit } .
    letter  =  "A" | ... | "Z" | "a" | ... | "z" | "_" .
    digit   =  "0" | ... | "9" .
```

The following are *reserved* and may not be used as identifiers:

```
    ARRAY    BEGIN    BY        CASE     CONST    DIV      DO
    ELSE     ELSIF    END       EXIT     FOR      IF       IMPORT
    IN       IS       LOOP      MOD      MODULE   NIL      OF
    OR       OUT      POINTER   PROCEDURE  RECORD  REPEAT  RETURN
    THEN     TO       TYPE      UNTIL    VAR      WHILE    WITH

    ABSTRACT EMPTY    EXTENSIBLE  LIMITED
```

Predeclared identifiers — basic types, predefined constants,
standard procedures — are not reserved; they may be shadowed by
local declarations, though one is strongly discouraged from doing so.

### 2.2 Number Literals

```
    integer    =  digit { digit }
                | digit { hexdigit } ( "H" | "L" ) .
    real       =  digit { digit } "." { digit } [ ScaleFactor ] .
    ScaleFactor =  ( "E" | "D" ) [ "+" | "-" ] digit { digit } .
    hexdigit   =  digit | "A" | ... | "F" .
```

The suffix `H` denotes a hexadecimal integer literal of type
`INTSHORT` (32 bits, signed). The suffix `L` denotes a hexadecimal
literal of type `LONGINT` (64 bits, signed). An unsuffixed integer
literal has the narrowest type containing it. Thus

```
    7              has type INTEGER
    0FFH           has type INTSHORT, value 255
    0FFFF0000L     has type LONGINT, value 4294901760
    3.14159        has type REAL
    1.5E3          has type REAL, value 1500.0
```

### 2.3 Character and String Literals

A character literal is either a single character between
apostrophes, or a hexadecimal value followed by `X`. Hence

```
    'A'            the character A
    41X            the same: U+0041
    0X             the null character, NUL
```

A string literal is a sequence of characters between double
quotation marks. The empty string is `""`.

```
    "Hello, world"
    "containing a "" double quote"
```

Doubling the quotation mark inside a string yields a literal `"`.

### 2.4 Operators and Delimiters

```
    +    -    *    /    DIV   MOD                   arithmetic
    =    #    <    <=   >     >=                    comparison
    &    OR   ~                                     boolean
    +    -    *    /    IN                          set
    :=                                              assignment
    .    ,    ;    :    ..    |                     punctuation
    (    )    [    ]    {     }                     brackets
    ^    @                                          dereference, address
    ->                                              method qualifier in receiver clause
```

Comments are written `(* … *)` and may be nested.

> **DE.** *Kommentare werden in `(* … *)` eingeschlossen und dürfen
> verschachtelt sein.*
> **RU.** *Комментарии заключаются в `(* … *)` и могут вкладываться
> друг в друга.*

---

## 3. THE BASIC TYPES — DIE GRUNDTYPEN — ОСНОВНЫЕ ТИПЫ

NewCP recognises the following basic types. The widths given are
those of the present implementation on the 64-bit Windows target.

| Type        | Width    | DE              | RU               | Range / domain                  |
|-------------|----------|-----------------|------------------|---------------------------------|
| `BOOLEAN`   |          | Wahrheitswert   | логический       | `FALSE`, `TRUE`                 |
| `BYTE`      |  8 bit   | Byte            | байт             | `0 .. 255`                      |
| `SHORTCHAR` |  8 bit   | Kurzzeichen     | короткий символ  | byte; ASCII / Latin-1           |
| `CHAR`      | 32 bit   | Zeichen         | символ           | one Unicode scalar              |
| `SHORTINT`  | 16 bit   | Kurzganzzahl    | короткое целое   | `-32 768 .. 32 767`             |
| `INTSHORT`  | 32 bit   | Halbganzzahl    | полуцелое        | `-2³¹ .. 2³¹−1`                 |
| `INTEGER`   | 64 bit   | Ganzzahl        | целое            | `-2⁶³ .. 2⁶³−1`                 |
| `LONGINT`   | 64 bit   | Langganzzahl    | длинное целое    | as `INTEGER` (this version)     |
| `SHORTREAL` | 32 bit   | Kurzreelle      | короткое веществ.| IEEE-754 single                 |
| `REAL`      | 64 bit   | Reelle          | вещественное     | IEEE-754 double                 |
| `SET`       | 32 bit   | Menge           | множество        | `{0 .. 31}`                     |

`INTSHORT` is an implementation extension of standard Component
Pascal, introduced because NewCP makes `INTEGER` 64 bits wide and
some interfaces still require a 32-bit signed slot. The standard
hierarchy `SHORTINT ⊂ INTEGER ⊂ LONGINT` is preserved.

The function `MIN(T)` yields the smallest value of type `T`; the
function `MAX(T)` yields the largest:

```
    MAX(SHORTINT) = 32767
    MIN(INTEGER)  = -9223372036854775808
    MAX(SET)      = 31
```

---

## 4. EXPRESSIONS — AUSDRÜCKE — ВЫРАЖЕНИЯ

### 4.1 The Operator Hierarchy

Operators are listed in *decreasing* order of binding strength.
Operators on the same line have the same precedence and group from
left to right.

```
    Level 4 (unary):    +  -  ~                          (DE: Vorzeichen, Negation)
    Level 3 (factor):   *  /  DIV  MOD  &
    Level 2 (term):     +  -  OR
    Level 1 (relation): =  #  <  <=  >  >=  IN  IS
```

The expression `a + b * c` therefore reads as `a + (b * c)`. The
boolean conjunction is the ampersand `&`; the boolean disjunction
is the word `OR`. The relation `IN` tests set membership; the
relation `IS` tests dynamic type.

> **DE.** *Die boolsche Konjunktion ist das kaufmännische Und `&`,
> die Disjunktion das Wort `OR`, die Negation die Tilde `~`.*
> **RU.** *Логическое И обозначается `&`, ИЛИ — словом `OR`,
> отрицание — тильдой `~`.*

### 4.2 Integer and Real Arithmetic

The operators `+ - *` apply to both integer and real arguments.
There are two division operators:

```
    /        real division, always yields REAL or SHORTREAL
    DIV      integer division, FLOOR(a/b);     0 ≤ (a MOD b) < b  if b > 0
    MOD      integer remainder, paired with DIV by the floor rule
```

Note the floor rule. `(-5) DIV 3 = -2`, not `-1`; correspondingly
`(-5) MOD 3 = 1`, satisfying `0 ≤ r < 3`. The unary minus binds
less tightly than `DIV`, so `-5 DIV 3` means `-(5 DIV 3) = -1`. To
avoid this trap, parenthesise: `(-5) DIV 3`.

### 4.3 Set Construction

```
    SetCtor   =  "{" [ Element { "," Element } ] "}" .
    Element   =  Expr [ ".." Expr ] .
```

Examples:

```
    {}                   the empty set
    {3, 5, 7}            a three-element set
    {3 .. 7}             every integer from 3 to 7 inclusive
    {0, 2 .. 4, 6}       mixed singleton and range
```

The set operators are

```
    +    union           DE: Vereinigung           RU: объединение
    -    difference      DE: Differenz             RU: разность
    *    intersection    DE: Durchschnitt          RU: пересечение
    /    symmetric diff  DE: symm. Differenz       RU: симметрическая разность
    -s   complement      DE: Komplement            RU: дополнение
```

Set elements are integers in `0 .. MAX(SET)`; for the present
implementation, `MAX(SET) = 31`.

---

## 5. STATEMENTS — ANWEISUNGEN — ОПЕРАТОРЫ

### 5.1 Assignment and Procedure Call

```
    StatementSeq  =  Statement { ";" Statement } .
    Statement     =  [ Assignment | ProcedureCall | IfStmt | CaseStmt
                     | WhileStmt | RepeatStmt | ForStmt | LoopStmt
                     | WithStmt | ExitStmt | ReturnStmt ] .
    Assignment    =  Designator ":=" Expr .
```

Assignment is `:=`. The single equals sign is comparison only.

### 5.2 IF and CASE

```
    IfStmt     =  "IF" Expr "THEN" StatementSeq
                  { "ELSIF" Expr "THEN" StatementSeq }
                  [ "ELSE" StatementSeq ]
                  "END" .

    CaseStmt   =  "CASE" Expr "OF" Case { "|" Case }
                  [ "ELSE" StatementSeq ] "END" .
    Case       =  [ CaseLabels { "," CaseLabels } ":" StatementSeq ] .
    CaseLabels =  ConstExpr [ ".." ConstExpr ] .
```

The selector of a `CASE` may be of integer, character, or other
ordinal type; the labels are constants or constant ranges. Labels
must not overlap.

```
    CASE n OF
      1, 3, 5: r := 1
    | 2, 4, 6: r := 2
    ELSE      r := 0
    END
```

### 5.3 Iterative Statements

```
    WhileStmt   =  "WHILE" Expr "DO" StatementSeq "END" .
    RepeatStmt  =  "REPEAT" StatementSeq "UNTIL" Expr .
    ForStmt     =  "FOR" ident ":=" Expr "TO" Expr [ "BY" ConstExpr ]
                   "DO" StatementSeq "END" .
    LoopStmt    =  "LOOP" StatementSeq "END" .
    ExitStmt    =  "EXIT" .
```

Each iterative form has its place. `WHILE` tests at the top;
`REPEAT` tests at the bottom; `FOR` runs a counter from start to
end inclusive, with an optional step `BY` (which may be negative);
`LOOP` repeats indefinitely and is left by `EXIT`. `EXIT` leaves
the *innermost* enclosing `LOOP`.

> **DE.** *`EXIT` verlässt stets nur die innerste umschließende
> `LOOP`-Anweisung. Wer eine äußere Schleife verlassen will,
> verwende eine Hilfsvariable.*
>
> **RU.** *Оператор `EXIT` покидает только ближайший охватывающий
> `LOOP`. Для выхода из внешнего цикла используйте вспомогательную
> переменную.*

### 5.4 WITH and Type Guards

The `WITH` statement narrows the static type of a variable inside
a branch:

```
    WITH a: Bird DO
        IF a.canFly THEN ... END
    | a: Fish DO
        result := 20 + a.fins
    ELSE
        result := a.legs
    END
```

The variable `a` has the narrowed type inside each arm. See §8.4.

### 5.5 RETURN

```
    ReturnStmt  =  "RETURN" [ Expr ] .
```

A `RETURN` with an expression yields a value from a function
procedure. A `RETURN` without an expression terminates a proper
(non-value-returning) procedure. The form `RETURN expr` is also
the only legitimate way to write the result of a function: the
classical Pascal idiom of assigning to the function name is *not*
supported.

---

## 6. DECLARATIONS — DEKLARATIONEN — ОПИСАНИЯ

### 6.1 Constants, Variables, and Types

```
    Declaration   =  ConstSection | TypeSection | VarSection | ProcDecl .
    ConstSection  =  "CONST" { IdentDef "=" ConstExpr ";" } .
    TypeSection   =  "TYPE"  { IdentDef "=" Type ";" } .
    VarSection    =  "VAR"   { IdentList ":" Type ";" } .
    IdentList     =  IdentDef { "," IdentDef } .
    IdentDef      =  ident [ "*" | "-" ] .
```

The optional mark `*` after a declared identifier *exports* it.
The mark `-` exports it as *read-only* (for variables) or
*implement-only* (for record types). An unmarked identifier is
*private to the module*.

> **DE.** *Der Stern `*` bedeutet Export, der Strich `-` bedeutet
> nur-lesbarer beziehungsweise nur-implementierender Export. Ein
> unmarkierter Bezeichner bleibt im Modul verborgen.*
>
> **RU.** *Звёздочка `*` означает экспорт, минус `-` — экспорт
> «только чтение» либо «только реализация». Идентификатор без
> метки остаётся скрытым внутри модуля.*

### 6.2 Structured Types

```
    Type        =  TypeRef | ArrayType | RecordType | PointerType | ProcType .
    ArrayType   =  "ARRAY" [ Length { "," Length } ] "OF" Type .
    RecordType  =  [ "ABSTRACT" | "EXTENSIBLE" | "LIMITED" ]
                   "RECORD" [ "(" TypeRef ")" ] [ FieldList ]  "END" .
    FieldList   =  Field { ";" Field } .
    Field       =  IdentList ":" Type .
    PointerType =  "POINTER" "TO" TypeRef .
    ProcType    =  "PROCEDURE" [ FormalParams ] .
```

A *fixed array* declares its length(s); the syntax `ARRAY n, m OF T`
abbreviates `ARRAY n OF ARRAY m OF T`. An *open array* (length
unspecified) is admissible only as a formal parameter type:

```
    PROCEDURE Sum(IN a: ARRAY OF INTEGER): INTEGER;
```

A *record* may inherit from another by naming it parenthetically.
A pointer to an extending record is assignable to a pointer to the
base record.

### 6.3 Pointer-Alias Idiom

By convention, every record meant to be used by reference is paired
with a pointer alias whose name omits the `Desc` suffix:

```
    TYPE
        PointDesc* = RECORD
            x*, y*: INTEGER
        END;
        Point*     = POINTER TO PointDesc;
```

Users work with `Point`. The system auto-dereferences a pointer in
a field selection, so one writes `p.x`, never `p^.x`.

> **DE.** *Die `Desc`-Konvention ist nicht eine Regel der Sprache,
> sondern ein Stil. Die gesamte BlackBox-Bibliothek folgt ihm; neue
> Module sollen es ebenso tun.*
>
> **RU.** *Соглашение «Desc» — не правило языка, а стиль; вся
> библиотека BlackBox следует ему, и новым модулям рекомендуется
> поступать так же.*

---

## 7. PROCEDURES — PROZEDUREN — ПРОЦЕДУРЫ

### 7.1 Plain Procedures

```
    ProcDecl     =  "PROCEDURE" [ Receiver ] IdentDef [ FormalParams ]
                    [ "," Attribute { "," Attribute } ] ";"
                    DeclSeq [ "BEGIN" StatementSeq ]
                    [ "RETURN" Expr ] "END" ident .

    FormalParams =  "(" [ FPSection { ";" FPSection } ] ")" [ ":" TypeRef ] .
    FPSection    =  [ "VAR" | "IN" | "OUT" ] IdentList ":" Type .
    Attribute    =  "NEW" | "ABSTRACT" | "EMPTY" | "EXTENSIBLE" .
```

A *proper procedure* has no result type and is invoked as a
statement. A *function procedure* declares its result type after
the closing parenthesis and returns a value via `RETURN expr`.

### 7.2 Parameter Modes

Four modes:

| Mode      | DE          | RU            | Semantics                                 |
|-----------|-------------|---------------|-------------------------------------------|
| (value)   | Wert        | значение      | callee receives a private copy            |
| `VAR`     | Verweis     | по ссылке     | callee receives a pointer; writes visible |
| `IN`      | Eingang     | входной       | reference; read-only inside the callee    |
| `OUT`     | Ausgang     | выходной      | reference; must be written before read    |

For records and arrays passed by value the runtime cost is large;
the compiler rejects them outright. Use `VAR` for read–write, `IN`
for read-only.

### 7.3 Methods — Bound Procedures

A *method* is a procedure with a *receiver* clause preceding the
name:

```
    Receiver  =  "(" [ "VAR" | "IN" ] ident ":" TypeRef ")" .
```

So:

```
    PROCEDURE (p: PointDesc) Norm*(): REAL, NEW;
    BEGIN
        RETURN Math.Sqrt(p.x * p.x + p.y * p.y)
    END Norm;
```

This binds `Norm` to the record `PointDesc`. Inside the body, `p`
plays the rôle of `self` / `this`. The attribute `NEW` declares
that this is a fresh slot in the vtable; omitting `NEW` declares
that this *overrides* a same-named method on the base record.

The attribute `EXTENSIBLE` marks a method as overridable; without
it, subtypes may not redefine it. `ABSTRACT` declares a method
without a body, to be supplied by every concrete subtype. The
following table summarises:

| Attributes        | DE                | RU                |
|-------------------|-------------------|-------------------|
| `NEW`             | Neue Methode      | новый метод       |
| `NEW, EXTENSIBLE` | Neu und erweiterb.| новый и расширяем.|
| `NEW, ABSTRACT`   | Neu und abstrakt  | новый и абстракт. |
| (no attribute)    | Überschreibung    | переопределение   |
| `EMPTY`           | Leerimplementierung | пустая реализация |

### 7.4 Allocation and `NEW`

```
    NEW(p)
```

allocates a record of the type pointed to by `p`, writes a hidden
type tag into the block header, zero-initialises the body, and
stores the resulting pointer in `p`. There is no `DISPOSE`. The
garbage collector reclaims unreachable blocks at unspecified times.

> **DE.** *Das Schlüsselwort `DISPOSE` gibt es nicht. Der
> Müllsammler entscheidet, wann ein nicht mehr erreichbarer Block
> freigegeben wird.*
>
> **RU.** *Оператора `DISPOSE` нет. Решение о моменте освобождения
> блока, на который не осталось ссылок, принимает сборщик мусора.*

---

## 8. THE OBJECT MODEL — DAS OBJEKTMODELL — ОБЪЕКТНАЯ МОДЕЛЬ

### 8.1 Inheritance

A record extends another by naming the parent in parentheses
immediately after the keyword `RECORD`:

```
    TYPE
        AnimalDesc* = ABSTRACT RECORD
            legs*: INTEGER
        END;
        Animal*     = POINTER TO AnimalDesc;

        DogDesc*    = RECORD (AnimalDesc) END;
        Dog*        = POINTER TO DogDesc;
```

`DogDesc` inherits the field `legs`. A `Dog` is assignable to an
`Animal`; the converse is not allowed without an explicit type
test.

### 8.2 Dynamic Dispatch

When a method is called through a base-typed pointer, the runtime
consults the block's type tag and dispatches to the method body
defined by the *dynamic* type:

```
    PROCEDURE (a: AnimalDesc) Sound*(): INTEGER, NEW, ABSTRACT;

    PROCEDURE (d: DogDesc) Sound*(): INTEGER;
    BEGIN RETURN 1 END Sound;

    PROCEDURE Greet*(a: Animal): INTEGER;
    BEGIN RETURN a.Sound() END Greet;             (* dispatches dynamically *)
```

### 8.3 Type Testing — `IS`

```
    expr  IS  T          yields TRUE iff the dynamic type of expr extends T
```

`IS` works for both record-typed and pointer-typed expressions.

### 8.4 Type Guards — `WITH`

To use a variable as its narrower dynamic type *within a block*,
the `WITH` statement applies. Inside each arm the variable is
treated as the named type:

```
    WITH a: DogDesc DO
        ... a is treated as DogDesc here ...
    | a: BirdDesc DO
        ... a is treated as BirdDesc here ...
    ELSE
        ... fallback ...
    END
```

A type guard with no `ELSE` raises a runtime trap if no arm
matches.

### 8.5 ABSTRACT and EMPTY

A record marked `ABSTRACT` cannot be instantiated with `NEW`; it
exists only to be extended. A method declared `NEW, ABSTRACT` has
no body; every concrete subtype must override it.

A method declared `EMPTY` has no body but is not abstract: a
concrete subtype need not override it. The default implementation
is the no-op (for a proper procedure) or zero (for a function).

> **DE.** *`EMPTY` ist nützlich für Rückrufmethoden, die der Nutzer
> nur dann implementiert, wenn ihn das Ereignis interessiert.*
>
> **RU.** *`EMPTY` удобна для методов обратного вызова, которые
> пользователь определяет, лишь когда событие ему важно.*

---

## 9. MODULES — MODULE — МОДУЛИ

### 9.1 The Compilation Unit

The compilation unit is the *module*. Its skeleton is

```
    Module  =  "MODULE" ident ";"
               [ ImportList ]
               DeclSeq
               [ "BEGIN" StatementSeq ]
               [ "CLOSE" StatementSeq ]
               "END" ident "." .
    ImportList  =  "IMPORT" Import { "," Import } ";" .
    Import      =  ident [ ":=" ident ] .
```

The module body, between `BEGIN` and `END`, runs *once*, when the
module is first loaded. The `CLOSE` section, if present, runs when
the module is unloaded.

```
    MODULE Counter;
    VAR n*: INTEGER;
    PROCEDURE Bump*; BEGIN INC(n) END Bump;
    BEGIN
        n := 0
    END Counter.
```

### 9.2 Imports

To use another module's facilities, name it in the `IMPORT` list.
Renaming is permitted:

```
    IMPORT
        Console,
        Str := Strings,
        F   := HostFiles;
```

Outside its own module, every exported identifier is referred to
by *qualified* name: `Strings.Length(s)`, never bare `Length(s)`.
This makes the source of every symbol immediate.

### 9.3 Separate Compilation, Dynamic Loading

NewCP compiles each module to LLVM intermediate code, then JITs
it on demand into the running process. Modules are loaded in the
order their dependencies require; mutual import is forbidden. The
loader maintains a *symbol file* per module, giving the compiler
enough type information to compile clients without recompiling
servers.

> **DE.** *Module werden einzeln übersetzt. Beim Aufruf eines
> noch nicht geladenen Moduls lädt das Laufzeitsystem es selbsttätig,
> initialisiert es und gibt erst dann die Kontrolle zurück.*
>
> **RU.** *Модули компилируются по отдельности. При обращении к ещё
> не загруженному модулю система времени исполнения сама загружает
> его, выполняет инициализацию и лишь затем возвращает управление.*

### 9.4 A Complete Example

Two co-operating modules:

```
    MODULE Geometry;
    TYPE
        PointDesc* = RECORD x*, y*: INTEGER END;
        Point*     = POINTER TO PointDesc;
    PROCEDURE NewPoint*(x, y: INTEGER): Point;
        VAR p: Point;
    BEGIN
        NEW(p); p.x := x; p.y := y; RETURN p
    END NewPoint;
    PROCEDURE (p: PointDesc) Norm*(): REAL, NEW;
    BEGIN
        RETURN Math.Sqrt(p.x * p.x + p.y * p.y)
    END Norm;
    END Geometry.

    MODULE GeometryTest;
    IMPORT Console, Geometry, Math;
    PROCEDURE Run*;
        VAR p: Geometry.Point;
    BEGIN
        p := Geometry.NewPoint(3, 4);
        Console.WriteShortString("|p| = ");
        Console.WriteInt(ENTIER(p.Norm()));
        Console.WriteLn
    END Run;
    END GeometryTest.
```

---

## 10. PROGRAMME DEVELOPMENT — DAS PROGRAMMIEREN — РАЗРАБОТКА ПРОГРАММ

### 10.1 The Driver

The command-line interface to the system is `newcp-driver`. Its
principal subcommands are

```
    newcp-driver dump-tokens <Module>            lexical analysis
    newcp-driver dump-ast <Module>               abstract-syntax tree
    newcp-driver dump-sema <Module>              type-bound tree
    newcp-driver dump-module-graph               import digraph
    newcp-driver dump-cfg <Module>               control-flow graph
    newcp-driver dump-ir <Module>                typed intermediate form
    newcp-driver dump-llvm <Module>              LLVM intermediate code
    newcp-driver dump-asm <Module>               native assembly
    newcp-driver dump-heap                       garbage-collector snapshot

    newcp-driver describe-interface <Module>     exported symbols + types
    newcp-driver load-module <Module> [Cmd]      JIT-load; optionally run Cmd
    newcp-driver run-igui <Module.Procedure>     run a procedure under iGui
```

Each *dump-* subcommand emits a textual artifact, identical in
shape across recompilations of the same source. The system is
explicitly *not* an opaque pipeline; every phase is reviewable.

> **DE.** *Es ist eine bewusste Entwurfsentscheidung, dass jede
> Phase der Übersetzung in Textform betrachtet werden kann. Die
> Übersicht des Werkzeugs hat denselben Rang wie seine Korrektheit.*
>
> **RU.** *Сделан намеренный выбор: каждая фаза трансляции доступна
> в текстовой форме. Прозрачность инструмента — столь же важное
> качество, как и его правильность.*

### 10.2 Suggested Workflow

1. Write the module in any text editor; save with extension `.cp`.
2. Run `newcp-driver dump-sema MyModule` to obtain a type-bound
   listing. Errors of declaration or type appear here.
3. Run `newcp-driver dump-ir MyModule` to inspect the intermediate
   form. Subtle bugs of evaluation order or type coercion are
   visible here.
4. Run `newcp-driver load-module MyModule MyModule.Run` to load
   the module into the live process and invoke its `Run` command.
5. The integrated `iGui` provides a graphical surface and a
   built-in editor `redit` for emergency repair.

---

## 11. THE STANDARD LIBRARY — DIE STANDARDBIBLIOTHEK — СТАНДАРТНАЯ БИБЛИОТЕКА

A representative selection of resident modules. Each is fully
described in the corresponding interface listing
(`newcp-driver describe-interface <Module>`); the present list is
merely orientation.

| Module        | DE                       | RU                          | Purpose                                              |
|---------------|--------------------------|-----------------------------|------------------------------------------------------|
| `Kernel`      | Laufzeitkern             | ядро времени исполнения     | type descriptors, allocation, finalizers, traps     |
| `Console`     | Konsole                  | консоль                     | text input/output to the host terminal              |
| `Log`         | Protokoll                | протокол                    | the resident log view; ring-buffered diagnostic     |
| `Math`        | Mathematik               | математика                  | `Sqrt`, `Sin`, `Cos`, `Exp`, `Ln`, etc.             |
| `SMath`       | Kurzmathematik           | короткая математика         | the same for `SHORTREAL`                            |
| `Integers`    | Ganzzahlen               | целые числа                 | arbitrary-width integer routines                    |
| `Strings`     | Zeichenketten            | строки                      | `Length`, `Find`, `Copy`, `IntToString`, `Replace`  |
| `Files`       | Dateien                  | файлы                       | abstract file interface                             |
| `HostFiles`   | Hostdateien              | файлы операционной системы  | concrete implementation of `Files`                  |
| `Dates`       | Zeit / Datum             | дата и время                | abstract calendar/clock interface                   |
| `HostDates`   | Hostzeit                 | системная дата и время      | concrete implementation                             |
| `Fonts`       | Schriftarten             | шрифты                      | abstract font enumeration and metrics               |
| `HostFonts`   | Hostschriftarten         | системные шрифты            | DirectWrite-backed implementation                   |
| `Stores`      | Speicher                 | хранилища                   | persistent typed serialisation                      |
| `Models`      | Modelle                  | модели                      | document-data abstraction                           |
| `Views`       | Sichten                  | представления               | document-view abstraction                           |
| `Controllers` | Steuerungen              | контроллеры                 | input dispatching                                   |
| `iGui`        | Eingebaute Oberfläche    | встроенный графический интерфейс | MDI windows, drawing, events, menus           |

The pattern `Xxx` / `HostXxx` is the BlackBox layered design: the
abstract module declares the interface (no Win32, no platform
dependency); `HostXxx` provides one concrete implementation, the
only module that imports `iGui` and the system-specific bits.

### 11.1 Console Output

The minimum:

```
    IMPORT Console;

    PROCEDURE Run*;
    BEGIN
        Console.WriteShortString("Hello, world.");
        Console.WriteLn
    END Run;
```

Useful Console procedures:

```
    WriteShortString(s)          a SHORTCHAR string
    WriteString(s)               a CHAR string (UTF-32)
    WriteInt(n)                  a signed integer, decimal
    WriteReal(x)                 a real number
    WriteHex(n, digits)          hexadecimal, padded
    WriteLn                      a line break
```

### 11.2 iGui — The Integrated Graphical Environment

The integrated GUI replaces the external `multiwingui` / `wingui`
host of classical BlackBox with an MDI host built directly into
the runtime. It is the *main thread*; the language runtime is
spawned underneath it.

Opening a child window, painting it, draining events:

```
    MODULE Phase2EventDemo;
    IMPORT iGui, Console;

    PROCEDURE Run*;
      VAR kind, childId, t, p1, p2, p3, p4: INTEGER;
          ok: INTSHORT;
    BEGIN
        REPEAT
            ok := iGui.NextEvent(kind, childId, t, p1, p2, p3, p4, -1);
            IF ok # 0 THEN
                IF kind = iGui.EvKey THEN
                    Console.WriteShortString("[key] ");
                    Console.WriteInt(p1); Console.WriteLn
                ELSIF kind = iGui.EvMouse THEN
                    Console.WriteShortString("[mouse] ");
                    Console.WriteInt(p1); Console.WriteShortString(",");
                    Console.WriteInt(p2); Console.WriteLn
                ELSIF kind = iGui.EvFrameClose THEN
                    EXIT
                END
            END
        UNTIL FALSE
    END Run;

    END Phase2EventDemo.
```

The full surface — drawing primitives, text layout, menus, timers,
DPI awareness — is given in the iGui phase demos under
`Mod/demo/igui/`.

---

## 12. CONVENTIONS, ATTRIBUTES, AND CARDINAL RULES

We close with the discipline that experience has shown to produce
maintainable Component Pascal programmes.

### 12.1 Naming

| Domain                       | DE                  | RU                  | Style                                |
|------------------------------|---------------------|---------------------|--------------------------------------|
| Module name                  | Modulname           | имя модуля          | `Strings`, `HostFonts`               |
| Type name (record)           | Verbundtyp          | тип-запись          | `WindowDesc`                         |
| Type name (pointer alias)    | Zeigertyp           | тип-указатель       | `Window`                             |
| Procedure / method           | Prozedur            | процедура           | `WriteInt`, `Norm`, `OpenChild`      |
| Field / variable             | Feld, Variable      | поле, переменная    | `lineHeight`, `nextEvent`            |
| Constant                     | Konstante           | константа           | `maxLines`, `EvFrameClose`           |
| Receiver parameter           | Empfangsparameter   | параметр-приёмник   | `self`, `s`, `p`, `this`             |

### 12.2 Cardinal Rules

These rules are not enforced by the compiler, but the standard
library follows them and clients are advised to do likewise.

1. **One module per file**, named the same as the module.
2. **Exports marked with `*`; nothing else exported.** A type
   exported with `*` exports its memory layout. A field marked
   `-` is read-only outside the module.
3. **Records by reference, not by value.** Declare `XxxDesc` and
   `Xxx = POINTER TO XxxDesc`; clients use `Xxx`.
4. **Abstract base, concrete subtype.** When a subsystem has more
   than one implementation, the interface is an `ABSTRACT RECORD`
   in module `Xxx`; implementations live in `HostXxx` (or
   `MyXxx`, `RemoteXxx`, …).
5. **No platform-specific imports in abstract modules.** Only
   `HostXxxSys` is allowed to import `iGui` or other
   system-specific facilities.
6. **`IN` for read-only parameters; `VAR` for read–write.**
   Records and arrays by value are rejected by the compiler.

> **DE.** *Diese Regeln sind kein Diktat der Sprache, sondern eine
> Sammlung bewährter Übereinkünfte. Wer sie hält, schreibt im Geiste
> der BlackBox; wer sie verletzt, mag gute Gründe haben.*
>
> **RU.** *Это не предписания языка, а свод устоявшихся соглашений.
> Кто их соблюдает, пишет в духе BlackBox; кто их нарушает, пусть
> имеет веские основания.*

---

## APPENDIX A — STANDARD PROCEDURES

A predeclared procedure may be applied as if it were declared in
the surrounding scope, but the compiler treats it specially. The
following appear in every NewCP scope.

### A.1 Numeric and Logical

| Procedure              | Effect                                                                 |
|------------------------|------------------------------------------------------------------------|
| `ABS(x)`               | absolute value                                                         |
| `ODD(n)`               | `TRUE` iff `n` is odd                                                  |
| `MIN(T)` / `MAX(T)`    | the type's minimum / maximum value                                     |
| `MIN(a, b)` / `MAX(a, b)` | the lesser / greater of two values                                  |
| `INC(v)` / `INC(v, n)` | increment a variable by 1 or by `n`                                    |
| `DEC(v)` / `DEC(v, n)` | decrement                                                              |
| `ASH(x, n)`            | arithmetic shift; `n > 0` left, `n < 0` right                          |
| `CAP(ch)`              | uppercase the character `ch`                                           |
| `ORD(x)`               | ordinal value: integer code of a character; bit-pattern of a set       |
| `CHR(n)`               | character of ordinal `n`                                               |
| `ENTIER(x)`            | the largest `INTEGER` ≤ the real `x` (floor)                           |
| `SHORT(x)` / `LONG(x)` | narrow / widen a numeric value                                         |
| `BITS(n)`              | the `SET` whose elements are the 1-bits of integer `n`                 |

### A.2 Sets

| Procedure         | Effect                                                                 |
|-------------------|------------------------------------------------------------------------|
| `INCL(s, n)`      | adjoin `n` to set `s`                                                  |
| `EXCL(s, n)`      | remove `n` from set `s`                                                |

### A.3 Allocation and Reflection

| Procedure         | Effect                                                                 |
|-------------------|------------------------------------------------------------------------|
| `NEW(p)`          | allocate the record `p` points at; install hidden type tag             |
| `NEW(p, n)`       | allocate an array pointer's open dimension(s)                          |
| `LEN(a)`          | length of the first dimension of array `a`                             |
| `LEN(a, i)`       | length of the `i`-th dimension                                         |
| `SIZE(T)`         | number of bytes occupied by a value of type `T`                        |

### A.4 SYSTEM (low-level)

The pseudo-module `SYSTEM` is imported explicitly:

```
    IMPORT SYSTEM;
```

Selected facilities:

```
    SYSTEM.ADR(x)                       address of x as INTEGER
    SYSTEM.VAL(T, x)                    bit-pattern reinterpretation
    SYSTEM.GET(a, v)                    raw load
    SYSTEM.PUT(a, v)                    raw store
    SYSTEM.MOVE(src, dst, n)            memory copy
    SYSTEM.LSH(x, n)                    logical shift
    SYSTEM.ROT(x, n)                    bit rotation
```

Use of `SYSTEM` is unsafe and is the boundary between portable
Component Pascal and the underlying machine. As such it is the
mechanism by which the resident runtime is constructed.

---

## APPENDIX B — A COMPLETE EXAMPLE

A queue, polymorphic in its element type by means of the universal
top type `ANYPTR`:

```
    MODULE Queues;

    TYPE
        Cell    = POINTER TO CellDesc;
        CellDesc = RECORD
            data: ANYPTR;
            next: Cell
        END;

        QueueDesc* = RECORD
            head, tail: Cell
        END;
        Queue*     = POINTER TO QueueDesc;

    PROCEDURE NewQueue*(): Queue;
        VAR q: Queue;
    BEGIN NEW(q); q.head := NIL; q.tail := NIL; RETURN q END NewQueue;

    PROCEDURE (q: QueueDesc) IsEmpty*(): BOOLEAN, NEW;
    BEGIN RETURN q.head = NIL END IsEmpty;

    PROCEDURE (q: QueueDesc) Push*(x: ANYPTR), NEW;
        VAR c: Cell;
    BEGIN
        NEW(c); c.data := x; c.next := NIL;
        IF q.tail = NIL THEN
            q.head := c; q.tail := c
        ELSE
            q.tail.next := c; q.tail := c
        END
    END Push;

    PROCEDURE (q: QueueDesc) Pop*(): ANYPTR, NEW;
        VAR x: ANYPTR;
    BEGIN
        IF q.head = NIL THEN RETURN NIL END;
        x := q.head.data;
        q.head := q.head.next;
        IF q.head = NIL THEN q.tail := NIL END;
        RETURN x
    END Pop;

    END Queues.
```

A client:

```
    MODULE QueueDemo;
    IMPORT Console, Queues;

    TYPE
        IntBox = POINTER TO IntBoxDesc;
        IntBoxDesc = RECORD value: INTEGER END;

    PROCEDURE NewBox(v: INTEGER): IntBox;
        VAR b: IntBox;
    BEGIN NEW(b); b.value := v; RETURN b END NewBox;

    PROCEDURE Run*;
        VAR q: Queues.Queue;
            x: ANYPTR;
    BEGIN
        q := Queues.NewQueue();
        q.Push(NewBox(1));
        q.Push(NewBox(2));
        q.Push(NewBox(3));
        WHILE ~q.IsEmpty() DO
            x := q.Pop();
            WITH x: IntBox DO
                Console.WriteInt(x.value);
                Console.WriteShortString(" ")
            END
        END;
        Console.WriteLn
    END Run;

    END QueueDemo.
```

`Run` prints `1 2 3 ` and a line break.

---

## APPENDIX C — A SHORT LEXICON

For the convenience of the trilingual reader.

| English                | Deutsch                  | Русский                       |
|------------------------|--------------------------|-------------------------------|
| module                 | Modul                    | модуль                        |
| procedure              | Prozedur                 | процедура                     |
| function (proc.)       | Funktionsprozedur        | функция-процедура             |
| record                 | Verbund                  | запись                        |
| field                  | Feld                     | поле                          |
| pointer                | Zeiger                   | указатель                     |
| array                  | Reihung                  | массив                        |
| set                    | Menge                    | множество                     |
| string                 | Zeichenkette             | строка                        |
| character              | Zeichen                  | символ                        |
| integer (64-bit)       | Ganzzahl                 | целое число                   |
| real number            | reelle Zahl              | вещественное число            |
| variable               | Variable                 | переменная                    |
| constant               | Konstante                | константа                     |
| type                   | Typ                      | тип                           |
| reference              | Verweis                  | ссылка                        |
| dereference            | Auflösung                | разыменование                 |
| assignment             | Zuweisung                | присваивание                  |
| statement              | Anweisung                | оператор                      |
| expression             | Ausdruck                 | выражение                     |
| iteration / loop       | Schleife                 | цикл                          |
| condition              | Bedingung                | условие                       |
| guard (type)           | Typprüfung               | проверка типа                 |
| import                 | Import / Einbindung      | импорт                        |
| export mark `*`        | Exportzeichen            | знак экспорта                 |
| garbage collector      | Müllsammler              | сборщик мусора                |
| heap                   | Halde                    | куча                          |
| stack                  | Stapel                   | стек                          |
| compilation unit       | Übersetzungseinheit      | единица компиляции            |
| compiler driver        | Übersetzersteuerung      | управляющая программа         |
| just-in-time compiler  | Bedarfsübersetzer (JIT)  | JIT-компилятор                |

---

## EPILOGUE

> **EN.** Component Pascal is a small language designed for
> serious work. It rewards discipline. It punishes ornament. Used
> well, it produces code that reads like prose and reasons like
> mathematics.
>
> **DE.** *Component Pascal ist eine kleine Sprache für ernsthafte
> Arbeit. Sie belohnt Disziplin und bestraft Schmuck. Recht
> verwendet, erzeugt sie Programme, die sich wie Prosa lesen und
> wie Mathematik beweisen lassen.*
>
> **RU.** *Component Pascal — небольшой язык для серьёзной работы.
> Он вознаграждает дисциплину и наказывает украшательство. При
> должном употреблении он порождает программы, которые читаются
> как проза и доказываются как математика.*

*— The Authors, in the spirit of N. Wirth and A. P. Ershov.*
