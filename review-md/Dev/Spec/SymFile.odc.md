**Oberon/F Symbol File Format**    bh

漜

SymFile    =    tag4 processor version Module { Object } .

Module    =    0 | negmno | [ LIB name ] MNAME name .

Object    =    [ SYS value ] [ LIB name ] [ ENTRY name ]

            ( Constant name

            | TYPE Struct

            | ALIAS Struct name

            | (RVAR | VAR) Struct name

            | (XPRO | IPRO) Signature name

            | CPRO Signature len { code1 } name

            ).

Constant    =      CHAR8 value1

        | CHAR16 value2

        | BOOL ( FALSE | TRUE )

        | ( INT8 | INT16 | INT32 | INT64 | SET ) value

        | REAL32 value4

        | REAL64 value8

        | STRING8 name

        | STRING16 length utf8string

        | NIL .

Struct    =      negref

        | STRUCT Module name [ SYS value ] [ LIB name ] [ ENTRY name ] [ STRING name ]

            ( PTR Struct

            | ARR Struct nofElem

            | DARR Struct

            | ( REC | ABSREC | EXTREC | FINREC ) Struct size align nofMeth { Field } { Method } END

            | PRO Signature

            | ALIAS Struct

            ) .

Field    =    ( FLD | RFLD ) Struct name offset | ( HDPTR | SYS value | HDPRO ) offset .

Method    =    [ IMPO ] [ ENTRY name ]

            ( TPRO | ABSPRO | EMPPRO | EXTPRO | FINPRO ) Signature name methno | HDTPRO methno .

Signature    =    Struct { [ SYS value ] ( VALPAR | VARPAR | INPAR | OUTPAR ) Struct offset name } END .

MNAME    =    16        BYTE    =    1    byte    =    1

END     =    18        BOOL    =    2    boolean    =    2

TYPE    =    19        CHAR8    =    3    char8    =    3

ALIAS    =    20        INT8    =    4    int8    =    4

VAR    =    21        INT16    =    5    int16    =    5

RVAR    =    22        INT32    =    6    int32    =    6

VALPAR    =    23        REAL32    =    7    real32    =    7

VARPAR    =    24        REAL64    =    8    real64    =    8

FLD    =    25        SET    =    9    set    =    9

RFLD    =    26        STRING8    =    10

HDPTR    =    27        NIL    =    11

HDPRO    =    28                    notyp    =    12

TPRO    =    29                    sysptr    =    13

HDTPRO    =    30                    anypointer    =    14

XPRO    =    31                    anyrecord    =    15

IPRO    =    32        CHAR16    =    16    char16    =    16

CPRO    =    33        STRING16    =    17

STRUCT    =    34        INT64    =    18    int64    =    18

SYS    =    35                    result    =    20

PTR    =    36                    iunknown    =    21

ARR    =    37                    ptriunk    =    22

DARR    =    38                    guid    =    23

REC    =    39        HDUTPTR    =    41

PRO    =    40        LIB    =    42

INPAR     =    25        ENTRY    =    43    FALSE    =    0

OUTPAR    =    26        FINPRO    =    31    TRUE    =    1

FINREC    =    25        ABSPRO    =    32

ABSREC    =    26        EMPPRO    =    33

EXTREC    =    27        EXTPRO    =    34    IMPO    =    22
