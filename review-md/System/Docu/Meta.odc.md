**Meta**

DEFINITION Meta;

    CONST

        undef = 0;

        typObj = 2; varObj = 3; procObj = 4; fieldObj = 5; modObj = 6; parObj = 7;

        boolTyp = 1; sCharTyp = 2; charTyp = 3;

        byteTyp = 4; sIntTyp = 5; intTyp = 6; longTyp = 10; sRealTyp = 7; realTyp = 8;

        setTyp = 9; anyRecTyp = 11; anyPtrTyp = 12;

        procTyp = 16; recTyp = 17; arrTyp = 18; ptrTyp = 19;

        final = 0; extensible = 1; limited = 2; abstract = 3;

        hidden = 1; readOnly = 2; exported = 4;

        value = 10; in = 11; out = 12; var = 13;

    TYPE

        Name = ARRAY 256 OF CHAR;

        Value = ABSTRACT RECORD END;

        Item = RECORD (Value)

            obj-, typ-, vis-, adr-: INTEGER;

            (VAR i: Item) Valid (): BOOLEAN, NEW;

            (VAR i: Item) GetTypeName (OUT mod, type: Name), NEW;

            (VAR i: Item) BaseTyp (): INTEGER, NEW;

            (VAR i: Item) Level (): INTEGER, NEW;

            (VAR i: Item) Size (): INTEGER, NEW;

            (VAR arr: Item) Len (): INTEGER, NEW;

            (VAR in: Item) Lookup (IN name: ARRAY OF CHAR; VAR i: Item), NEW;

            (VAR i: Item) GetBaseType (VAR base: Item), NEW;

            (VAR rec: Item) GetThisBaseType (level: INTEGER; VAR base: Item), NEW;

            (VAR proc: Item) NumParam (): INTEGER, NEW;

            (VAR proc: Item) GetParam (n: INTEGER; VAR par: Item), NEW;

            (VAR proc: Item) GetParamName (n: INTEGER; OUT name: Name), NEW;

            (VAR proc: Item) GetReturnType (VAR type: Item), NEW;

            (VAR rec: Item) Is (IN type: Value): BOOLEAN, NEW;

            (VAR ptr: Item) Deref (VAR ref: Item), NEW;

            (VAR arr: Item) Index (index: INTEGER; VAR elem: Item), NEW;

            (VAR proc: Item) Call (OUT ok: BOOLEAN), NEW;

            (VAR proc: Item) ParamCall (IN par: ARRAY OF Item; VAR dest: Item;

                                                        OUT ok: BOOLEAN), NEW;

            (VAR proc: Item) ParamCallVal (IN par: ARRAY OF POINTER TO Value; VAR dest: Item;

                                                        OUT ok: BOOLEAN), NEW;

            (VAR var: Item) GetVal (VAR x: Value; OUT ok: BOOLEAN), NEW;

            (VAR var: Item) PutVal (IN x: Value; OUT ok: BOOLEAN), NEW;

            (VAR var: Item) GetStringVal (OUT x: ARRAY OF CHAR; OUT ok: BOOLEAN), NEW;

            (VAR var: Item) GetSStringVal (OUT x: ARRAY OF SHORTCHAR; OUT ok: BOOLEAN), NEW;

            (VAR var: Item) PutStringVal (IN x: ARRAY OF CHAR; OUT ok: BOOLEAN), NEW;

            (VAR var: Item) PutSStringVal (IN x: ARRAY OF SHORTCHAR; OUT ok: BOOLEAN), NEW;

            (VAR var: Item) PtrVal (): ANYPTR, NEW;

            (VAR var: Item) PutPtrVal (x: ANYPTR), NEW;

            (VAR var: Item) IntVal (): INTEGER, NEW;

            (VAR var: Item) PutIntVal (x: INTEGER), NEW;

            (VAR var: Item) RealVal (): REAL, NEW;

            (VAR var: Item) PutRealVal (x: REAL), NEW;

            (VAR var: Item) LongVal (): LONGINT, NEW;

            (VAR var: Item) PutLongVal (x: LONGINT), NEW;

            (VAR var: Item) CharVal (): CHAR, NEW;

            (VAR var: Item) PutCharVal (x: CHAR), NEW;

            (VAR var: Item) BoolVal (): BOOLEAN, NEW;

            (VAR var: Item) PutBoolVal (x: BOOLEAN), NEW;

            (VAR var: Item) SetVal (): SET, NEW;

            (VAR var: Item) PutSetVal (x: SET), NEW;

            (VAR type: Item) New (): ANYPTR, NEW;

            (VAR val: Item) Copy (): ANYPTR, NEW;

            (VAR rec: Item) CallWith (proc: PROCEDURE (VAR rec, par: ANYREC); VAR par: ANYREC), NEW

        END;

        Scanner = RECORD

            this-: Item;

            eos-: BOOLEAN;

            (VAR s: Scanner) ConnectToMods, NEW;

            (VAR s: Scanner) ConnectTo (IN obj: Item), NEW;

            (VAR s: Scanner) Scan, NEW;

            (VAR s: Scanner) GetObjName (OUT name: Name), NEW;

            (VAR s: Scanner) Level (): INTEGER, NEW

        END;

    LookupFilter = PROCEDURE (IN path: ARRAY OF CHAR; OUT i: Item; OUT done: BOOLEAN)

    PROCEDURE Lookup (IN name: ARRAY OF CHAR; OUT mod: Item);

    PROCEDURE LookupPath (IN path: ARRAY OF CHAR; OUT i: Item);

    PROCEDURE GetItem (obj: ANYPTR; OUT i: Item);

    PROCEDURE InstallFilter (filter: LookupFilter);

    PROCEDURE UninstallFilter (filter: LookupFilter);

    PROCEDURE GetThisItem (IN attr: ANYREC; OUT i: Item);

END Meta.

*Meta* provides access to Component Pascal run-time type information. *Meta* is restricted to public information, i.e., it doesn't allow access to non-exported items of a module. *Meta* is safe, it doesn't allow to change data which is not exported as modifiable. Generally, *Meta* only allows to do with a module what could be done by a normal client module also. The difference is that *Meta* is more dynamic; it allows inspection and modification of data depending on run-time decisions, without static import of the inspected or modified module.

Constants are not accessible via *Meta*, they are not represented at run-time in order to minimize space overhead.

Examples:

[<u>ObxCtrls</u>](../../Obx/Mod/Ctrls.odc.md)    slider control, extended from Controls.Control

[<u>ObxFldCtrls</u>](../../Obx/Mod/FldCtrls.odc.md)    special-purpose text field control, extended from Controls.Control

How to call procedures using *Meta*:

In order to call an arbitrary procedure (methods are not possible) whose signature is statically known, the following must be done: first, an item must be created that describes the function:

    Meta.Lookup(moduleName, item);

    IF item.obj = Meta.modObj THEN

        item.Lookup(procedureName, item);

        IF item.obj = Meta.procObj THEN

            item.GetVal(item0, ok);

            IF ok THEN

                item0.fun(x)

                ...

The item *item* is a normal, non-extended *Meta.Item* item. In contrast, *item0* must be an extension of *Meta.Value* that contains as one additional field a procedure variable of the correct type:

    item0: RECORD (Meta.Value)

                fun: PROCEDURE (x: REAL): REAL

            END;

CONST **undef**

Possible result code for object classes, type classes, visibility classes.

CONST **typObj, varObj, procObj, fieldObj, modObj, parObj**

Object classes.

CONST **boolTyp, sCharTyp, charTyp, byteTyp, sintTyp, intTyp, longTyp,**

**            sRealTyp, realTyp, setTyp,** **anyRecTyp, anyPtrTyp, procTyp, recTyp, arrTyp, ptrTyp**

Type classes.

CONST **final, extensible, limited, abstract**

Record attributes.

CONST **hidden, readOnly, exported**

Visibility classes.

CONST **value, in, out, var**

Parameter kinds.

TYPE **Name**

String type for meta item names.

TYPE **Value**

ABSTRACT

A value may be extended exactly once, with a single field.

TYPE **Item (Value)**

**obj-**: INTEGER

**typ-**: INTEGER

**vis-**: INTEGER

**adr-**: INTEGER

Properties of the object represented by the item. The meaning of the fields depends on the class of the object as follows:

Class:    Type    Variable    Procedure    Field    Module    Parameter

*obj:*    *typObj    varObj    procObj    fieldObj    modObj    parObj*

*typ:*    type class    type class    *undef*    type class    *undef*    type class

*vis:*    *undef*    visibility    visibility    visibility    *undef*    kind

*adr:*    0    address    address    offset    0    number

PROCEDURE (VAR i: Item) **Valid** (): BOOLEAN

NEW

Determines whether the item is valid, i.e., initialized, set to a defined type, and its module is still loaded.

PROCEDURE (VAR i: Item) **GetTypeName** (OUT mod, type: Name)

NEW

Get the item's type name and the name of this type's module.

Pre

i.Valid()    20

i.typ >= recTyp    21

module of type is still loaded    24

PROCEDURE (VAR i: Item) **BaseTyp** (): INTEGER

NEW

Returns the item's base type.

Pre

i.Valid()    20

i.typ IN {arrTyp, recTyp, ptrTyp}    21

PROCEDURE (VAR i: Item) **Level** (): INTEGER

NEW

Returns the item's level.

Pre

i.Valid()    20

i.typ IN {recTyp, arrTyp}    21

PROCEDURE (VAR i: Item) **Size** (): INTEGER

NEW

Returns the item's size in bytes.

Pre

i.Valid()    20

i.typ # undef    21

PROCEDURE (VAR arr: Item) **Len** (): INTEGER

NEW

Returns the array's length.

Pre

i.Valid()    20

i.typ = arrTyp    21

PROCEDURE (VAR in: Item) **Lookup** (IN name: ARRAY OF CHAR; VAR i: Item)

NEW

Lookup an item in a module or a field in a record.

Pre

in.Valid()    20

in.obj = modObj  OR  in.typ = recTyp    21

Post

i.obj # undef

    lookup was successful

i.obj = undef

    lookup was not successful

PROCEDURE (VAR i: Item) **GetBaseType** (VAR base: Item)

NEW

Assign *i*'s base type to *base*.

Pre

i.Valid()    20

i.typ IN {recTyp, arrTyp}    21

PROCEDURE (VAR rec: Item) **GetThisBaseType** (level: INTEGER; VAR base: Item)

NEW

Assign *i*'s *level*-th base type to *base*. If the level does not exist, *i.obj* is set to *undef*.

Pre

rec.Valid()    20

rec.typ IN {recTyp, arrTyp}    21

level >= 0  &  level < 16    28

PROCEDURE (VAR proc: Item) **NumParam** (): INTEGER

NEW

Returns the number of parameters of a procedure or procedure type.

Pre

proc.Valid()    20

proc.obj = procObj OR proc.typ = procTyp    21

PROCEDURE (VAR proc: Item) **GetParam** (n: INTEGER; VAR par: Item)

NEW

Assigns the *n*-th parameter of *proc* to *par*. *n* must be in the range 0..proc.NumParam()-1. *par.obj* is set to *parObj*, *par.vis* reflects the kind of the parameter (*value*, *in*, *out*, or *var*), and *par.typ* its type.

Pre

proc.Valid()    20

proc.obj = procObj OR proc.typ = procTyp    21

PROCEDURE (VAR proc: Item) **GetParamName** (n: INTEGER; OUT name: Name)

NEW

Assigns the name of the *n*-th parameter of *proc* to *name*. If the parameter does not exist, name is set to "".

Pre

proc.Valid()    20

proc.obj = procObj OR proc.typ = procTyp    21

PROCEDURE (VAR proc: Item) **GetReturnType** (VAR type: Item)

NEW

Assigns the return type of a procedure or procedure type to *type*. For proper procedures *type.typ* is set to *undef*.

Pre

proc.Valid()    20

proc.obj = procObj OR proc.typ = procTyp    21

PROCEDURE (VAR rec: Item) **Is** (IN type: Value): BOOLEAN

NEW

Perform a type test *rec IS type*.

Pre

rec.Valid()    20

rec.typ = recTyp    21

type IS Item

    type.Valid()    20

    type.typ = recTyp    21

~(type IS Item)

    type.Level() = 1    25

    number of fields of type = 1    26

PROCEDURE (VAR ptr: Item) **Deref** (VAR ref: Item)

NEW

Dereference pointer *ptr* and assign the result to *ref*.

Pre

ptr.typ = ptrTyp    21

ref must be a level 1 record    25

ref must have exactly one field    26

PROCEDURE (VAR arr: Item) **Index** (index: INTEGER; VAR elem: Item)

NEW

Assign the *index*-th element of array *arr* to *elem*.

Pre

arr.Valid()    20

arr.typ = arrTyp    21

arr.obj = varObj    22

PROCEDURE (VAR proc: Item) **Call** (OUT ok: BOOLEAN)

NEW

Call a parameterless procedure.

Pre

proc.Valid()    20

proc.obj = procObj  OR  proc.obj = varObj & proc.typ = procTyp    22

PROCEDURE (VAR proc: Item) **ParamCall** (IN par: ARRAY OF Item; VAR dest: Item;

                                                                OUT ok: BOOLEAN)

NEW

Call a procedure with a given set of parameters. *par* must contain a valid variable of appropriate type for each parameter of the procedure. Depending on the parameter kind, either the variable itself or its actual value is passed to the call. If the procedure returns a value, it is assigned to variable *dest*.

Pre

proc.Valid()    20

proc.obj = procObj OR proc.obj = varObj & proc.typ = procTyp    22

LEN(par) >= proc.NumParam()    32

par[i].Valid()    20

par[i].obj = varObj    22

proc.GetParam(i).vis IN {out, var}

    par[i].vis = exported    27

proc.GetReturnType() valid

    dest.Valid()    20

    dest.obj = varObj    22

    dest.vis = exported    27



PROCEDURE (VAR proc: Item) **ParamCallVal** (IN par: ARRAY OF POINTER TO Value; VAR dest: Value;

                                                                    OUT ok: BOOLEAN)

NEW

Call a procedure with a given set of parameters. *par* must contain a valid variable item or a value record of appropriate type for each parameter of the procedure. Depending on the parameter kind, either the variable itself or its actual value is passed to the call. If the procedure returns a value, it is assigned to the variable item or value record* dest*.

Pre

proc.Valid()    20

proc.obj = procObj OR proc.obj = varObj & proc.typ = procTyp    22

LEN(par) >= proc.NumParam()    32

par[i] IS Item

    par[i].Valid()    20

    par[i].obj = varObj    22

par[i] IS Item & proc.GetParam(i).vis IN {out, var}

    par[i].vis = exported    27

~(par[i] IS Item)

    par[i] is extension of Value    25

    par[i] contains a single field    26

proc.GetReturnType() valid & dest IS Item

    dest.Valid()    20

    dest.obj = varObj    22

    dest.vis = exported    27

proc.GetReturnType() valid & ~(dest IS Item)

    dest is extension of Value    25

    dest contains a single field    26



PROCEDURE (VAR var: Item) **GetVal** (VAR x: Value; OUT ok: BOOLEAN)

NEW

Assigns the value of var to x. The actual type of x must either be *Item *or a custom extension of *Value* containing a single field of any type. In the former case the item denotes the assigned variable, in the latter the value is assigned directly to the field.

Pre

var.Valid()    20

var.obj IN {varObj, procObj}    22

x IS Item

    x.Valid()    20

    x.obj = varObj    22

    x.vis = exported    27

~(x IS Item)

    x is extension of Value    25

    x contains a single field    26

PROCEDURE (VAR var: Item) **PutVal** (IN x: Value; OUT ok: BOOLEAN)

NEW

Assigns the value of x to var. The actual type of x must either be *Item *or a custom extension of *Value* containing a single field of any type. In the former case the item denotes the value, in the latter the value is read directly from to the field.

Pre

var.Valid()    20

var.obj = varObj    22

var.vis = exported    27

x IS Item

    x.Valid()    20

    x.obj IN {varObj, procObj}    22

~(x IS Item)

    x is extension of Value    25

    x contains a single field    26

PROCEDURE (VAR var: Item) **GetStringVal** (OUT x: ARRAY OF CHAR; OUT ok: BOOLEAN)

NEW

Reads a string value from an item.

Pre

var.Valid()    20

var.typ = arrTyp & var.BaseTyp() = charTyp    21

var.obj = varObj    22

PROCEDURE (VAR var: Item) **GetSStringVal** (OUT x: ARRAY OF SHORTCHAR; OUT ok: BOOLEAN)

NEW

Reads a short string value from an item.

Pre

var.Valid()    20

var.typ = arrTyp & var.BaseTyp() = sCharTyp    21

var.obj = varObj    22

PROCEDURE (VAR var: Item) **PtrVal** (): ANYPTR

NEW

Reads a pointer value from an item.

Pre

var.Valid()    20

var.typ IN {anyPtrTyp, ptrTyp}    21

var.obj = varObj    22

PROCEDURE (VAR var: Item) **IntVal** (): INTEGER

NEW

Reads a integer value from an item.

Pre

var.Valid()    20

var.typ IN (sCharTyp, charTyp, byteTyp, sIntTyp, intTyp}    21

var.obj = varObj    22

PROCEDURE (VAR var: Item) **RealVal** (): REAL

NEW

Reads a real value from an item.

Pre

var.Valid()    20

var.typ IN {sRealTyp, realTyp}    21

var.obj = varObj    22

PROCEDURE (VAR var: Item) **LongVal** (): LONGINT

NEW

Reads a long value from an item.

Pre

var.Valid()    20

var.typ = longTyp    21

var.obj = varObj    22

PROCEDURE (VAR var: Item) **CharVal** (): CHAR

NEW

Reads a character value from an item.

Pre

var.Valid()    20

var.typ IN {sCharTyp, charTyp}    21

var.obj = varObj    22

PROCEDURE (VAR var: Item) **BoolVal** (): BOOLEAN

NEW

Reads a boolean value from an item.

Pre

var.Valid()    20

var.typ = boolTyp    21

var.obj = varObj    22

PROCEDURE (VAR var: Item) **SetVal** (): SET

NEW

Reads a set value from an item.

Pre

var.Valid()    20

var.typ = setTyp    21

var.obj = varObj    22

PROCEDURE (VAR var: Item) **PutStringVal** (IN x: ARRAY OF CHAR; OUT ok: BOOLEAN)

NEW

Writes a string value to an item.

Pre

var.Valid()    20

var.typ = arrTyp & var.BaseTyp() = charTyp    21

var.obj = varObj    22

var.vis = exported    27

PROCEDURE (VAR var: Item) **PutSStringVal** (IN x: ARRAY OF SHORTCHAR; OUT ok: BOOLEAN)

NEW

Writes a short string value to an item.

Pre

var.Valid()    20

var.typ = arrTyp & var.BaseTyp() = sCharTyp    21

var.obj = varObj    22

var.vis = exported    27

PROCEDURE (VAR var: Item) **PutPtrVal** (x: ANYPTR)

NEW

Writes a pointer value to an item.

Pre

var.Valid()    20

var.typ IN {anyPtrTyp, ptrTyp}    21

var.obj = varObj    22

var.vis = exported    27

PROCEDURE (VAR var: Item) **PutIntVal** (x: INTEGER)

NEW

Writes an integer value to an item.

Pre

var.Valid()    20

var.typ IN {sCharTyp, charTyp, byteTyp, sIntTyp, intTyp}    21

var.obj = varObj    22

var.vis = exported    27

PROCEDURE (VAR var: Item) **PutRealVal** (x: REAL)

NEW

Writes a real value to an item.

Pre

var.Valid()    20

var.typ IN {sRealTyp, realTyp}    21

var.obj = varObj    22

var.vis = exported    27

PROCEDURE (VAR var: Item) **PutLongVal** (x: LONGINT)

NEW

Writes a long value to an item.

Pre

var.Valid()    20

var.typ = longTyp    21

var.obj = varObj    22

var.vis = exported    27

PROCEDURE (VAR var: Item) **PutCharVal** (x: CHAR)

NEW

Writes a character value to an item.

Pre

var.Valid()    20

var.typ IN {sCharTyp, charTyp}    21

var.obj = varObj    22

var.vis = exported    27

PROCEDURE (VAR var: Item) **PutBoolVal** (x: BOOLEAN)

NEW

Writes a boolean value to an item.

Pre

var.Valid()    20

var.typ = boolTyp    21

var.obj = varObj    22

var.vis = exported    27

PROCEDURE (VAR var: Item) **PutSetVal** (x: SET)

NEW

Writes a set value to an item.

Pre

var.Valid()    20

var.typ = setTyp    21

var.obj = varObj    22

var.vis = exported    27

PROCEDURE (VAR type: Item) **New** (): ANYPTR

NEW

Generates a new empty heap object. The item must be a record or a pointer type. The type of the new object is the same as the pointer, or a pointer to the record described by the item.

Pre

type.Valid()    20

type.typ IN (recTyp, ptrTyp)    21

PROCEDURE (VAR val: Item) **Copy** (): ANYPTR

NEW

The same as *New*, but also copies the contents byte by byte. The item must be a record variable.

Pre

val.Valid()    20

val.typ = recTyp    21

val.obj = varObj    22

PROCEDURE (VAR rec: Item) **CallWith** (proc: PROCEDURE (VAR rec, par: ANYREC); VAR par: ANYREC)

NEW

Call procedure proc with the parameters *rec* (i.e., the item itself, the "self" parameter) and parameter *par*.

Pre

rec.Valid()    20

rec.typ = recTyp    21

rec.obj = varObj    22

TYPE **Scanner**

A scanner allows to iterate over all modules, all items in a module, or all fields in a record.

**this-**: Item

The result of the most recent *Scan* operation.

**eos-**: BOOLEAN

This flag tells whether the most recent *Scan* operation has attempted to read beyond the last item.

PROCEDURE (VAR s: Scanner) **ConnectToMods**

NEW

Each invocation of *s.Scan* will return another module.

Post

s.this.obj = undef

~s.eos

PROCEDURE (VAR s: Scanner) **ConnectTo** (IN obj: Item)

NEW

Connect the scanner to a particular module or record.

Pre

obj.Valid()    20

obj.obj = modObj  OR  obj.typ = recTyp    21

PROCEDURE (VAR s: Scanner) **Scan**

Scan a new item. The result is put into *s.this*. If an attempt was made to scan beyond the last item, *s.eos* is set, otherwise it is cleared.

Pre

s is connected    20

PROCEDURE (VAR s: Scanner) **GetObjName** (OUT name: Name)

NEW

Get the name of the most recently scanned item.

Pre

s.this.Valid()    20

PROCEDURE (VAR s: Scanner) **Level** (): INTEGER

NEW

Returns the scanned record's extension level.

Pre

s.this.Valid()    20

s connecte to record variable    22

**TYPE** LookupFilter = PROCEDURE (IN path: ARRAY OF CHAR; OUT i: Item; OUT done: BOOLEAN)

Type used for extension hook that allows *Meta* to operate remotely, for example.

PROCEDURE **Lookup** (IN name: ARRAY OF CHAR; OUT mod: Item)

Set up an item to a module.

Post

mod.obj = modObject

    lookup was successful

mod.obj = undef

    lookup was not successful

PROCEDURE **LookupPath** (IN path: ARRAY OF CHAR; OUT i: Item)

Lookup an item via a whole designator, starting with a module name.

PROCEDURE **GetItem** (obj: ANYPTR; OUT i: Item)

Create an item out of a pointer variable.

PROCEDURE **InstallFilter** (filter: LookupFilter)

Install an extension hook.

PROCEDURE **UninstallFilter** (filter: LookupFilter);

Uninstall an extension hook.

PROCEDURE **GetThisItem** (IN attr: ANYREC; OUT i: Item)

Used internally in extension hooks (creates an item out of a record variable). Use *Lookup*, *LookupPath*, or *GetItem* instead.

