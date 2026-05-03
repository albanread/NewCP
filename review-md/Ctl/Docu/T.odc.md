**CtlT**

DEFINITION CtlT;

    IMPORT Dates;

    CONST

        shortint = 2; integer = 3; shortreal = 4; real = 5; currency = 6; date = 7; string = 8;

        object = 9; result = 10; boolean = 11; any = 12; interface = 13; byte = 17; enumerator = -1;

    TYPE

        Strg = POINTER TO ARRAY OF CHAR;

        OleCy = LONGINT;

        OleDate = REAL;

        IUnknown = COM.IUnknown;

        IDispatch = WinOleAut.IDispatch;

        RESULT = INTEGER;

        GUID = COM.GUID;

        Variant = WinOleAut.VARIANT;

        ParList = ARRAY [untagged] OF Variant;

        Any = POINTER TO ABSTRACT RECORD

            typeId, dim: SHORTINT;

            (x: Any) Bool (): BOOLEAN, NEW, EXTENSIBLE;

            (x: Any) Byte (): BYTE, NEW, EXTENSIBLE;

            (x: Any) Cy (): OleCy, NEW, EXTENSIBLE;

            (x: Any) Date (): OleDate, NEW, EXTENSIBLE;

            (x: Any) Int (): INTEGER, NEW, EXTENSIBLE;

            (x: Any) Real (): REAL, NEW, EXTENSIBLE;

            (x: Any) SInt (): SHORTINT, NEW, EXTENSIBLE;

            (x: Any) SReal (): SHORTREAL, NEW, EXTENSIBLE;

            (x: Any) Str (): Strg, NEW, EXTENSIBLE

        END;

        Object = POINTER TO ABSTRACT RECORD (Any)

            disp: IDispatch

        END;

        OutObject = POINTER TO ABSTRACT RECORD (Object)

            source: Object;

            (obj: OutObject) GetIID (OUT iid: GUID), NEW, ABSTRACT;

            (obj: OutObject) Invoke (id, n: INTEGER; VAR par: ParList; VAR ret: Variant), NEW, ABSTRACT

        END;

        ByteT = POINTER TO RECORD (Any) val: BYTE END;

        ShortInt = POINTER TO RECORD (Any) val: SHORTINT END;

        Integer = POINTER TO RECORD (Any) val: INTEGER END;

        ShortReal = POINTER TO RECORD (Any) val: SHORTREAL END;

        RealT = POINTER TO RECORD (Any) val: REAL END;

        Boolean = POINTER TO RECORD (Any) val: BOOLEAN END;

        Result = POINTER TO RECORD (Any) val: RESULT END;

        Currency = POINTER TO RECORD (Any) val: OleCy END;

        DateT = POINTER TO RECORD (Any) val: OleDate END;

        String = POINTER TO RECORD (Any) val: Strg END;

        Interface = POINTER TO RECORD (Any) val: IUnknown END;

        AnyArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF Any END;

        ObjectArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF Object END;

        ByteArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF BYTE END;

        ShortIntArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF SHORTINT END;

        IntegerArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF INTEGER END;

        ShortRealArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF SHORTREAL END;

        RealArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF REAL END;

        BooleanArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF BOOLEAN END;

        ResultArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF RESULT END;

        CurrencyArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF OleCy END;

        DateArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF OleDate END;

        StringArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF Strg END;

        InterfaceArray = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF IUnknown END;

        AnyArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF Any END;

        ObjectArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF Object END;

        ByteArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF BYTE END;

        ShortIntArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF SHORTINT END;

        IntegerArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF INTEGER END;

        ShortRealArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF SHORTREAL END;

        RealArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF REAL END;

        BooleanArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF BOOLEAN END;

        ResultArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF RESULT END;

        CurrencyArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF OleCy END;

        DateArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF OleDate END;

        StringArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF Strg END;

        InterfaceArray2 = POINTER TO RECORD (Any) p: POINTER TO ARRAY OF ARRAY OF IUnknown END;

        Enumerator = POINTER TO ABSTRACT RECORD

            (e: Enumerator) First (): Any, NEW, ABSTRACT;

            (e: Enumerator) Next (): Any, NEW, ABSTRACT

        END;

    VAR

        context: SET;

        lcid: INTEGER;

    PROCEDURE OleDateFromDateAndTime (IN date: Dates.Date; IN time: Dates.Time): OleDate;

    PROCEDURE OleDateToDateAndTime (d: OleDate; OUT date: Dates.Date; OUT time: Dates.Time);



    PROCEDURE Obj (disp: IDispatch): Object;

    PROCEDURE Byte (val: BYTE): ByteT;

    PROCEDURE SInt (val: SHORTINT): ShortInt;

    PROCEDURE Int (val: INTEGER): Integer;

    PROCEDURE SReal (val: SHORTREAL): ShortReal;

    PROCEDURE Real (val: REAL): RealT;

    PROCEDURE Bool (val: BOOLEAN): Boolean;

    PROCEDURE Res (val: RESULT): Result;

    PROCEDURE Cy (val: OleCy): Currency;

    PROCEDURE Date (val: OleDate): DateT;

    PROCEDURE Str (IN val: ARRAY OF CHAR): String;

    PROCEDURE Intfce (val: IUnknown): Interface;

    PROCEDURE AnyArr (IN val: ARRAY OF Any): AnyArray;

    PROCEDURE ObjArr (IN val: ARRAY OF Object): ObjectArray;

    PROCEDURE ByteArr (IN val: ARRAY OF BYTE): ByteArray;

    PROCEDURE SIntArr (IN val: ARRAY OF SHORTINT): ShortIntArray;

    PROCEDURE IntArr (IN val: ARRAY OF INTEGER): IntegerArray;

    PROCEDURE SRealArr (IN val: ARRAY OF SHORTREAL): ShortRealArray;

    PROCEDURE RealArr (IN val: ARRAY OF REAL): RealArray;

    PROCEDURE BoolArr (IN val: ARRAY OF BOOLEAN): BooleanArray;

    PROCEDURE ResArr (IN val: ARRAY OF RESULT): ResultArray;

    PROCEDURE CyArr (IN val: ARRAY OF OleCy): CurrencyArray;

    PROCEDURE DateArr (IN val: ARRAY OF OleDate): DateArray;

    PROCEDURE StrArr (IN val: ARRAY OF Strg): StringArray;

    PROCEDURE IntfceArr (IN val: ARRAY OF IUnknown): InterfaceArray;



    PROCEDURE AnyArr2 (IN val: ARRAY OF ARRAY OF Any): AnyArray2;

    PROCEDURE ObjArr2 (IN val: ARRAY OF ARRAY OF Object): ObjectArray2;

    PROCEDURE ByteArr2 (IN val: ARRAY OF ARRAY OF BYTE): ByteArray2;

    PROCEDURE SIntArr2 (IN val: ARRAY OF ARRAY OF SHORTINT): ShortIntArray2;

    PROCEDURE IntArr2 (IN val: ARRAY OF ARRAY OF INTEGER): IntegerArray2;

    PROCEDURE SRealArr2 (IN val: ARRAY OF ARRAY OF SHORTREAL): ShortRealArray2;

    PROCEDURE RealArr2 (IN val: ARRAY OF ARRAY OF REAL): RealArray2;

    PROCEDURE BoolArr2 (IN val: ARRAY OF ARRAY OF BOOLEAN): BooleanArray2;

    PROCEDURE ResArr2 (IN val: ARRAY OF ARRAY OF RESULT): ResultArray2;

    PROCEDURE CyArr2 (IN val: ARRAY OF ARRAY OF OleCy): CurrencyArray2;

    PROCEDURE DateArr2 (IN val: ARRAY OF ARRAY OF OleDate): DateArray2;

    PROCEDURE StrArr2 (IN val: ARRAY OF ARRAY OF Strg): StringArray2;

    PROCEDURE IntfceArr2 (IN val: ARRAY OF ARRAY OF IUnknown): InterfaceArray2;



    PROCEDURE Connect (sink: OutObject; source: Object);

    PROCEDURE Disconnect (sink: OutObject; source: Object);

    PROCEDURE Disp (obj: Object): IDispatch;

END CtlT.

Base module for OLE automation controller applications. For an introduction to automation controller development see the [<u>Developer Manual</u>](Dev-Man.odc.md).

CONST** any, object, byte, shortint, integer**, **shortreal**, **real, boolean,**

**            result, currency, date, string, interface, enumerator**

Constants used as values of the *typeId* field of type *Any.*

TYPE **Strg**

Basic string type. Used for string-typed out- and in/out-parameters and return values. For in-parameters the type ARRAY OF CHAR is used to permit string constants as actual parameters.

TYPE **OleCy**

Basic currency type used in OLE. *OleCy* is a fixed point number with 4 decimal digits after the decimal point, stored in a 64 bit integer. The integer value 73500 corresponds to the currency value 7.35.

TYPE **OleDate**

Basic date/time type used in OLE. *OleDate* is a 64 bit floating point number containing a fractional day count. The value 2.0 corresponds to midnight 1.1.1900. The exact conversion formula is:

x := ((second / 60 + minute) / 60 + hour) / 24 + day + 2

where *x* is the resulting real number and *day* is the day count since 1.1.1900.

The procedures *OleDateFromDateAndTime* and *OleDateToDateAndTime* are provided for conversions between *OleDate* and the BlackBox types *Dates.Date* amd *Dates.Time.*

TYPE **IUnknown**, **IDispatch**, **RESULT**, **GUID**, **Variant**, **ParList**

OLE types used internally in the automation interface modules.

TYPE **Any**

ABSTRACT

Base type of all automation types. Covers all possible automation types including arrays. *Any* is used if the type of a parameter of an automation method is not statically defined. *NIL* can be used for parameters of type *Any* if the parameter is not used at all (legal for optional parameters).

**typeId**: SHORTINT

**dim**: SHORTINT

Type and (array-) dimension of the represented value. *dim* is 0 for a scalar value and > 0 for an array. *typeId* can be one of the basic types *byte*, *shortint*, *integer*, *shortreal*, *real*, *boolean*, *result*, *currency*, *date*, *string*, or *interface*; or one of the special types *object*, *enumerator*, or *any*. *typeId* = *enumerator* implies *dim* = 0. *typeId* = *any* is only used with *dim* > 0 and means an array with unspecified or inhomogeneous element type.

PROCEDURE (x: Any) **Bool** (): BOOLEAN

PROCEDURE (x: Any) **Byte** (): BYTE

PROCEDURE (x: Any) **Cy** (): OleCy

PROCEDURE (x: Any) **Date** (): OleDate

PROCEDURE (x: Any) **Int** (): INTEGER

PROCEDURE (x: Any) **Real** (): REAL

PROCEDURE (x: Any) **SInt** (): SHORTINT

PROCEDURE (x: Any) **SReal** (): SHORTREAL

PROCEDURE (x: Any) **Str** (): Strg

NEW, EXTENSIBLE

Used for convenient access to basic type values. Values are converted to the return type if necessary.

Pre

value type is convertible to result type    20

TYPE **Object (Any)**

ABSTRACT

Base type of all automation objects implemented in automation interface modules.

**disp**: IDispatch

Used internally.

TYPE **OutObject (Object)**

ABSTRACT

Base type of all callback objects. An abstract extension with the specific methods is declared in the automation interface module. A concrete implementation can be declared in a user module and connected to an event source by *Connect* or *OleViews.Connect*.

**source**: Object;

The connected source object.

PROCEDURE (obj: OutObject) **GetIID** (OUT iid: GUID)

PROCEDURE (obj: OutObject) **Invoke** (id, n: INTEGER; VAR par: ParList; VAR ret: Variant)

NEW, ABSTRACT

Implemented in the automation interface module, used internally.

TYPE **ByteT**, **ShortInt**, **Integer**, **ShortReal**, **RealT**, **Boolean**, **Result**, **Currency**, **DateT**, **String**, **Interface**

Concrete implementations of scalar basic type objects.

**val**: <type>

Actual value.

TYPE **AnyArray, ObjectArray, ByteArray, ShortIntArray, IntegerArray, ShortRealArray, RealArray, BooleanArray, ResultArray, CurrencyArray, DateArray, StringArray, InterfaceArray**

Concrete implementations of one-dimensional array objects.

**p**: POINTER TO ARRAY OF <type>

Actual values.

TYPE **AnyArray2, ObjectArray2, ByteArray2, ShortIntArray2, IntegerArray2,** **ShortRealArray2,**

**        ** **RealArray2,** **BooleanArray2,** **ResultArray2,** **CurrencyArray2,** **DateArray2,** **StringArray2,**

    **     InterfaceArray2**

Concrete implementations of two-dimensional array objects.

**p**: POINTER TO ARRAY OF ARRAY OF <type>

Actual values.

TYPE **Enumerator**

ABSTRACT

Base type of enumerators. Enumerators are used to access all elements of a collection object in a systematic way. A concrete enumerator can be requested from a collection object through the method *_NewEnum.*

PROCEDURE (e: Enumerator) **First** (): Any

NEW, ABSTRACT

Returns the first object from a collection or *NIL* if the collection is empty.

PROCEDURE (e: Enumerator) **Next** (): Any

NEW, ABSTRACT

Returns the next consecutive object from a collection or *NIL* if the there are no more objects in the collection.

VAR **context**: SET

The COM object allocation contect used to allocate new automation objects. Initialized to *WinOle.CLSCTX_INPROC_SERVER* + *WinOle.CLSCTX_LOCAL_SERVER*. Can be changed if a different allocation context should be used.

VAR **lcid**: INTEGER

The language identifier used in automation method calls. Initialized to *WinApi.LOCALE_SYSTEM_DEFAULT* . Can be changed if a different language identifier should be used.

PROCEDURE **OleDateFromDateAndTime** (IN date: Dates.Date; IN time: Dates.Time): OleDate

Converts BlackBox *Date* and *Time* structures to an *OleDate* value.

PROCEDURE **OleDateToDateAndTime** (d: OleDate; OUT date: Dates.Date; OUT time: Dates.Time)

Converts an *OleDate* value to BlackBox *Date* and *Time* structures.

PROCEDURE **Connect** (sink: OutObject; source: Object)

Used to connect a callback object (an object implementing a callback interface) to an event source.

Pre

sink is not connected    20

source is connectable    21

PROCEDURE **Disconnect** (sink: OutObject; source: Object)

Used to disconnect a callback object (an object implementing a callback interface) from an event source.

Pre

sink is connected to source    20

PROCEDURE **Obj** (disp: IDispatch): Object

PROCEDURE **Byte** (val: BYTE): ByteT

PROCEDURE **SInt** (val: SHORTINT): ShortInt

PROCEDURE **Int** (val: INTEGER): Integer

PROCEDURE **SReal** (val: SHORTREAL): ShortReal

PROCEDURE **Real** (val: REAL): RealT

PROCEDURE **Bool** (val: BOOLEAN): Boolean

PROCEDURE **Res** (val: RESULT): Result

PROCEDURE **Cy** (val: OleCy): Currency

PROCEDURE **Date** (val: OleDate): DateT

PROCEDURE **Str** (IN val: ARRAY OF CHAR): String

PROCEDURE **Intfce** (val: IUnknown): Interface

Generator functions for scalar basic type objects.

PROCEDURE **AnyArr** (IN val: ARRAY OF Any): AnyArray

PROCEDURE **ObjArr** (IN val: ARRAY OF Object): ObjectArray

PROCEDURE **ByteArr** (IN val: ARRAY OF BYTE): ByteArray

PROCEDURE **SIntArr** (IN val: ARRAY OF SHORTINT): ShortIntArray

PROCEDURE **IntArr** (IN val: ARRAY OF INTEGER): IntegerArray

PROCEDURE **SRealArr** (IN val: ARRAY OF SHORTREAL): ShortRealArray

PROCEDURE **RealArr** (IN val: ARRAY OF REAL): RealArray

PROCEDURE **BoolArr** (IN val: ARRAY OF BOOLEAN): BooleanArray

PROCEDURE **ResArr** (IN val: ARRAY OF RESULT): ResultArray

PROCEDURE **CyArr** (IN val: ARRAY OF OleCy): CurrencyArray

PROCEDURE **DateArr** (IN val: ARRAY OF OleDate): DateArray

PROCEDURE **StrArr** (IN val: ARRAY OF Strg): StringArray

PROCEDURE **IntfceArr** (IN val: ARRAY OF IUnknown): InterfaceArray

Generator functions for one-dimensional array objects.

PROCEDURE **AnyArr2** (IN val: ARRAY OF ARRAY OF Any): AnyArray2

PROCEDURE **ObjArr2** (IN val: ARRAY OF ARRAY OF Object): ObjectArray2

PROCEDURE **ByteArr2** (IN val: ARRAY OF ARRAY OF BYTE): ByteArray2

PROCEDURE **SIntArr2** (IN val: ARRAY OF ARRAY OF SHORTINT): ShortIntArray2

PROCEDURE **IntArr2** (IN val: ARRAY OF ARRAY OF INTEGER): IntegerArray2

PROCEDURE **SRealArr2** (IN val: ARRAY OF ARRAY OF SHORTREAL): ShortRealArray2

PROCEDURE **RealArr2** (IN val: ARRAY OF ARRAY OF REAL): RealArray2

PROCEDURE **BoolArr2** (IN val: ARRAY OF ARRAY OF BOOLEAN): BooleanArray2

PROCEDURE **ResArr2** (IN val: ARRAY OF ARRAY OF RESULT): ResultArray2

PROCEDURE **CyArr2** (IN val: ARRAY OF ARRAY OF OleCy): CurrencyArray2

PROCEDURE **DateArr2** (IN val: ARRAY OF ARRAY OF OleDate): DateArray2

PROCEDURE **StrArr2** (IN val: ARRAY OF ARRAY OF Strg): StringArray2

PROCEDURE **IntfceArr2** (IN val: ARRAY OF ARRAY OF IUnknown): InterfaceArray2

Generator functions for two-dimensional array objects.

PROCEDURE **Disp** (obj: Object): IDispatch

Used internally.

