**Services**

DEFINITION Services;

    CONST

        immediately = -1; now = 0;

        resolution = 1000;

    TYPE

        Action = POINTER TO ABSTRACT RECORD

            (a: Action) Do-, NEW, ABSTRACT

        END;

    PROCEDURE DoLater (a: Action; notBefore: LONGINT);

    PROCEDURE RemoveAction (a: Action);

    PROCEDURE Ticks (): LONGINT;

    PROCEDURE GetTypeName (IN rec: ANYREC; OUT type: ARRAY OF CHAR);

    PROCEDURE SameType (IN ra, rb: ANYREC): BOOLEAN;

    PROCEDURE IsExtensionOf (IN ra, rb: ANYREC): BOOLEAN;

    PROCEDURE Is (IN rec: ANYREC; IN type: ARRAY OF CHAR): BOOLEAN;

    PROCEDURE Extends (IN type, base: ARRAY OF CHAR): BOOLEAN;

    PROCEDURE Level (IN type: ARRAY OF CHAR): INTEGER;

    PROCEDURE TypeLevel (IN rec: ANYREC): INTEGER;

    PROCEDURE AdrOf (IN rec: ANYREC): INTEGER;

    PROCEDURE Collect;

END Services.

This module provides a variety of low-level services. Currently, only a timing service, a background processing service, and a reflection service is provided.

*Actions* are objects whose *Do* procedures are executed in a delayed fashion, when the system is idle. An action which re-installs itself whenever it is invoked operates as a non-preemptive background task.

The reflection service, unlike module *Meta*, only works on records, and it is also applicable to temporary variables on the stack (while *Meta* items typically denote long-lived variables).

Examples:

[<u>ObxActions  docu</u>](../../Obx/Docu/Actions.odc.md)

[<u>ObxCubes  docu</u>](../../Obx/Docu/Cubes.odc.md)

CONST **immediately**

This value can be passed to *DoLater* as *notBefore* parameter, if the action must be executed as part of the currently executing command.

CONST **now**

This value can be passed to *DoLater* as *notBefore* parameter, if the action should be executed as quickly as possible, after the currently executing command.

CONST **resolution**

Time resolution in ticks per second. The current time can be inquired by procedure *Ticks*.

TYPE **Action**

ABSTRACT

Actions are objects whose *Do* procedures are called in a deferred way, when the system is idle again.

PROCEDURE (a: Action) **Do-**

NEW, ABSTRACT

For a registered action *a*, *a.Do* is called eventually, depending how it has been registered (see *DoLater*).

PROCEDURE **DoLater** (a: Action; notBefore: LONGINT)

Register an action. Its *Do* procedure will be executed once, later when time permits. *a.Do* is called eventually after *Ticks()* has reached *notBefore*, or if *notBefore = now*. If the action's Do *procedure* should be executed more than once, it may call *DoLater* to re-install itself. A particular action can only be registered once; additional attempts have no effect.

Pre

a # NIL    20

PROCEDURE **RemoveAction** (a: Action)

Remove a registered action. If action *a* is not registered or *NIL*, nothing happens.

PROCEDURE **Ticks** (): LONGINT

Returns the current time in clock ticks. These ticks have a resolution of *resolution*., i.e., *resolution* ticks per second.

PROCEDURE **GetTypeName** (IN rec: ANYREC; OUT type: ARRAY OF CHAR)

Returns the name of a record type, in the form "module.type". If a pointer is passed, its record type is returned. If the record is anonymous, the pointer's type name is returned, e.g., "FormViews.StdView".

PROCEDURE **SameType** (IN ra, rb: ANYREC): BOOLEAN

Determines whether two record variables have (exactly!) the same type.

PROCEDURE **IsExtensionOf** (IN ra, rb: ANYREC): BOOLEAN

Determines whether the record type of *ra* is the same or an extension of the type of *rb*.

PROCEDURE **Is** (IN rec: ANYREC; IN type: ARRAY OF CHAR): BOOLEAN

Determines whether the record type of *rec* is the same or an extension of the type given in *type* (e.g., as "FormViews.View"). If type *type* was not found, *FALSE* is returned.

Pre

type # ""    20

type has form "module.type"    21

PROCEDURE **Extends** (IN type, base: ARRAY OF CHAR): BOOLEAN

Determines whether the record type *type* is the same or an extension of the type *base*. If type *type* or *base* was not found, *FALSE* is returned.

Pre

type # ""  &  base # ""    20

type and base have form "module.type"    21

PROCEDURE **Level** (IN type: ARRAY OF CHAR): INTEGER

Determines the extension level of a record type *type*. A newly introduced type has level 0. If type *type* was not found, *-1* is returned.

Pre

type # ""    20

type has form "module.type"    21

PROCEDURE **TypeLevel** (IN rec: ANYREC): INTEGER

Determines the extension level of a record variable *rec*'s type. A newly introduced type has level 0.

PROCEDURE **AdrOf** (IN rec: ANYREC): INTEGER

Returns the address of a record variable.

PROCEDURE **Collect**

Forces a garbage collection.

