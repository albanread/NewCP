** Direct-To-COM Compiler**

<a id="3"></a>**The Direct-To-COM Compiler**

**Contents**

[<u>COM Compiler Extensions</u>](#3.1)

[<u>COM Sysflags for Interface Structures</u>](#3.2)

[<u>COM sysflags for VAR parameters</u>](#3.3)

[<u>The Module COM</u>](#3.4)

[<u>New Predefined Procedures</u>](#3.5)

[<u>Unions</u>](#3.6)

[<u>COM Programming Hints</u>](#3.7)

[<u>Compiler Implementation Restrictions</u>](#3.8)

The Direct-To-COM (DTC) compiler is a Component Pascal compiler that supports Microsoft's Component Object Model (COM) binary interface standard. In DTC Component Pascal, COM objects are declared and implemented in a superset of the Component Pascal language. They can be used from any other language, and COM objects implemented in any language can be used from Component Pascal.

In Component Pascal, COM interfaces are mapped to special Component Pascal records. Interface pointers are pointers to such records. The virtual function table is the method table of the Component Pascal type descriptor. It is hidden from the programmer. A COM interface can be defined in the following way:

        **ILookup*** = POINTER TO ABSTRACT RECORD

                                ["{C4910D71-BA7D-11CD-94E8-08001701A8A3}"] (COM.IUnknown) END;

        PROCEDURE (this: ILookup) **LookupByName***(

            name: WinApi.PtrWSTR; OUT number: WinApi.PtrWSTR): COM.RESULT, NEW, ABSTRACT;



        PROCEDURE (this: ILookup) **LookupByNumber***(

            number: WinApi.PtrWSTR; OUT name: WinApi.PtrWSTR): COM.RESULT, NEW, ABSTRACT;

Interface definitions can be derived from one another using Component Pascal subtyping. The above interface is a subtype of the interface *COM.IUnknown*. The interface identifier (IID) is specified in the declaration of the interface. The interface is declared to be abstract and cannot be instantiated. Concrete implementations of interface objects are extensions of interface records. In contrast to the interface records which are referenced through (reference-counted) interface pointers, the implementation records are referenced through regular Component Pascal pointers.

The COM standard uses reference counting as its memory management strategy. Reference counting errors, however, are the most difficult and dangerous errors when programming to the OLE interfaces. The "Safer OLE" technology of the DTC compiler adds automatic memory management to COM objects. In contrast to C and C++, programmers using the DTC Component Pascal compiler do not need to worry about reference counting. The compiler automatically generates calls of an object's *AddRef* and *Release* methods where necessary, and the compiler also automatically implements the *AddRef* and *Release* methods for interfaces implemented in Component Pascal. When a COM object is no longer used, it is automatically removed from memory. Components written in DTC Component Pascal become more reliable and maintainable.

In Component Pascal, the above example looks as follows:

    VAR

        factory: IFactory;

        animal1, animal2: IAnimal;

        hr: COM.RESULT;



    BEGIN

        hr := GetFactory(COM.ID(IFactory), factory);

        IF hr >= 0 THEN

            hr := factory.CreateInstance(NIL, COM.ID(animal1), animal1);

            IF hr >= 0 THEN

                animal2 := animal1;

                animal2.Sleep();

                hr := factory.CreateInstance(NIL, COM.ID(animal1), animal1);

                IF hr >= 0 THEN

                    animal1.Eat()

                END

            END

        END

    END

Calls to the reference counting methods are generated for assignments to interface pointer variables. The garbage collector recognizes COM interface implementations and does not remove such objects with a reference count greater than zero. If the garbage collector finds a COM interface object with reference count zero, then all interface pointers stored in this object are released before the object is collected. The same holds for local variables if the scope is left and for global variables that are stored in a component to be unloaded. This automatic garbage collection mechanism just generates code that would have to be written manually in any case. Thus it does not require any extra run time and is fully compatible with other components, created with other tools.

The DTC compiler is integrated in the development environment BlackBox. The debugger allows to inspect components and objects on a symbolic level. In order to browse through dynamic memory, you use hyper-text links to follow pointers.

Effective support for COM in Component Pascal requires special language constructs, e.g. to specify the unique id of a COM interface. We clearly separate these language extensions from the core language. The reason is that such interfacing features should and need not be used in normal Component Pascal modules, and moreover, COM is only one of an increasing number of object models which could be supported. Modules using the special language constructs must flag this fact in the module header. If you look at the beginning of a module source, it should immediately be clear whether this is an (unportable) module and whether it is unsafe or not. We already have followed this principle in earlier products: system flags and other similar features are only allowed in modules which import the pseudo-module *SYSTEM*. This module and the necessary compiler extensions are not considered part of the language definition proper. In a similar vein, a pseudo-module COM needs to be imported in order to make the special COM features available in a module. This module is not considered a part of Component Pascal itself.

To quote Prof. N. Wirth from the ETH technical report #82 (From Modula to Oberon):

"It appears preferrable to drop the pretense of portability of programs that import a "standard", yet system-specific module. Both, the module SYSTEM and the type transfer functions are therefore eliminated, and with them also the types ADDRESS and WORD. Individual implementations are free to provide system-dependent modules, but they do not belong to the general language definition. Their use then declares a program to be patently implementation-specific, and thereby non-portable."

In this spirit, procedural DLLs and COM DLLs are system-specific. A software system should minimize the number and size of modules directly using them, and encapsulate the latter such that their module interfaces only use the core Component Pascal features wherever possible (abstraction!).

For this reason it is considered good software engineering practice to structure the implementation of a complex COM object such that the number of objects (and their modules) that contain references to interfaces are minimized and they are concentrated at the "top" and "bottom" of the module hierarchy, so that the intermediate layer consists of pure Component Pascal modules which don't use any special COM features:

This leads to a module structure such as the following, where the top-level module implements the COM interfaces (export), the bottom-level modules implement access to existing COM interfaces (import), while the intermediate modules are implemented in portable, safe, and non-COM-specific modules:

<a id="3.1"></a>**COM Compiler Extensions**

The compiler features added in order to support COM are modest. In particular, new sysflags for interface records and VAR parameters have been defined and union records are provided. Some additional types and functions are defined in the pseudo module COM.

In order to access these extended facilities, the pseudo module COM must be imported. In the following, the special COM extensions are explained.

<a id="3.2"></a>**COM Sysflags for Interface Structures**

Interface records are marked by the *interface *sysflag* *or by a GUID (globally unique identifier) flag. The GUID string must be a valid GUID constant spelled out in hex and wrapped in braces.

        TYPE

            IExample = POINTER TO ABSTRACT RECORD

                ["{91C074A1-C2D7-11CF-A4A1-444553540000}"] (COM.IUnknown) END;



        PROCEDURE (e: IExample) MethodA(): COM.RESULT, NEW, ABSTRACT;



Instead of an explicit GUID, an interface record can also be marked with the interface sysflag. Such interfaces however cannot be requested through a QueryInterface call as they have no identifier. They can be used to implement outgoing interfaces which are not explicitely requested.

        TYPE

            IExample = POINTER TO ABSTRACT RECORD [interface] (COM.IUnknown) END;



        PROCEDURE (e: IExample) MethodA(): COM.RESULT, NEW, ABSTRACT

Interface records and interface pointers must be (direct or indirect) extensions of COM.IUnknown. Interface records must be abstract and therefore must not contain any fields. Procedures bound to interface records must also be abstract. Abstract records cannot be allocated. The only legal usage of an interface record is as a record or pointer base type or as type of an interface pointer variable.

An implementation extension of an interface must overwrite all abstract procedures bound to the interface record. Exceptions are the procedures defined by COM.IUnknown:

The following example defines an implementation of the *IExample* interface. By convention, type names of interface pointers start with an I, and type names of (implementing) classes with a C.

        TYPE

            CExample = POINTER TO RECORD (IExample) END;



        PROCEDURE (e: CExample) MethodA(): COM.RESULT;

        BEGIN RETURN WinApi.E_NOTIMPL

        END MethodA;





<a id="3.3"></a>**COM Sysflags for VAR Parameters**

[nil]    The *nil* flag for VAR parameters indicates that NIL may also be used as an actual parameter.

With the function VALID (see below) it can be tested whether an actual VAR parameter is valid, i.e. is not NIL. If the actual parameter is not valid (i.e. is NIL), then it must neither be read nor written. It may only be passed as actual parameter for another nil-parameter. Attempts to read or write an invalid nil parameter leads to a NIL dereference trap.

[new] [iid]    The iid and new VAR parameter flags allow to specify polymorphic out parameters. They allow to implement *QueryInterface-*like operations.

The *new* and *iid* parameters must always be paired in a parameter list. A *new* parameter must be followed by a corresponding *iid* parameter or vice versa. A parameter list may contain only one *new-iid* pair**.**

The *iid* parameter must be an IN parameter of type GUID, and the *new* parameter must be an OUT parameter of an interface type.

The type passed as *iid* parameter defines the static type of the polymorphic out parameter. The dynamic type of the out parameter can be a subtype of the asserted static type. The actual parameter passed for the interface parameter may be a base type of the asserted static type and may be an extension of the formal parameter. In other words, the following relation must be satisfied for the actual parameters of a polymorphic out parameter (<= refers to the subtype relation)

            type of formal *new* parameter

            <= static type of actual *new* parameter

            <= type specified through* iid* parameter

            <= dynamic type of actual *new* parameter (on return)



The following program fragment shows some examples of the use of polymorphic out parameters.

        TYPE

            IExample = POINTER TO ABSTRACT RECORD

                ["{91C074A1-C2D7-11CF-A4A1-444553540000}"] (COM.IUnknown) END;



            IExample2 = POINTER TO ABSTRACT RECORD

                ["{91C074A2-C2D7-11CF-A4A1-444553540000}"] (IExample) END;



        PROCEDURE GetInterface(IN [iid] id: COM.GUID; OUT [new] int: COM.IUnknown);

        END GetInterface;



        PROCEDURE Test;

            VAR i0: COM.IUnknown; i1: IExample; i2: IExample2;

        BEGIN

            GetInterface(COM.ID(i1), i0); (* valid *)

            GetInterface(COM.ID(i1), i1); (* valid *)

            GetInterface(COM.ID(i1), i2); (* compiler error: wrong [iid] - [new] pair *)

        END Test;



A procedure like

    PROCEDURE QueryInterface (IN [iid] id: COM.GUID; OUT [new] int: COM.IUnknown);

can be called in four different ways:

-     QueryInterface(COM.ID(p1), p0) where p0 is an extension of COM.IUnknown and p1 is an extension of p0.

            VAR i1: IExample; i2: IExample2;

            QueryInterface(COM.ID(i2), i1);

-    QueryInterface(COM.ID(T), p) where p is an extension of COM.IUnknown and T is a subtype of the type of p.

            VAR i1: IExample;

            QueryInterface(COM.ID(IExample2), i1);

-    QueryInterface(id, p) where id is an arbitrary GUID and p is of type COM.IUnknown.

            VAR p: COM.IUnknown;

            QueryInterface("{91C074A1-C2D7-11CF-A4A1-444553540000}", p);

-    QueryInterface(id, int) where id and int are another [iid], [new] pair.

            PROCEDURE P (IN [iid] id: COM.GUID; OUT [new] int: COM.IUnknown);

            BEGIN QueryInterface(id, int)

            END P;

In a procedure with an *iid-new* parameter pair, the interface parameter cannot be assigned directly, but *id* and *int* can be used as formal parameters to another [new], [iid] pair or to COM.QUERY (see below).

<a id="3.4"></a>**The Module COM**

The module *COM* contains certain types and procedures that are necessary to implement COM components. As module SYSTEM, module COM is a pseudo module, i.e., no symbol file exists for module COM. If module COM is imported, this is only a declaration for the compiler. A pseudo description of the interface is shown below:

    DEFINITION COM;

        TYPE

            RESULT = INTEGER;

            GUID = ARRAY 16 OF BYTE;    *(* modified compare semantics *)*



            IUnknown = POINTER TO ABSTRACT RECORD ["{00000000-0000-0000-C000-000000000046}"]

                (this: IUnknown) QueryInterface (IN [iid] iid: GUID; OUT [new] int: IUnknown): RESULT,

                                                                                                                                 NEW, ABSTRACT;

                (this: IUnknown) AddRef, NEW, ABSTRACT;    *(* hidden, can neither be called nor overwritten *)*

                (this: IUnknown) Release, NEW, ABSTRACT;    *(* hidden, can neither be caled nor overwritten *)*

*    *            (this: IUnknown) RELEASE-, NEW, EXTENSIBLE;

            END;



        PROCEDURE QUERY (p: IUnknown; IN [iid] id: GUID; OUT [new] ip: IUnknown): BOOLEAN;



        PROCEDURE ID (t: *INTERFACETYPE*): GUID;

        PROCEDURE ID (p: IUnknown): GUID;

        PROCEDURE ID (s: ARRAY OF CHAR): GUID;



    END COM.

TYPE **RESULT**

Result type of most COM operations (alias of INTEGER). The error translator of the Direct-To-COM development environment can be used to obtain a meaningful description from such a result.

TYPE** GUID**

Type for globally unique identifiers (GUIDs). A GUID is a 128 bit identifier. String constant expressions representing a valid GUID can be assigned to a GUID variable.

    VAR id: COM.GUID;

        id := "{12345678-1000-11cf-adf0-444553540000}";

TYPE **IUnknown**

COM Interface, ABSTRACT

The type IUnknown is the base type of all COM interfaces. IUnknown is a pointer to an abstract record.

PROCEDURE (this: IUnknown) **AddRef**;

NEW, HIDDEN

PROCEDURE (this: IUnknown) **Release**;

NEW, HIDDEN

The AddRef and Release methods are necessary to control the reference count of a COM object. AddRef and Release are always handled implicitly, they can neither be overwritten nor called.

PROCEDURE (this: IUnknown) **QueryInterface** (IN [iid] iid: GUID; OUT [new] int: IUnknown): RESULT;

NEW, DEFAULT

QueryInterface is used to ask for interfaces a COM object supports. It is implemented by default but may be overwritten if special behaviour is needed (e.g. for objects which support several interfaces). For an example see module *ComObject*.

The default implementation of QueryInterface has the following form. The pointer this.outer is a hidden reference to another COM object which can be specified with the two-argument NEW procedure (see below).

    PROCEDURE (this: IUnknown) QueryInterface (

                        IN [iid] iid: COM.GUID; OUT [new] int: COM.IUnknown): COM.RESULT;

    BEGIN

        IF this.outer # NIL THEN

            **RETURN **this.outer.QueryInterface(iid, int)

        ELSE

            IF COM.QUERY(this, iid, int) THEN **RETURN** WinApi.S_OK

            ELSE **RETURN** WinApi.E_NOINTERFACE

            END

        END

    END QueryInterface;

PROCEDURE (this: IUnknown) **RELEASE-;**

NEW, EMPTY

The RELEASE method is called whenever the reference cout drops from one to zero. Note that if the system calls the RELEASE method, the interface is not necessarily removed by the garbage collector as Component Pascal pointer references to the object may still exist.

RELEASE gives additional control over COM objects besides the FINALIZE method which is provided for all (tagged) Component Pascal records. In contrast to the FINALIZE method, the RELEASE method may be called several times.

PROCEDURE **ID** (t: *INTERFACETYPE*): GUID

The actual parameter must be the name of a type.

Returns the GUID associated with interface type t (pointer or record).

PROCEDURE **ID** (p: IUnknown): GUID

Returns the GUID associated with the static type of the actual parameter passed for p.

PROCEDURE **ID** (s: ARRAY OF CHAR): GUID

Returns the GUID from a textual representation (string constant).

PROCEDURE **QUERY** (p: COM.IUnknown;

                                    IN [iid] id: COM.GUID, OUT [new] ip: COM.IUnknown): BOOLEAN;

QUERY allows safe assignments to polymorphic out parameters.

If there exists a type t with the following properties:

    - t is an interface type

    - p points to an extension of t

    - ID(t) = id

then p is assigned to ip and the function returns TRUE. Otherwise ip remains unchanged and the function returns FALSE. COM.QUERY allows safe and simple implementations of QueryInterface without any restrictions.

<a id="3.5"></a>**New Predefined Procedures**

In the Direct-To-COM compiler the following additional predefined procedures are available.

Function procedures

    VALID (v: VARPAR): BOOLEAN;    v (a VAR [nil] parameter) is not nil



Proper procedures

    NEW(VAR p: COM.IUnknown);    allocates p^

    NEW(VAR p: COM.IUnknown; outer: COM.IUnknown);    allocates p^ as subobject of outer

The two-argument version of NEW is used when implementing aggregation (see example *ComAggregate*) or to implement COM objects supporting several interfaces (see example *ComObject*). *AddRef*, *Release*, and *QueryInterface* calls are forwarded to the outer object. If *AddRef* or *Release* of *p* is called, then both the reference count of *p* and of the outer interface are incremented or decremented respectively.

<a id="3.6"></a>**Unions**

In order to simplify the mapping of C data structures, the Direct-To-COM compiler supports union records. In a union record all fields are aligned at offset 0. The size of a union record is the maximal size of its fields. The semantics of union records are the one of C union structures and not the one of Pascal-like variant records. With nested records and union records, Pascal-like variant records can be simulated also. A union record is marked with the *union* sysflag.

Example:

    TYPE

        Complex = RECORD

            type: SHORTINT;

            u: RECORD [union]

                cart: RECORD x, y: LONGREAL END;

                polar: RECORD phi, rho: LONGREAL END

            END

        END

Warning: pointers in union records are unsafe and are ignored by the garbage collector!

<a id="3.7"></a>**COM Programming Hints**

If you want to free an interface explicitly for some reason, just assign NIL to the interface pointer. This will call Release() on the pointer and additionally prevents you from using the (no longer valid) pointer accidentally.

The golden rule of COM programming in Component Pascal:

*    Never use an interface pointer if you can use a Component Pascal pointer instead.*

Although interface pointers are safely handled by the compiler, there are still unsafe constructs which must be handled carefully. This specifically applies to C pointers which are not interface pointers, like pointers to strings. The lifecycle of such pointers must be controlled by explicit *Allocate* and *Deallocate* calls. It also applies to structures containing C-style unions. Pointers in such structures (including interface pointers) must be handled manually. See the *ComTools* documentation for an example of how such a structure can be used safely.

<a id="3.8"></a>**Compiler Implementation Restrictions**

-    Untagged open arrays (arrays with unspecified length) containing interface pointers may not be assigned as a whole.

Workaround: Assign the individual elements.

-    The array supplied for an out parameter of type "untagged array of interface pointer" may not be open.

Workaround: Use an array with defined length.

-    Out parameters of type "untagged open array of pointer" are not initialized to NIL.

Workaround: Initialize them manually.

