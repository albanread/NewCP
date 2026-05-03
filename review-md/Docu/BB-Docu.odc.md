**Documentation Conventions**

Documentation has two major purposes: to communicate how a program's service can be used interactively (user manual) and how the program's service can be used programmatically (developer manual). In BlackBox, services are arranged in subsystems. A subsystem typically consists of several possibly cooperating modules. In the general case, we need three kinds of documentation texts: a user manual, one module interface description per module, and an overview over how the modules may interact with each other, i.e., an introduction to the overall architecture of the subsystem (developer manual).

A subsystem is represented on the disk as a directory, e.g., directory *Text*. In this directory, there is a nested directory called *Docu*, e.g. *Text/Docu*. This documentation directory may contain the following text documents:

ꀢ a user manual called *User-Man*

ꀢ a hypertext map to the documentation, called *Sys-Map*

ꀢ a programmer's introduction called *Dev-Man*

ꀢ and one module interface description per module, having the module  name without its subsystem prefix, e.g. *Views* for module *TextViews*.

Note that all documentation texts which do not directly denote the programmer's reference for a particular module must have a dash ("-") in their names, so that the repository browser can distinguish between module documentations and other documentation files.

The remainder of this text describes the format of a module interface description.

Each module description begins with a module definition. The Component Pascal notation for module definitions is used, which lists all exported items of a module, and only those. A module begins with the keyword *DEFINITION* instead of *MODULE*, export marks are omitted (except read-only marks), and procedure bodies and the module body are left out as well. A method appears in the corresponding record declaration itself, without the *PROCEDURE* keyword.

The hypothetical module "Dictionaries" serves as an example:

DEFINITION Dictionaries;

    CONST maxElems = 32;

    TYPE

        Dictionary = POINTER TO LIMITED RECORD

            elems-: INTEGER;

            (d: Dictionary) Put (string: ARRAY OF CHAR; OUT key: INTEGER), NEW;

            (d: Dictionary) Get (key: INTEGER; OUT string: ARRAY OF CHAR), NEW

        END;

    VAR someDict: Dictionary;

    PROCEDURE Init;

END Dictionaries.

*In an extended record, the inherited procedures are not listed, only the new procedures, or the procedures which have changed signatures or where the semantics are specialized so that they need to be clarified. *

*A changed signature may be caused by a covariant function result, i.e., the procedure returns a pointer that is an extension of the one returned by the base procedure. Or it may be caused by changed method attributes (final, ABSTRACT, EMPTY, EXTENSIBLE), e.g., an abstract method may have been implemented and made final.*

*A module definition is followed by a brief description of the abstraction(s) provided by the module.*

This module provides a concrete type Dictionary for the manipulation of string dictionaries.

*Then the constants of the module are listed, without their values:*

CONST **maxElems**

Maximum number of elements in a dictionary.

*This is followed by the module's types:*

TYPE **Dictionary**

LIMITED

Manages a dictionary of strings, with an integer number as key.

*Record attributes are given in the  second line, as above. Record attributes are either none (final by default), EXTENSIBLE, ABSTRACT, or LIMITED.*

*If the type is an extension of another type, this is marked, e.g., as*

*Dictionary (OtherModule.Dictionary)*

*Then the record fields are listed:*

**elems**-: INTEGER    0 <= elems <= maxElems

The number of elements currently in the dictionary.

*As in the above example, an invariant over the record field's value may optionally be given.*

*Then the procedures bound to this type are given:*

PROCEDURE (d: Dictionary) **Put** (string: ARRAY OF CHAR; OUT key: INTEGER)

NEW

*For each method, it is specified whether this procedure is final (default), extensible, abstract, or empty.*

*An abstract procedure must not be called.*

*An empty procedure must not be called by an extending procedure (i.e., through a super-call of an implementing procedure).*

*In the second line, the method attributes are given, which are either none (final by default), EXTENSIBLE, ABSTRACT, or EMPTY. Before this attribute, NEW is written for newly introduced methods.*

*An implement-only method can only be called in its defining module. This module can thus establish the method's preconditions before every call to it that occurs in the module implementation. If the implementation is correct, these preconditions can never be violated, since no other module may perform a call. Thus an implementor can assume that the preconditions hold, and need not test them. Preconditions of implement-only methods are thus not attributed with a trap number; instead, they are marked as "guaranteed", meaning that the defining module guarantees them.*

*An explanation of the procedure's behavior follows:*

Put a string into dictionary *d*, and receive *key* in return. If the string is already in the dictionary, the same key is returned as when it was inserted first. If the string is not in the dictionary, it is inserted and a value which has not been used before is returned.

*After an explanation in plain English, some preconditions for this procedure may be specified. These preconditions are given in a semi-formal notation:*

Pre

20    string # ""

*This means that if the precondition is violated (i.e. if string = ""), the currently executing command is aborted with exception number 20. Instead of a number, the following exceptions may be specified as well:*

* invalid index*

* type guard check*

*A second precondition may thus be the following:*

invalid index    string in d  OR  d.elems < maxElems

*In older parts of the documentation, the precondition numbers may also be given after the expression, instead of in front of it. Over time, all documentation should be adapted to the new and better readable style.*

*After the preconditions, some postconditions may be specified:*

Post

string in d'

    old key returned

~(string in d')

    string in d

    d.elems = d.elems' + 1

    new key returned

*This postcondition contains two cases, namely what happens if the string is already contained in d, and what happens if the string is not yet contained in d. The conditions are textually aligned to the left, while the respective conclusions are indented. Occasionally, conditions are nested by further indentations.*

*To refer to values before the operation, an expression is followed by an apostrophe if this is necessary for clarity (i.e. if the value may be changed at all). Thus the expression*

*    d.elems = d.elems' + 1*

*means that the value of d.elems has been incremented by one.*

*Component Pascal is generally used as syntax for such expressions, although some liberties are taken, e.g., simple auxiliary procedures that are not described further may be used.*

*Function results are generically called "result".*

*The amount of formalism is kept small. It is attempted to limit formal specifications to few and simple conditions, or to the explanation of particularly subtle situations. It is not intended to specify complete formal semantics of the library, thus the formalism does not replace plain text explanations, examples, or graphical illustrations completely.*

*However, it is felt that checked preconditions in particular are often helpful even in this incomplete and semi-formal way, both for documentation and for debugging purposes.*

PROCEDURE (d: Dictionary) **Get** (key: INTEGER; OUT string: ARRAY OF CHAR)

NEW

Retrieve a string from the dictionary, by using *key*. If the value is not found, *string* is set to the empty string.

Post

key in d

    return corresponding string

~(key in d)

    return ""

*Procedures which are inherited through type extension are usually not listed here. Exceptions are procedures which have extended their semantics. Semantic extensions are restricted to having weaker preconditions or stronger postconditions.*

*After all types, the global variables are listed:*

VAR **someDict**: Dictionary    someDict # NIL

This variable contains a dictionary.

*The optional invariant is established by the module body.*

*Finally, the procedures exported by the module are listed:*

PROCEDURE **Init** (d: Dictionary)

Initializes a dictionary. Any dictionary may be allocated once only.

Pre

20    d not yet initialized

*In BlackBox, all procedures whose names start with "Init" may be called only once per object, i.e., they are "snappy".*

