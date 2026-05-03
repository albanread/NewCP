**Stores**

DEFINITION Stores;

    IMPORT Files;

    CONST

        alienVersion = 1; alienComponent = 2;

        inconsistentVersion = -1; inconsistentType = -2; moduleFileNotFound = -3;

        invalidModuleFile = -4; inconsModuleVersion = -5; typeNotFound = -6;

    TYPE

        TypeName = ARRAY 64 OF CHAR;

        TypePath = ARRAY 16 OF TypeName;

        OpName = ARRAY 32 OF CHAR;

        Domain = POINTER TO LIMITED RECORD

            PROCEDURE (d: Domain) GetSequencer (): ANYPTR, NEW;

            PROCEDURE (d: Domain) SetSequencer (sequencer: ANYPTR), NEW

        END;

        Operation = POINTER TO ABSTRACT RECORD

            PROCEDURE (op: Operation) Do, NEW, ABSTRACT

        END;

        Store = POINTER TO ABSTRACT RECORD

            PROCEDURE (s: Store) Domain (): Domain, NEW;

            PROCEDURE (s: Store) CopyFrom- (source: Store), NEW, EMPTY;

            PROCEDURE (s: Store) Internalize- (VAR rd: Reader), NEW, EMPTY;

            PROCEDURE (s: Store) Externalize- (VAR wr: Writer), NEW, EMPTY;

            PROCEDURE (s: Store) ExternalizeAs- (VAR s1: Store), NEW, EMPTY

        END;

        Reader = RECORD

            rider-: Files.Reader;

            cancelled-: BOOLEAN;

            readAlien-: BOOLEAN;

            (VAR rd: Reader) ConnectTo (f: Files.File), NEW;

            (VAR rd: Reader) Pos (): INTEGER, NEW;

            (VAR rd: Reader) SetPos (pos: INTEGER), NEW;

            (VAR rd: Reader) ReadBool (OUT x: BOOLEAN), NEW;

            (VAR rd: Reader) ReadSChar (OUT x: SHORTCHAR), NEW;

            (VAR rd: Reader) ReadXChar (OUT x: CHAR), NEW;

            (VAR rd: Reader) ReadChar (OUT x: CHAR), NEW;

            (VAR rd: Reader) ReadByte (OUT x: BYTE), NEW;

            (VAR rd: Reader) ReadSInt (OUT x: SHORTINT), NEW;

            (VAR rd: Reader) ReadXInt (OUT x: INTEGER), NEW;

            (VAR rd: Reader) ReadInt (OUT x: INTEGER), NEW;

            (VAR rd: Reader) ReadLong (OUT x: LONGINT), NEW;

            (VAR rd: Reader) ReadSReal (OUT x: SHORTREAL), NEW;

            (VAR rd: Reader) ReadXReal (OUT x: REAL), NEW;

            (VAR rd: Reader) ReadReal (OUT x: REAL), NEW;

            (VAR rd: Reader) ReadSet (OUT x: SET), NEW;

            (VAR rd: Reader) ReadSString (OUT x: ARRAY OF SHORTCHAR), NEW;

            (VAR rd: Reader) ReadXString (OUT x: ARRAY OF CHAR), NEW;

            (VAR rd: Reader) ReadString (OUT x: ARRAY OF CHAR), NEW;

            (VAR rd: Reader) ReadStore (OUT x: Store), NEW;

            (VAR rd: Reader) ReadVersion (min, max: INTEGER; OUT version: INTEGER), NEW;

            (VAR rd: Reader) TurnIntoAlien (cause: INTEGER), NEW

        END;

        Writer = RECORD

            rider-: Files.Writer;

            writtenStore-: Store;

            (VAR wr: Writer) ConnectTo (f: Files.File), NEW;

            (VAR wr: Writer) Pos (): INTEGER, NEW;

            (VAR wr: Writer) SetPos (pos: INTEGER), NEW;

            (VAR wr: Writer) WriteBool (x: BOOLEAN), NEW;

            (VAR wr: Writer) WriteSChar (x: SHORTCHAR), NEW;

            (VAR wr: Writer) WriteXChar (x: CHAR), NEW;

            (VAR wr: Writer) WriteChar (x: CHAR), NEW;

            (VAR wr: Writer) WriteByte (x: BYTE), NEW;

            (VAR wr: Writer) WriteSInt (x: SHORTINT), NEW;

            (VAR wr: Writer) WriteXInt (x: INTEGER), NEW;

            (VAR wr: Writer) WriteInt (x: INTEGER), NEW;

            (VAR wr: Writer) WriteLong (x: LONGINT), NEW;

            (VAR wr: Writer) WriteSReal (x: SHORTREAL), NEW;

            (VAR wr: Writer) WriteXReal (x: REAL), NEW;

            (VAR wr: Writer) WriteReal (x: REAL), NEW;

            (VAR wr: Writer) WriteSet (x: SET), NEW;

            (VAR wr: Writer) WriteSString (IN x: ARRAY OF SHORTCHAR), NEW;

            (VAR wr: Writer) WriteXString (IN x: ARRAY OF CHAR), NEW;

            (VAR wr: Writer) WriteString (IN x: ARRAY OF CHAR), NEW;

            (VAR wr: Writer) WriteStore (x: Store), NEW;

            (VAR wr: Writer) WriteVersion (version: INTEGER), NEW

        END;

        AlienComp = POINTER TO LIMITED RECORD

            next-: AlienComp

        END;

        AlienPiece = POINTER TO LIMITED RECORD (AlienComp)

            next-: AlienComp;

            pos-, len-: INTEGER

        END;

        AlienPart = POINTER TO LIMITED RECORD (AlienComp)

            next-: AlienComp;

            store-: Store

        END;

        Alien = POINTER TO LIMITED RECORD (Store)

            path-: TypePath;

            cause-: INTEGER;

            file-: Files.File;

            comps-: AlienComp

        END;

    PROCEDURE InitDomain (s: Store);

    PROCEDURE CopyOf (s: Store): Store;

    PROCEDURE Join (s0, s1: Store);

    PROCEDURE Joined (s0, s1: Store): BOOLEAN;

    PROCEDURE Unattached (s: Store): BOOLEAN;

    PROCEDURE ExternalizeProxy (s: Store): Store;

    PROCEDURE Report (IN msg, p0, p1, p2: ARRAY OF CHAR);

END Stores.

**Introduction**

Module *Stores* defines a data type *Store*, which should be used as base type of all storable extensible objects. When storing an object of an extensible type, it is necessary to store not only its contents but also its particular type. The type is needed to create an object of the correct type when reading the data in again.

A variable of (an extension of) type *Store* is stored in a file. A store must implement the *Internalize* procedure which takes a *Reader* as parameter, and the *Externalize* procedure which takes a *Writer* as parameter. These procedures read/write the store's persistent state. (A store may also have temporary state which is not internalized/externalized.) Readers and writers are mappers on files. The types *Views.View* and *TextModels.Model* are examples of *Store* extensions.

Stores may form arbitrary graphs. The "boundary" of a graph is determined by its *domain*. This is an object that represents the whole collection of stores that can be externalized to a file, or be internalized from a file. The stores of a domain are externalized and internalized such that pointers among them are reconstructed correctly upon internalization. In particular, *alias pointers* are handled correctly: if several stores point to another element of the same domain, this element is read in only once, and all the pointers to it are rebuilt. Arbitrary graphs can be handled this way, e.g., cyclic data structures. Links to stores of other domains are prohibited.

A document consists of a hierarchy of views and models, both of which are stores. Possibly there may occur further stores in a document, such as text attribute objects or controller objects. All stores of a document share the same domain.

The stores of a domain may be manipulated indirectly via *operation* objects. An operation implements a method *Do*, which must be auto-inverse (i.e., executing it twice undoes its effect). This is the key to the undo/redo mechanism of the compound document architecture of BlackBox. However, in principle it could also be used by a transaction mechanism to implement transactions on persistent non-document data represented as stores.

The contents of a store is stored in a file. When reading the same file by another BlackBox configuration, it may occur that not all necessary modules are available in this configuration, i.e., the module which defines the store's type cannot be loaded. Yet reading such an "alien" store does not fail completely. Instead of the correct store type, an "alien" object is generated. Obviously such an alien cannot interpret the data it represents, and therefore cannot provide any special behavior. However, it may be copied and stored into another file, such that its contents on the new file are intact and consistent.

To define the set of stores that belong to a domain and to set up an actual domain, two procedures have to be provided by the framework. A first procedure is needed to collect stores in sets and to join store sets, and a second procedure is used to assign an actual domain object to a set of stores. In BlackBox these procedures are *Join* and *InitDomain*. They are discussed in more detail below.

**Join**

Procedure *Join* is used to join two store sets. This procedure has to be called whenever a link from a store in one set is established to a store in the other set. As arguments, representatives of the two store sets have to be passed. *Join* is symmetric. If the two stores which are passed as arguments already belong to the same set, *Join* has no effect. Figure 1 demonstrates the effect of a *Join* operation on two store sets X and Y.

*Figure 1: Effect of a Join operation on two disjoint store sets X and Y.*

*Join* calls have to be added at those places where newly created stores are connected, usually in a factory procedure. All those stores have to be joined with a store *s* which are written to disk in the *Externalize* method of store *s*.

*Join* calls are not necessary in the *Internalize* and *CopyOf* methods where usually also links between stores are established. Module *Stores* has sufficient information to join these stores itself.

The function *Joined *allows to test whether two given stores belong to the same store set, i.e., whether they have been joined or not. *Joined *is symmetric, reflexive and transitive.

**InitDomain**

In order to assign an actual domain object to a free store or a store set, the procedure *InitDomain* is called. As parameter a member of the set (or a single store) is passed. The actual domain object is created inside module *Stores *(*Domain* is limited) and then assigned to all stores in the set*.* After a domain has been assigned to a store graph this domain can be accessed with the *Domain() *method defined in the *Store* type. If *InitDomain* is called on a store which is already assigned to a domain object, then the procedure has no effect. The framework calls *InitDomain* in module *Documents* only whenever a new document is created. Usually, there is no need for BlackBox programmers to call *InitDomain* themselves, except when a deep copy of a document is created.

The arguments of *Join* may or may not be bound to a domain. If only one argument store is bound to a domain, then the effect of *Join* is that all stores in the other set are bound to the same domain object. If both arguments are bound to a domain then they have to be bound to the same one and *Join* has no effect, otherwise a precondition trap is raised.

The function *Joined *can be applied on stores independent of whether they are bound to a domain or not. If store *s0* or *s1* is bound to a domain, then *Joined(s0, s1)* is equivalent to *s0.Domain() = s1.Domain()*.

The procedures *Join* and *InitDomain* specify the following pre and post conditions.

    PROCEDURE **Join **(s0, s1: Store)

        PRE 20    s0 # NIL

        PRE 21    s1 # NIL

        PRE 22    s0.Domain() = NIL OR s1.Domain() = NIL OR s0.Domain() = s1.Domain()

        POST    Joined(s0, s1)



    PROCEDURE** InitDomain **(s: Store)

        PRE 20    s # NIL

        POST    s.Domain() # NIL



If two stores have to be joined and precondition 22 is not met, then a deep copy of one of the two stores has to be generated using *CopyOf.*

**Alias pointers and copying**

The stores of a domain are externalized and internalized such that pointers among them are reconstructed correctly upon internalization. In particular, *alias pointers* are handled correctly: if several stores point to a particular element of the same domain, this element is read in only once, and all the pointers to it are rebuilt. Arbitrary graphs can be handled this way, including cyclic data structures.

Cloning of stores is provided by procedure *CopyOf*. This procedure creates a new object and then calls the *CopyFrom* method with the original object as parameter. This mechanism can conceptually be regarded as writing the store to a file and then reading it back as a clone.

Alias pointers are handled correctly. Whenever *CopyOf* is called, all copied stores are stored in a table which is associated with the store's set of joined stores. All recursive calls of *CopyOf* can access this information. After the top-level call of *CopyOf *terminates, the table with the copies is discarded.

However, there would be a problem with restoring alias pointers if during a call of *CopyOf* additional stores are added to the store set which contains the alias information. Therefore, *Join(s0, s1) *must not be called while a *CopyOf* operation is active either on *s0* or on *s1*. If such a situation appears, then an implementation trap is raised. This can be avoided by joining the involved stores before copying the store graph.

**Additional Remarks**

*Aliens*

Aliens are handled in a special way regarding domains. Since aliens are immutable, they are never copied and can be inserted into any store graph. As a store can only reference one domain, this would violate the rule that a store in a document may not refer to stores that belong to another domain. Therefore, all aliens are assigned a special domain for aliens. Module *Stores* knows this rule and does not trap if an alien is externalized.

*Operations and undo*

Calls of *Join* or *InitDomain* on stores are not operations, thus these calls cannot be undone! If stores are joined or assigned to a domain as a side effect of an operation and if the operation is undone, then the asignments which were performed on involved stores are not undone. This may require that a deep copy is generated before an operation is applied to a store graph.

*Shallow copies*

Module *Views* offers a procedure *CopyOf * which allows to make shallow copies of views. A shallow copy of a view means that the copy refers to the same model like the original. As models and views are always joined, this implies that a view and its shallow copy are also joined.

*Inspecting a store set*

A store set cannot be inspected programmatically beyond what is offered by procedure *Joined*. However, the debugger allows to iterate over all stores in a store set. The stores which belong to the same store set are linked in a circular list. As soon as a domain object is associated to the set, the links are set to *NIL* and each store in the set refers to the new domain object. To iterate over a store set the debugger has to be called before the store set is bound. Then, the stores can be iterated with the store's next field. Listing 2 shows a simple procedure which generates an empty text view and then opens the debugger. Following the *next* fields of the view *v*, you see that the following seven stores are joined in one set: *TextModels.Model*, *TextModels.Attributes*, *TextRulers.Ruler*, *TextRulers.Style*, *TextRulers.Attributes*, *TextViews.View* and *TextControllers.Controller*.

MODULE Test;

    IMPORT TextViews, TextModels;



    PROCEDURE **Do***;

        VAR v: TextViews.View;

    BEGIN

        v := TextViews.dir.New(TextModels.dir.New());

        HALT(0);

    END Do;



END Test.

*Listing 2: Inspecting the stores in a store set*

*Unattached*

Function *Unattached(s)* tests whether a store* s *has been joined to other stores or was bound to a domain. If neither is the case *Unattached *returns *TRUE*. This function is only provided by the framework to decide whether a single store can be simply joined to another store graph or whether a deep copy must be generated; e.g., for a store that is kept in a cache and which may be joined to different store graphs.

For this special purpose the following code pattern is used. In this example, store *s1* is copied if it is attached. This way an unnecessary copy call in the case that the store is unattached is prevented.

    IF ~Stores.Joined(s0, s1) THEN

        IF ~Stores.Unattached(s1) THEN s1 := Stores.CopyOf(s1) END;

        Stores.Join(s0, s1)

    END;

The pattern is not symmetric because it is often known statically that either s0 or s1 should never be copied. It is used for small stores which are generated once and which may be joined to different store sets.

**Data types**

*Stores* provides a pair of mapper types, which are used as parameters in a store's *Internalize* and *Externalize* procedures. These readers/writers use the following external (little endian) format:

BOOLEAN

    1 byte (0 = FALSE, 1 = TRUE)

SHORTCHAR

    1 byte in the Latin-1 character set (i.e., Unicode page 0; 00X..0FFX)

CHAR

    2 byte in the Unicode character set (0000X..0FFFFX)

BYTE

    1 byte (-128..127)

SHORTINT

    2 bytes (-32768..32767)

INTEGER

    4 bytes (-2147483648..2147483647)

LONGINT

    8 bytes (-9223372036854775808..9223372036854775807)

SHORTREAL

    4 bytes IEEE format

REAL

    8 bytes IEEE format

SET

    4 bytes (least significant bit = element 0)

Short String

    string in the Latin-1 character set, followed by a 00X

String

    string in the Unicode character set, followed by a 0000X

CONST **alienVersion**

This value is assigned to a *Reader*'s *cause* field if *Reader.ReadVersion* read a version outside of the specified range.

CONST **alienComponent**

This value can be used as *cause* parameter to *Reader.TurnIntoAlien* to indicate that the store itself could be read, but that some store contained in it is an alien. As an example, a view may turn itself into an alien if its model is an alien.

CONST **inconsistentVersion**

This value is assigned to a *Reader*'s *cause* field if *Reader.ReadVersion* read a data block which has an inconsistent length, i.e., not all of its data have been read, or it has been attempted to read beyond the end of the data.

CONST **inconsistentType**

This value is assigned to a *Reader*'s *cause* field if *Reader.ReadVersion* detected a change in the type extension hierarchy of the internalized type.

CONST **moduleFileNotFound**

This value is assigned to a *Reader*'s *cause* field if *Reader.ReadVersion* tried to load a module defining an internalized type, and the codefile for this module couldn't be found.

CONST **invalidModuleFile**

This value is assigned to a *Reader*'s *cause* field if *Reader.ReadVersion* tried to load a module defining an internalized type, and the module couldn't be loaded because it imports another module which cannot be loaded for some reason.

CONST **inconsModuleVersion**

This value is assigned to a *Reader*'s *cause* field if *Reader.ReadVersion* tried to load a module defining an internalized type, and the module couldn't be loaded because its version is inconsistent with some already loaded module.

CONST **typeNotFound**

This value is assigned to a *Reader*'s *cause* field if *Reader.ReadVersion* tried to internalize a non-existing type (the module was found, however).

TYPE **TypeName**

String type for the type name of an object.

TYPE **TypePath**

Array of type names.

TYPE **OpName**

String type for the name of an operation.

TYPE **Domain**

LIMITED

A domain represents a graph of stores which may be saved in a file as a whole. All stores of a domain refer to an object of type *Domain*.

PROCEDURE (d: Domain) GetSequencer (): ANYPTR

NEW

Used internally.

PROCEDURE (d: Domain) SetSequencer (sequencer: ANYPTR)

NEW

Used internally.

TYPE **Operation**

ABSTRACT

An operation is an object that represents a command performed on some store(s) of a domain. An operation can be undone (i.e., aborted) and redone.

PROCEDURE (op: Operation) **Do**

NEW, ABSTRACT

This method implements the actual behavior of the operation. It must be auto-inverse, i.e., if executed an even number of times, it must have no effect. If executed an odd number of times, it should have the same effect.

TYPE **Store**

ABSTRACT

Storable extensible data types like *Views.View* or *TextModels.Text* are derived from *Store*.

Stores are typically allocated by suitable directories, e.g., *Views.Directory* or *TextModels.Directory*.

Stores are used as base types for all objects that must be both extensible and persistent.

PROCEDURE (s: Store) **Domain** (): Domain

NEW

A store may be associated with a domain. This is done by the procedure *InitDomain*, which assigns a domain to the store.

*Domain* may be called by arbitrary clients.

PROCEDURE (s: Store) **CopyFrom**- (source: Store)

NEW, EMPTY

Copy the contents of *source* to *s*. Copying is a deep copy.

Pre

source # NIL    guaranteed

TYP(source) = TYP(s)    guaranteed

s.Domain() = NIL    guaranteed

s is not yet initialized    guaranteed

PROCEDURE (s: Store) **Internalize**- (VAR rd: Reader)

NEW, EMPTY

(For backward compatibility, this method is actually still EXTENSIBLE. This may change in the future.)

Reads the contents of *s* from reader *rd*. *Internalize* must read the same (amount of) data as is written by the corresponding *Externalize* procedure.

*Internalize* is called locally.

*Internalize* is extended by various persistent object types, e.g., models, views, and controllers.

Pre

source.Domain() = NIL    guaranteed

source is not yet initialized    guaranteed

PROCEDURE (s: Store) **Externalize**- (VAR wr: Writer)

NEW, EMPTY

(For backward compatibility, this method is actually still EXTENSIBLE. This may change in the future.)

Write the contents of *s* to writer *wr*. *Externalize* must write the same (amount of) data as is read by the corresponding *Internalize* procedure.

*Externalize* ist called locally.

*Externalize* is extended by various persistent object types, e.g., models, views, and controllers.

PROCEDURE (s: Store) **ExternalizeAs**- (VAR s1: Store)

NEW, EMPTY

Before a store's *Externalize* procedure is called, its *ExternalizeAs* procedure is called, which gives the store the opportunity to denote another store that should be externalized in its place (a "proxy"). It is also possible to set *s1* to *NIL*, which means that the store should not be externalized at all. This is used e.g. for compiler error markers, which are never stored.

*ExternalizeAs* ist called locally.

Pre

s1 = s    guaranteed

TYPE **Reader**

Reader for Component Pascal values like integers, reals, or sets. A reader contains a *Files.Reader*, to which it forwards most operations.

Readers are used in the *Store.Internalize* procedure.

Readers are not extensible.

**rider**-: Files.Reader

The file rider which links a *Reader* to a file.

**cancelled**-: BOOLEAN    valid during a *Store.Internalize* call

Tells whether the currently executing *Internalize* has been called by *ReadVersion* or *TurnIntoAlien*.

**readAlien**-: BOOLEAN

Tells whether any alien has been read since the last *ConnectTo*.

PROCEDURE (VAR rd: Reader) **ConnectTo** (f: Files.File)

NEW

Connect the reader to a file. All the following operations require connected readers, i.e., *rd.rider # NIL*. This precondition is not checked explicitly, however. After connecting, the reader's position is at the beginning of the file. If the same reader should be reused on another file, it must first be closed, by connecting it to *NIL*.

*ConnectTo* is used internally.

Pre

20    (f = NIL) OR (rd.rider = NIL)

Post

f = NIL

    rd.rider = NIL

f # NIL

    (rd.rider # NIL) & (rd.rider.Base() = f)

    rd.Pos() = 0

PROCEDURE (VAR rd: Reader) **Pos** (): INTEGER

NEW

Returns the reader's current position.

Post

0 <= result <= rd.rider.Base().Length()

PROCEDURE (VAR rd: Reader) **SetPos** (pos: INTEGER)

NEW

Sets the reader's current position to *pos*.

Pre

20    pos >= 0

21    pos <= rd.rider.Base().Length()

Post

rd.Pos() = pos

~rd.rider.eof

PROCEDURE (VAR rd: Reader) **ReadBool** (OUT x: BOOLEAN)

NEW

Reads a Boolean value.

PROCEDURE (VAR rd: Reader) **ReadSChar** (OUT x: SHORTCHAR)

NEW

Reads a short character (00X..0FFX).

PROCEDURE (VAR rd: Reader) **ReadXChar** (OUT x: CHAR)

NEW

Same as *ReadSChar*, but has a *CHAR*-type parameter.

This procedure is provided to simplify migration from Release 1.2 to 1.3.

PROCEDURE (VAR rd: Reader) **ReadChar** (OUT x: CHAR)

NEW

Reads a character (0000X..0FFFFX).

PROCEDURE (VAR rd: Reader) **ReadByte** (OUT x: BYTE)

NEW

Reads a very short integer (-128..127).

PROCEDURE (VAR rd: Reader) **ReadSInt** (OUT x: SHORTINT)

NEW

Reads a short integer (-32768..32767).

PROCEDURE (VAR rd: Reader) **ReadXInt** (OUT x: INTEGER)

NEW

Same as *ReadSInt*, but has an *INTEGER*-type parameter.

This procedure is provided to simplify migration from Release 1.2 to 1.3.

PROCEDURE (VAR rd: Reader) **ReadInt** (OUT x: INTEGER)

NEW

Reads an integer (-2147483648..2147483647).

PROCEDURE (VAR rd: Reader) **ReadLong** (OUT x: LONGINT)

NEW

Reads a long integer (-9223372036854775808..9223372036854775807).

PROCEDURE (VAR rd: Reader) **ReadSReal** (OUT x: SHORTREAL)

NEW

Reads a short real (32-bit IEEE number).

PROCEDURE (VAR rd: Reader) **ReadXReal** (OUT x: REAL)

NEW

Same as *ReadSReal*, but has a *REAL*-type parameter.

This procedure is provided to simplify migration from Release 1.2 to 1.3.

PROCEDURE (VAR rd: Reader) **ReadReal** (OUT x: REAL)

NEW

Reads a real (64-bit IEEE number).

PROCEDURE (VAR rd: Reader) **ReadSet** (OUT x: SET)

NEW

Reads a set (32 elements).

PROCEDURE (VAR rd: Reader) **ReadSString** (OUT x: ARRAY OF SHORTCHAR)

NEW

Reads a 0X-terminated short string.

Pre

invalid index     LEN(x) > Length(string)

PROCEDURE (VAR rd: Reader) **ReadXString** (OUT x: ARRAY OF CHAR)

NEW

Same as *ReadSString*, but has a string-type parameter.

This procedure is provided to simplify migration from Release 1.2 to 1.3.

PROCEDURE (VAR rd: Reader) **ReadString** (OUT x: ARRAY OF CHAR)

NEW

Reads a 0X-terminated string.

Pre

invalid index     LEN(x) > Length(string)

PROCEDURE (VAR rd: Reader) **ReadStore** (OUT x: Store)

NEW

Reads a store's type, allocates it, and then reads its contents, by calling the store's *Internalize* procedure. *x* may also be *NIL*, or an alien if the store's module cannot be loaded, or if internalization has been cancelled by the *Internalize* procedure.

If the store has already been read in, a pointer to the same store is returned instead of allocating a new one. This means that arbitrary graphs that have been written with *WriteStore* are reconstructed correctly, including alias pointers to the same store, cycles, etc.

If the file on which the reader operates does not contain correct input, then an assertion trap will be caused (traps 101 to trap 106).

Pre

20    the reader is at the start position of a new store

Post

empty store on file

    x = NIL

non-empty store on file

    x # NIL

        x IS Alien

            x.cause # 0

            x.type # ""

            x.file # NIL

            x.pos >= 0    beginning of store's data

            x.len >= 0    length of store's data

            alien store contents are on x.file in the range [x.pos .. x.pos + x.len[.

            These data include only the store's contents, not its prefix

        ~(x IS Alien)

            x was read successfully

PROCEDURE (VAR rd: Reader) **ReadVersion** (min, max: INTEGER; OUT version: INTEGER)

NEW

Read a version byte and return it in *version*. If *version* is not in the specified range *[min .. max]*, the store currently being read is turned into an alien, with *cause = alienVersion*.

Pre

20    0 <= min <= max

Post

min <= version <= max

    legal version

(version < min) OR (version > max)

    illegal version

    rd.cause = alienVersion

    rd.cancelled

    rd.readAlien

PROCEDURE (VAR rd: Reader) **TurnIntoAlien** (cause: INTEGER)

NEW

A store which is currently being internalized can turn itself into an alien, e.g., if it has read a component store which is an alien.

Pre

20    cause > 0

TYPE **Writer**

Writer for Component Pascal values like integers, reals, or sets. A writer contains a *Files.Writer*, to which it forwards most operations.

Writers are used in the *Externalize* procedure.

Writers are not extensible.

**rider**-: Files.Writer

A file rider which links a *Writer* to a file.

**writtenStore**-: Store

Store which was most recently written as an effect of a call to *WriteStore*.

PROCEDURE (VAR wr: Writer) **ConnectTo** (f: Files.File)

NEW

Connect the writer to a file. All the following operations require connected writers, i.e., *wr.rider # NIL*. This precondition is not checked explicitly, however. After connecting, the writer's position is at the end of the file. If the same writer should be reused on another file, it must first be closed, by connecting it to *NIL*.

*ConnectTo* is used internally.

Pre

20    (f = NIL) OR (wr.rider = NIL)

Post

f = NIL

    wr.rider = NIL

f # NIL

    wr.rider # NIL  &  wr.rider.Base() = f

    wr.Pos() = wr.rider.Base().Length()

PROCEDURE (VAR wr: Writer) **Pos** (): INTEGER

NEW

Returns the writer's current position.

Post

0 <= result <= wr.rider.Base().Length()

PROCEDURE (VAR wr: Writer) **SetPos** (pos: INTEGER)

NEW

Sets the writer's current position to *pos*.

Pre

20    pos >= 0

21    pos <= wr.rider.Base().Length()

Post

wr.Pos() = pos

PROCEDURE (VAR wr: Writer) **WriteBool** (x: BOOLEAN)

NEW

Writes a Boolean value.

PROCEDURE (VAR wr: Writer) **WriteSChar** (x: SHORTCHAR)

NEW

Writes a character (00X..0FFX).

PROCEDURE (VAR wr: Writer) **WriteXChar** (x: CHAR)

NEW

Same as *WriteSChar*, but has a *CHAR*-type parameter.

This procedure is provided to simplify migration from Release 1.2 to 1.3.

PROCEDURE (VAR wr: Writer) **WriteChar** (x: CHAR)

NEW

Writes a character (0000X..0FFFFX).

PROCEDURE (VAR wr: Writer) **WriteByte** (x: BYTE)

NEW

Writes a very short integer (-128..127).

PROCEDURE (VAR wr: Writer) **WriteSInt** (x: SHORTINT)

NEW

Writes a short integer (-32768..32767).

PROCEDURE (VAR wr: Writer) **WriteXInt** (x: INTEGER)

NEW

Same as *WriteSInt*, but has an *INTEGER*-type parameter.

This procedure is provided to simplify migration from Release 1.2 to 1.3.

PROCEDURE (VAR wr: Writer) **WriteInt** (x: INTEGER)

NEW

Writes an integer (-2147483648..2147483647).

PROCEDURE (VAR wr: Writer) **WriteLong** (x: LONGINT)

NEW

Writes a long integer (-9223372036854775808..9223372036854775807).

PROCEDURE (VAR wr: Writer) **WriteSReal** (x: SHORTREAL)

NEW

Writes a real (32-bit IEEE number).

PROCEDURE (VAR wr: Writer) **WriteXReal** (x: REAL)

NEW

Same as *WriteSReal*, but has a *REAL*-type parameter.

This procedure is provided to simplify migration from Release 1.2 to 1.3.

PROCEDURE (VAR wr: Writer) **WriteReal** (x: REAL)

NEW

Writes a long real (64-bit IEEE number).

PROCEDURE (VAR wr: Writer) **WriteSet** (x: SET)

NEW

Writes a set (32 elements).

PROCEDURE (VAR wr: Writer) **WriteSString** (IN x: ARRAY OF SHORTCHAR)

NEW

Writes a 0X-terminated short string.

PROCEDURE (VAR wr: Writer) **WriteXString** (IN x: ARRAY OF CHAR)

NEW

Same as *WriteSString*, but has a string-type parameter.

This procedure is provided to simplify migration from Release 1.2 to 1.3.

PROCEDURE (VAR wr: Writer) **WriteString** (IN x: ARRAY OF CHAR)

NEW

Writes a 0X-terminated string.

PROCEDURE (VAR wr: Writer) **WriteStore** (x: Store)

NEW

Writes the store's type and then its contents, by calling the store's *Externalize* procedure. *x* may also be *NIL*, or an alien. Before *Externalize*, *ExternalizeAs* is called in order to give the store the opportunity to denote a proxy which should be stored in its stead. *WriteStore* writes *x* and (via *Externalize*) all the stores that it contains. Cycles are handled correctly, i.e., a store is only written once, even if referenced several times in a complex graph.

All stores that are written using the same writer must have the identical domain.

Pre

20    wr.rider # NIL

21    x # NIL  =>  writer must have no domain, or the same one as x (and as all previously written stores)

PROCEDURE **WriteVersion** (version: INTEGER)

NEW

Writes a version byte.

Pre

20    0 <= version <= 127

TYPE **AlienComp, AlienPiece, AlienPart, Alien**

LIMITED

These auxiliary types are used internally, to handle alien stores.

PROCEDURE **Join** (s0, s1: Store)

Join two stores in the same store set. See the explanation at the beginning of this text.

Pre

20    s0 # NIL

21    s1 # NIL

22    s0.Domain() = NIL  OR  s1.Domain() = NIL  OR  s0.Domain() = s1.Domain()

Post

Joined(s0, s1)

PROCEDURE **Joined** (s0, s1: Store): BOOLEAN

Test whether two stores are joined. *Joined(x, x)* always returns *TRUE*, i.e., it is reflexive.

Pre

20    s0 # NIL

21    s1 # NIL

PROCEDURE **Unattached** (s: Store): BOOLEAN

Tests whether *s* is not attached to a domain *and* whether it has not been joined with another store.

Rarely used.

Pre

20    s # NIL

PROCEDURE **CopyOf** (s: Store): Store

Returns the clone of a store, with its contents copied.

*CopyOf* allocates a new record with the same *dynamic* type as *s*, and initializes it by calling its *CopyFrom* procedure. The copy is a deep copy.

Pre

20    s # NIL

Post

(result # NIL) & (result # s)

TYP(result) = TYP(s)

PROCEDURE **InitDomain** (s: Store)

Initializes the domain of store *s*. See the explanation at the beginning of this text.

Pre

20    s # NIL

Post

s.Domain() # NIL

PROCEDURE **ExternalizeProxy** (s: Store): Store

Causes *s* to call its *ExternalizeAs* method, basically doing the following:

    IF s # NIL THEN s.ExternalizeAs(s) END;

    RETURN s

PROCEDURE **Report** (IN msg, p0, p1, p2: ARRAY OF CHAR)

When a store encounters a problem during internalization, it can report the problem by calling this procedure. The parameters are similar to *Dialog.ShowParamMsg*.

