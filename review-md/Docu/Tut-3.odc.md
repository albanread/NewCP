**3 BlackBox Design Practices**

In the previous chapters, various patterns and design approaches have been described. They help to solve problems that occur in interactive compound document applications. In this chapter, more general aspects are discussed which help to make a framework component-oriented, i.e., extensible through dynamically loaded black-box components.

We don't attempt to create a full design method that gives step-by-step recipes for how to design a new framework. This would not be realistic. But we want to demonstrate and motivate the practices, approaches, and patterns that have been followed in the design of the BlackBox Component Framework. Some of the design practices have even led to the incorporation of specific framework-related features into the language Component Pascal. Language support for framework design is discussed first, followed by the way Component Pascal components are managed, and the rules which govern many component collaborations in BlackBox.

**3.1 Language support**

A framework embodies a collection of design patterns, cast into the notation of a particular programming language. Thus the expressiveness of the language, in particular of its interface definition subset, has a major impact on how much of the framework's design can be captured directly in code. Capturing architectural decisions and design patterns explicitly in the language is important for making refactoring of a framework less risky. Refactoring of a framework and its extension components allows to prevent that old architectures grow into brittle structures that threaten to collapse under their own weight. Preventing architecture degradation is the key to keeping software systems productive over a longer period of time, especially if the software consists of components that are replaced, added, or removed incrementally over time.

A framework represents an architecture for solving a certain class of problems. A good architecture uses a minimal number of design patterns wherever they are applicable. Consistent use of design patterns make the framework easier to document and easier to comprehend.

Design patterns are abstract solutions to a problem. For a framework, they need to be formulated in terms of a concrete programming language. The programming language has a large influence on how well the design pattern can be represented and how well consistency with the pattern can be maintained.

There are two major "philosophies" of programming language design. One of them is exemplified by the language C. C is a terse systems programming language which allows to easily and efficiently manipulate memory data structures. The other approach is exemplified by Pascal. Pascal is a readable application programming language which provides a high degree of safety.

The C approach is particularly adequate for writing low-level code such as device drivers. When C was increasingly being used for applications as well, a *zero errors ideology* formed, which says that because programming errors must not occur, they will not occur. A "real programmer" doesn't make mistakes, and therefore doesn't need any kind of protection facilities forced upon him by the language. Safety features such as index overflow checks are for beginners only; for professionals they are merely a handicap.

However, modern psychological research has clearly shown that humans make mistakes all the time, whether they write programs, develop mathematical proofs, fly airplanes, or perform operations on a patient. Mistakes are made whether or not they are admitted. The important insight is that most mistakes can be corrected easily if they are detected early on. Detection works best in an openly communicating team, where a team member is not afraid of others double-checking his or her work (and thereby exposing the mistakes). Good airlines let their pilots train such cooperative behavior under stress. Some very forward-thinking clinics use a similar training for surgeons and their aides. They have overcome the zero errors ideology in their fields.

In the world of programming, a reverse trend has shaped the industry in the last ten years, by making C the language of choice for all kinds of programs. In terms of software engineering and safety consciousness, this was a huge step backwards behind the state-of-the-art. Large companies tried to limit the damage by imposing the use of tools that reintroduce at least a modest level of safety - instead of solving the problems where it costs least, namely at the language level.

But while it is difficult for a programmer to admit that he makes mistakes, it is much easier for him to acknowledge that *other* programmers make mistakes. This became relevant with the Internet. The Internet makes it easy and sometimes even automatic to download and execute small programs ("applets") from unknown sites. This code clearly cannot be trusted in general. Foor good reason, this has scared large corporations enough to look at safer languages again. In fact, safety concerns were the reason why the language Java was created in the first place. Superficially, Java looks similar to C++; but unlike C and C++, it is completely typesafe, like Component Pascal.

In Component Pascal, objects and their classes (record types) may be hidden completely in a module, or they may be wholly exported, i.e., they and their parts (record fields / instance variables) may be made completely visible outside of the defining module. In practice, it is useful to have even more control. For this reason, Component Pascal allows to determine for each record field whether it is fully exported, read-only exported, or hidden. The following example (Figure 3-1) shows how the asterisk ("*") is used for export, and the dash ("-") is used for the more restricted read-only export:

MODULE ObxSample;

    TYPE

        File* = POINTER TO RECORD

            len: INTEGER    *(* hidden instance variable *)*

        END;

        Rider* = POINTER TO RECORD

            *(* there may be several riders on one file *)*

            file-: File;    *(* read-only instance variable *)*

            eof*: BOOLEAN;    *(* fully exported instance variable *)*

            pos: INTEGER    *(* hidden instance variable *)*

                *(* Invariant: (pos >= 0) & (pos < file.len) *)*

        END;

    PROCEDURE (f: File) GetLength* (OUT length: INTEGER), NEW;

    BEGIN

        length := f.len

    END GetLength;

    PROCEDURE (rd: Rider) SetPos* (pos: INTEGER), NEW;

    BEGIN

        *(* assert invariants, so that errors may not be propagated across components *)*

*        *ASSERT(pos >= 0); ASSERT(pos < rd.file.len);

        rd.pos := pos

    END SetPos;

    ...

END ObxSample.

Listing 3-1. Sample module in Component Pascal

Java does not support read-only export, but it does support protected fields, which are fields that are only visible to extensions of a class, but not to normal clients. This feature is only relevant if implementation inheritance is used across module boundaries. For reasons that go beyond the scope of this text, implementation inheritance is not a good idea across black-box abstractions, such as components, and thus should not normally be used across component boundaries or in component framework interfaces. Protected export is thus not supported by Component Pascal.

Safety means different things to different people. On the one hand, C++ can be considered safer than plain C. On the other hand, the Internet also raises concerns that go beyond mere safety; security has to be considered as well (i.e., unauthorized access, criminal attacks, and so on). Security issues are a matter of authorization and authorization checking and are generally dealt with at the operating system or hardware level. Higher-level security features are a matter of library definition and implementation. However, overheads incurred by the excessive crossing of hardware protection boundaries can be avoided if it is possible to build on strong safety properties of the used language(s), which leads us back to the question of safety. But what exactly is "safety"?

In short, "safety" of a programming language means that its definition allows to specify invariants, and that its implementation guarantees that these invariants are kept. (The availability of a trusted implementation must be assumed; this trust is usually earned by successfully surviving attacks.) In short: *safety is "invariants taken seriously"*.

What kinds of invariants are we talking about? The most fundamental invariants are the memory invariants: memory occupied by a variable is only used in the way that the language (e.g., a type declaration) allows. In practice, this means that arbitrary type casting must be ruled out and that manual deallocation of dynamic data structures is not permitted anymore, because deallocation could happen too early, and some still used memory could be reallocated and thus being used simultaneously by two unrelated variables. These dangling pointers are usually desastrous, but can be avoided completely if garbage collection is used instead of manual deallocation. Consequently, Java and Component Pascal are garbage-collected.

Memory invariants are fundamental. More application-specific invariants are typically established over several cooperating objects. For example, an object which represents a file access path must always have a current position which lies within the length of the file, where the file is represented by a second object. This invariant spans two objects (and thus classes). It can only be guaranteed if the programming language allows to define interactions between the two objects that are private to them, so that no outside code may interfere.

Note that this is different from the "protected" relation discussed above. Rather than covering the relation between a base class and its yet unknown subclasses, the issue here is classes that have been co-designed and will always be used in conjunction. A module or package construct is a suitable structured means to allow the definition of such higher-order safety properties. Java packages are not ideal in this respect. Since Java packages are *open modules*, new classes can be added to a package at run-time, and may well violate the invariants that have been established earlier. Such addition of "foreign" classes to a package thus needs to be prevented by run-time management facilities that are beyond the control of the language definition. In Java, this is related to the concept of unique ownership: every package is conceptually owned by its source and only that single source should have authority to add new classes to a package. In any case, the Java package construct is an improvement over most other object-oriented languages, for example the entirely unstructured approach of C++'s friend classes, or the complete absence of a suitable construct in standard Smalltalk.

In contrast to Java packages, Component Pascal supports *closed modules*, or modules for short. In Component Pascal, a module is the appropriate unit of compilation, loading, and information hiding.

Applying strong typing and information hiding makes it easier to catch mistakes as early as possible, when they are still easy and inexpensive to correct. This is valuable because it helps increase the program's robustness.

But the benefits of type and module safety go even further. They also provide more *flexibility*. This is surprising at first sight. Why should restrictions such as types (which restrict the operations on variables) and modules (which restrict visibility) create anything else than *reduced* flexibility? The reason for this so-called *refactoring paradoxon* is two-fold. On the one hand, everything that is completely hidden in a module may be changed only by considering this one module. Local changes in the hidden part of the module don't reverberate beyond the module itself. Not even recompilation of other client modules is necessary. On the other hand, when a well-typed interface for some reason *is* changed, then mere recompilation of the clients will detect the interface usages that have become inconsistent, e.g., after a parameter's type or a method name was changed. A compiler thus can actually increase the confidence in a software system which has been "refactored" due to some interface changes.

By checking typed interfaces at a carefully chosen level of granularity, a balance can be struck between release-to-release binary compatibility and detection of definite inconsistencies. Component Pascal supports an expressive type system to allow the detection of such inconsistencies. For example, newly introduced methods must be marked as *NEW*, and *VAR* parameters can be specialized to *IN* or *OUT* parameters. In Java, for example, it is not possible to distinguish between a new method, an overloading attempt, and an overriding attempt. By misspelling a method name, by changing a base or a subclass, or by combining incompatible versions, this ambiguity can lead to errors that are hard to track.

A framework typically predefines interfaces for extensions of the framework. The framework may even contain code that uses these interfaces, although no implementation exists yet. The calling of code that resides "higher up" in the module hierarchy is typical for object-oriented frameworks. A language can support this typical framework control flow pattern by providing some form of interfaces, which are abstract classes to be implemented elsewhere. Component Pascal supports abstract record types with single inheritance, in a way that a complete spectrum between fully abstract and fully concrete types are possible. Java is similar to Component Pascal in this respect, except that it additionally supports a separate interface construct. A Java interface is the same thing as a fully abstract class, except that it allows multiple (interface) inheritance.

MODULE TestViews;

    TYPE

        View* = POINTER TO ABSTRACT RECORD

            (* partially abstract type *)

            context-: Context;

            ... some hidden fields ...

        END;

        Context* = POINTER TO ABSTRACT RECORD END;

        (* fully abstract type *)

        PROCEDURE (v: View) Restore* (l, t, r, b: INTEGER), NEW, ABSTRACT;

        ...

END TestViews.

Listing 3-2. Semi-abstract and abstract record types in Component Pascal

Invariants are properties that supposedly stay invariant. Thus it is questionable whether even a subtype (subclass / extended type) should be allowed to arbitrarily modify the behavior of its basetype. In object-oriented languages, such modifications can be achieved by overriding inherited methods. Java allows to make classes or methods *final*, to give the framework designer the possibility to prevent any kind of invariant violation through the back door of overriding. Component Pascal record types and methods are final by default, they can be marked as extensible explicitly.

Component Pascal goes beyond Java with several other language constructs. One are *limited* records. Those are records that may be extended and allocated only within their defining module. From the perspective of importing modules, limited types are final and not even allocatable. This makes it possible to guarantee that all allocation occurs centrally in the defining (framework) module, which gives this module full control over initialization. For example, it may provide factory functions that allocate an object and initialize it in different ways, establishing invariants before the objects are passed to client modules. This is more flexible and simpler than constructors as used in Java.

Implement-only export is a prime example of a feature motivated by typical framework design patterns. A record type's methods may be exported as implement-only, by using a dash instead of an asterisk. An implement-only method can only be called inside the defining module. But it can be *implemented* outside the defining module, in an implementation component of the framework. For example, the BlackBox Component Framework's store mechanism uses this feature to protect a store's *Internalize* and *Externalize* methods from being called out of their correct context. Basically, the framework (in this case the *Stores* module) uses the implement-only methods in all possible legal ways (i.e., implements all legal kinds of use-cases), and only exports them for implementation purposes.

For methods that represent optional interfaces, the method attribute *EMPTY* is supported. An empty method is a fully abstract hook that can be implemented in an extension, but unlike abstract methods it need not be implemented. For example, a view has an empty *HandleCtrlMsg* method which need only be implemented by views that react on user input, e.g., via mouse or keyboard.

In the *Design Patterns* book of Gamma et. al. ["Design Patterns, Elements of Reusable Object-Oriented Software"; Erich Gamma, Richard Helm, Ralph Johnson, John Vlissides; Addison-Wesley, 1994; ISBN 0-201-63361-2], a list of common design problems is identified. These problems often lead to unnecessary redesigns, because the software doesn't allow for a sufficient degree of change. Several of these problems are addressed by Component Pascal:

ꀢ Creating an object by specifying a class explicity

Modules allow to hide classes. A hidden class cannot be instantiated directly by a client module, since it is not visible there. This makes it possible to enfore a variety of indirect allocation mechanisms.

Abstract record types allow to separate interfaces from implementations, so that abstract record types can be exported, without risking direct allocation by clients.

Like abstract types, limited record types cannot be directly allocated by clients.

Implement-only export allows to restrict the execution of allocation and initialization sequences to the defining module, which also prevents clients from binding themselves to concrete classes by instantiating them directly.

These features can be combined to develop safe implementations of all creational patterns described in *Design* *Patterns*: abstract factories, builders, factory methods, prototypes, and singletons.

ꀢ Dependence on specific operations

Static record variables can be used as light-weight (stack-allocated) message objects, instead of using hard-coded method signatures or heavy-weight (heap-allocated) message objects. Message records can be forwarded, filtered, broadcast, and so on. Yet they don't violate type safety, since addresses of static records cannot be manipulated in unsafe ways.

Message records can be used to implement the chain of responsibility and observer patterns, which decouple a message sender and its receiver(s).

ꀢ Dependence on hardware and software platforms

ꀢ Dependence on object representations or implementations

Portability is one of the main advantages of using a true high-level language. Component Pascal abstracts from the underlying hardware, yet its semantic gap is small enough that very efficient machine code can be generated. As in Java, type sizes are defined so that data transfer between machines doesn't create problems.

ꀢ Algorithmic dependencies

Parts of an algorithm can be made replaceable by using abstract methods, empty (hook) methods, or auxiliary objects. Implement-only export makes it possible to safely assign responsibilities: correct method calling sequences must be implemented by the defining module, implementing the methods must be done by the extension programmer. This feature allows to develop safe implementations of all behavioral patterns (builder, internal iterator, strategy, template method, and so on).

ꀢ Tight coupling

Modules act as visibility boundaries. Thus tight coupling is possible where it is necessary (within a module's implementation), while loose coupling is possible across module boundaries. This simplifies safe implementation of virtually all design patterns.

ꀢ Extending functionality by subclassing

Classes can be hidden in modules. Final or limited classes, and final methods can be exported without risk that they may be subclassed or overridden.

ꀢ Inability to alter classes conveniently

Strong typing, record and method attributes such as *NEW*, and run-time assertions increase the confidence that an interface can be changed in a way that all clients can be made consistent again easily and reliably. This is basic software engineering, and thus important for all kinds of designs.

All these problems have one common theme: a local change may cause ripple effects that affect the entire software system. Good languages and design practices help to design for change:

*Ensure that possible design changes have local effects only.*

*Where this is not possible, ensure that the effects are detected by the compiler.*

*Where this is not possible, ensure that the effects are detected at run-time as early as possible.*

It is clear that there still remains a considerable gap between current state-or-the-art languages like Component Pascal, and complete specification languages that also allow to specify the semantics of a program. In the future, it will be the challenge to close those parts of the gap which have a good enough cost/benefit ratio, i.e., which help to change a component's behavior in a controlled manner, without making programs unreadable and unwriteable for average programmers.

**3.2 Modules and subsystems**

In this section, we give a more concrete idea of how Component Pascal code looks like and how it is managed by the BlackBox Component Builder environment. For this purpose, we have to go into more BlackBox-specific details than in the other sections of this part of the book.

All Component Pascal code lives in modules. A module is the compilation unit of Component Pascal. A module has an interface and a hidden implementation. Syntactically this is achieved by marking some items in a module, e.g., some types and procedures, as *exported*. Everything not exported is invisible and thus not directly accessible from outside of the module. If a module needs services of one or several other modules, it *imports* them. Thereby the module declares that it requires descriptions of the interfaces of the imported modules at compile-time, and suitable implementations of these modules at run-time. Listing 3-3 shows a possible implementation of a module called *ObxPhoneDB*. It exports three procedures for looking up phone book entries: by index, by name, and by number. Please note the asterisks which denote items as exported, in this case the type *String* and the three lookup procedures:

MODULE ObxPhoneDB;

    CONST

        maxLen = 32;    *(* maximum length of name/number strings *)*

        maxEntries = 5;    *(* maximum number of entries in the database *)*

    TYPE

        **String*** = ARRAY maxLen OF CHAR;

        Entry = RECORD

            name, number: String

        END;

    VAR db: ARRAY maxEntries OF Entry;

    PROCEDURE **LookupByIndex*** (index: INTEGER; OUT name, number: String);

    BEGIN    *(* given an index, return the corresponding <name, number> pair *)*

        ASSERT(index >= 0);

        IF index < maxEntries THEN

            name := db[index].name; number := db[index].number

        ELSE

            name := ""; number := ""

        END

    END LookupByIndex;

    PROCEDURE **LookupByName*** (name: String; OUT number: String);

        VAR i: INTEGER;

    BEGIN    *(* given a name, find the corresponding phone number *)*

        i := 0; WHILE (i # maxEntries) & (db[i].name # name) DO INC(i) END;

        IF i # maxEntries THEN    *(* name found in db[i] *)*

            number := db[i].number

        ELSE    *(* name not found in db[0..maxEntries-1] *)*

            number := ""

        END

    END LookupByName;

    PROCEDURE **LookupByNumber*** (number: String; OUT name: String);

        VAR i: INTEGER;

    BEGIN    *(* given a phone number, find the corresponding name *)*

        i := 0; WHILE (i # maxEntries) & (db[i].number # number) DO INC(i) END;

        IF i # maxEntries THEN    *(* number found in db[i] *)*

            name := db[i].name

        ELSE    *(* number not found in db[0..maxEntries-1] *)*

            name := ""

        END

    END LookupByNumber;

BEGIN    *(* initialization of database contents *)*

    db[0].name := "Daffy Duck"; db[0].number := "310-555-1212";

    db[1].name := "Wile E. Coyote"; db[1].number := "408-555-1212";

    db[2].name := "Scrooge McDuck"; db[2].number := "206-555-1212";

    db[3].name := "Huey Lewis"; db[3].number := "415-555-1212";

    db[4].name := "Thomas Dewey"; db[4].number := "617-555-1212"

END ObxPhoneDB.

Listing 3-3. Implementation of ObxPhoneDB

Before a module can be used, it must be loaded from disk into memory. But before it can be loaded, it must be compiled (command *Dev->Compile*). When compiling a module, the compiler produces a code file and a symbol file. The code file contains the executable code, which can be loaded into memory. The code file is a kind of super-lightweight DLL. The compiler also produces a symbol file, which contains a binary representation of the module's interface. If a module imports other modules, the compiler reads the symbol files of all these modules, in order to check that their interfaces are used correctly. The compilation process can be visualized in the following way:

Figure 3-4. Compilation process

When you compile a module for the first time, a new symbol file is generated. In the log window, the compiler writes a message similar to the following one:

compiling "ObxPhoneDB"

  new symbol file   964   640

The first of the two numbers indicates that the machine code in the new code file is 964 bytes long. The second number indicates that the module contains 320 bytes global variables (five entries in the db variable; each entry consisting of two strings with 32 elements each; each element is a 2-byte Unicode character). If a symbol file for exactly the same interface already exists, the compiler writes a shorter message:

compiling "ObxPhoneDB"   964   640

If the interface has changed, the compiler writes a new symbol file and indicates the changes compared to the old version in the log. For example, if you just have introduced procedure *LookupByNumber* in the most recent version, the compiler writes:

compiling "ObxPhoneDB"

  LookupByNumber is new in symbol file   964   640

Symbol files are only used at compile-time, they have no meaning at run-time. In order to load a module, only its code file is needed. Modules are loaded dynamically, i.e., there is no separate linking step as required by more static languages. To see a list of currently loaded modules, call command *Info->Loaded Modules*. As result, a window will be opened with a contents similar to the following one:

*module name    bytes used    clients    compiled    loaded*        <u>Update</u>

StdLinks     20639      1      2.7.1996  18:42:15     29.8.1996  14:31:14

StdFolds     20425      1      2.7.1996  18:41:33     29.8.1996  14:31:12

StdCmds     25066      7      2.7.1996  18:39:12     29.8.1996  14:31:00

Config       125      0      2.7.1996  18:38:21     29.8.1996  14:31:20

Init       682      0      2.7.1996  18:40:21     29.8.1996  14:31:05

Controls     78876      5      7.7.1996  14:14:58     29.8.1996  14:31:00

Services      1472      5      2.7.1996  18:37:14     29.8.1996  14:30:54

Containers     37348     40      2.7.1996  18:37:51     29.8.1996  14:30:52

Properties      8337     42      2.7.1996  18:37:40     29.8.1996  14:30:49

Controllers      6037     42      2.7.1996  18:37:36     29.8.1996  14:30:49

Views     31589     49      2.7.1996  18:37:33     29.8.1996  14:30:49

Models      4267     50      2.7.1996  18:37:27     29.8.1996  14:30:48

Converters      2189     51    14.7.1996  22:45:12     29.8.1996  14:30:48

Dialog      8979     54      2.7.1996  18:37:13     29.8.1996  14:30:48

Dates      3848     45      2.7.1996  18:37:07     29.8.1996  14:30:48

Meta     19275     11      2.7.1996  18:37:10     29.8.1996  14:30:48

Stores     22302     53      2.7.1996  18:37:22     29.8.1996  14:30:47

Strings     17547     15      2.7.1996  18:37:05     29.8.1996  14:30:47

Math     15408      2      3.7.1996    1:45:05     29.8.1996  14:30:47

Ports     10631     56      2.7.1996  18:37:17     29.8.1996  14:30:46

Fonts      1589     58      2.7.1996  18:37:15     29.8.1996  14:30:46

Files      3814     62      2.7.1996  18:36:28           linked

...

Table 3-5. List of loaded modules

The list shows all loaded modules. For each module, it shows its code size in bytes, how many other modules import it, when it has been compiled, and when it has been loaded.

It is easy to get an overview over the already loaded modules, but what about modules not yet loaded? The idea of having access to a wealth of prebuilt components raises some organizational issues. How do you find out exactly which components are available? How do you find out which of the available components provide the services that you need?

This is a matter of conventions, documentation, and supporting tools. For the BlackBox Component Builder, it is a convention that collections of related components, called *subsystems*, are placed into separate directories; all of which are located directly in the BlackBox directory. There are subsystems like System, Std, Host, Mac, Win, Text, Form, Dev, or Obx. The whole collection of subsystems is called the BlackBox *repository*. The basic idea behind the repository's structure is that everything that belongs to a component (code files, symbol files, documentation, resources) are stored together in a systematic and simple directory structure, according to the rule

*Keep all constituents of a component in one place.*

It is only appropriate for component-oriented software that addition and removal of a component can be performed incrementally, by adding or removing a directory. All kinds of central installation or registration mechanisms which distribute the constituents of a component should be avoided, since they inevitably lead to (unnecessary) management problems.

Figure 3-6. Standard subsystems of the BlackBox Component Builder

Each subsystem directory, e.g. Obx, may contain the following subdirectories:

Figure 3-7. Structure of a typical subsystem directory

The module source is saved in a subsystem's Mod directory. The file name corresponds to the module name without its subsystem prefix; e.g., the modules *ObxPhoneDB* and *ObxPhoneUI* are stored as Obx/Mod/PhoneDB and Obx/Mod/PhoneUI, respectively. For each source file, there may be a corresponding symbol file, e.g., Obx/Sym/PhoneDB; a corresponding code file, e.g., Obx/Code/PhoneDB; and a corresponding documentation file, e.g., Obx/Docu/PhoneDB. There may be zero or more resource documents in a subsystem, e.g., Obx/Rsrc/PhoneUI. There is not necessarily a 1:1 relationship between modules and resources, although it is generally recommended to use a module name as part of a resource name, in order to simplify maintenance.

Modules whose names have the form SubMod, e.g., *TextModels*, *FormViews*, or *StdCmds*, are stored in their respective subsystems given by their name prefixes, e.g., *Text*, *Form*, or *Std*. The subsystem prefix starts with an uppercase letter and may be followed by several other uppercase letters and then by several lowercase letters or digits. The first uppercase letter afterwards denotes the particular module in the subsystem.

Modules which belong to no subsystem, i.e., modules whose names are not in the form of SubMod, are stored in a special subsystem called *System*. The whole BlackBox library and framework core belongs to this category, e.g., the modules *Math*, *Files*, *Stores*, *Models*, etc.

Each subsystem directory may contain the following subdirectories:

    Code    Directory with the executable code files, i.e., lightweight DLLs.

For example, for module "FormCmds" there is file "Form/Code/Cmds".

A module for interfacing native DLLs (Windows DLLs or Mac OS code fragments) has no code file.

    Docu    Directory with the fully documented interfaces and other docu.

For example, for module "FormCmds" there is file "Form/Docu/Cmds".

For a module that is only used internally, its docu file is not distributed to the customer.

Often, there are further documentation files which are not specific to a particular module of the subsystem. Such files contain one or more dashes as parts of their names, e.g., "Dev/Docu/P-S-I".

Typical files are

"Sys-Map" (overview with hyperlinks to other documents of this subsystem)

"User-Man" (user manual)

"Dev-Man" (developer manual)

    Mod    Directory with module sources.

For example, for module "FormCmds" there is file "Form/Mod/Cmds".

For a module that is not published in source code ("white box"), its source file is not distributed to the customer.

    Rsrc    Directory with the subsystem's resource documents.

For example, for module "FormCmds" there is file "Form/Rsrc/Cmds".

There may be zero, one, or more resource files for one module. If there are several files, the second gets a suffix "1", the third a suffix "2", and so on. For example, "Form/Rsrc/Cmds", "Form/Rsrc/Cmds1", "Form/Rsrc/Cmds2", etc.

Often, there are further resource files which are not specific to a particular module of the subsystem.

Typical files are

"Strings" (string resources of this subsystem)

"Menus" (menus of this subsystem)

    Sym    Directory with the symbol files.

For example, for module "FormCmds" there is file "Form/Sym/Cmds".

For a module that is only used internally, its symbol file is not distributed to the customer.

Table 3-8. Contents of the standard subsystem subdirectories

If you want to find out about the repository, its subsystems and their subdirectories, you can invoke the command *Info->Repository*. If you want to find out more about a module, you can select the module name in a text and then execute *Info->Source*, *Info->Interface* or *Info->Documentation*. These commands open the module's Mod, Sym or Docu files. For this purpose, *Info->Interface* converts the binary representation of the symbol file into a readable textual description. For example, type the string "ObxPhoneDB" into the log window, select the string, and then execute the *Info->Interface* browser command. As a result, the following text will be opened in a new window:

DEFINITION ObxPhoneDB;

    TYPE

        String = ARRAY 32 OF CHAR;

    PROCEDURE LookupByIndex (index: INTEGER; OUT name, number: String);

    PROCEDURE LookupByName (name: String; OUT number: String);

    PROCEDURE LookupByNumber (number: String; OUT name: String);

END ObxPhoneDB.

Listing 3-9. Definition of ObxPhoneDB

A module definition as generated by the browser syntactically is a subset of the module implementation, except for the keyword *MODULE* which is replaced by *DEFINITION*. This syntax can be regarded as the interface description language (IDL) of Component Pascal. Texts in this language are usually created out of complete module sources by the browser or similar tools, and thus need not be written manually and cannot be compiled.

Since the browser command operates on the symbol file of a module, it can be used even if there is neither a true documentation nor a code file available. During prototyping, where documentation is rarely available, the symbol file browser is very convenient for quickly looking up details like the signature of a procedure, or to get an overview over the interface of an entire module. When a full documentation is available, the command *Info->Documentation* can be used. It opens the module's documentation file, which starts with the definition of the module's interface just like above, but then continues with an explanation of the module's purpose and a detailed description of the various items exported by the module, for example:

Module ObxPhoneDB provides access to a phone database. Access may happen by index, by name, or by number. An entry consists of a name and a phone number string. Neither may be empty. The smallest index is 0, and all entries are contiguous.

PROCEDURE **LookupByIndex** (index: INTEGER; OUT name, number: ARRAY OF CHAR)

Return the <name, number> pair of entry *index*. If the index is too large, <"", ""> is returned.

The procedure operates in constant time.

Pre

index >= 0    20

Post

index is legal

    name # ""  &  number # ""

index is not legal

    name = ""  &  number = ""

PROCEDURE **LookupByName** (name: ARRAY OF CHAR; OUT number: ARRAY OF CHAR)

Returns a phone number associated with *name*, or "" if no entry for *name* is found.

The procedure operates in linear time, depending on the size of the database.

Post

name found

    number # ""

name not found

    number = ""

PROCEDURE **LookupByNumber** (number: ARRAY OF CHAR; OUT name: ARRAY OF CHAR)

Returns the name associated with *number*, or "" if no entry for *number* is found.

The procedure operates in linear time, depending on the size of the database.

Post

number found

    name # ""

number not found

    name = ""



Listing 3-10. Documentation of ObxPhoneDB

Note that preconditions and postconditions are documented in a semi-formal notation. Their goal is not to give a complete formal specification, but rather to help making the plain text description less ambiguous, where this is possible without using overly complex (and thus unreadable) formal conditions. The following assertion numbers are used for run-time checking in BlackBox:

Free    0 .. 19

Preconditions    20 .. 59

Postconditions    60 .. 99

Invariants    100 .. 120

Reserved    121 .. 125

Not Yet Implemented             126

Reserved             127

Listing 3-11. Assertion numbers in BlackBox

It is well known that the detection of an error is more difficult and more expensive the later it occurs, i.e., the farther apart the cause and its effects are. This motivates the following design rule:

*Let errors become manifest as early as possible.*

In a component-oriented system, defects should always be contained within their components, and not be allowed to propagate into other components. The other components may even be black-boxes for which no source code is available, which makes source-level debugging impossible. Furthermore, the control flow of a large object-oriented software system is so convoluted that it is unrealistic, and thus a waste of time, to trace it beyond component boundaries for debugging purposes.

The only viable debugging approach is to design everything, from programming language to libraries to components and applications using a defensive programming style. In particular, entry points into components (procedure/method calls) should refuse to execute if their preconditions are not met:

*Never let errors propagate beyond component boundaries.*

Fortunately, most precondition checks are inexpensive and thus their run-time overhead is negligible. This is important because in a component-oriented system, run-time checks cannot be switched off in a production system, because there *are* no separate development and production systems. In practice, most components during development are already debugged ("production") black-boxes, and the others are currently being debugged white-boxes. The production components must cooperate in order to make adherence to the above rule possible, which means never switching off run-time checks.

**3.3 Bottleneck interfaces**

One particular pattern that we would like to discuss here is the Carrier-Rider-Mapper pattern. This pattern is used in several ways in the BlackBox Component Framework: in the text subsystem, in the file abstraction, in the frame abstraction, in the container/context abstraction, and others. Let us consider texts as an example.

We regard a text as a so-called *carrier*. A carrier is an object that carries (contains) data, in this case textual data. Basically, a text can be regarded as a linear stream of elements, where elements have attributes such as font, color, style and vertical offset (for subscript and superscript characters). Elements are characters or views. For all practical purposes, a view in a text can simply be regarded as a special character.

When reading a text, it is convenient not to require the specification of a text position for each and every character read. This convenience can be achieved by supporting a *current position*, i.e., the text itself knows where to read the next character. Each read operation automatically increments the current position.

Several client objects may use a text carrier independently. For example, a text view needs to read its text when it redraws the text on screen, and a menu command may need to read the same text as its input. These clients are independent of each other; they don't need to know about each other. For this reason, the current read position cannot be stored in the text carrier itself, since this would mean that clients could interfere with each other and lose their independence.

To avoid such interference, a carrier provides allocation functions that return so-called *rider* objects. A rider is an independent access path to a carrier's data. Every text rider has its own current position, and possibly further state such as the current attributes.

Figure 3-12. Carrier-Rider separation

The reason why carrier and rider are separated into different data types is the one-to-many relationship between the two: for one carrier there may be an arbitrary number (including zero) riders.

Typically, carrier and rider need to be implemented by the same component, since a rider must have intimate knowledge about the internals of its carrier. For example, text riders and text carriers are both implemented in module *TextModels*. A text rider contains hidden pointers to the internal data structure of a text carrier. Since this information is completely hidden by the module boundary of *TextModels*, no outside client can make a rider inconsistent with its carrier. This kind of invariant, guaranteed by Component Pascal's module system, is an example why information hiding beyond single classes is important for safety reasons.

In the design patterns terminology, a text rider is an *external iterator*. However, not all riders are necessarily iterators. For example, a context object managed by a container is not an iterator.

Text riders come in two flavors: readers and writers. Readers support the reading of characters and views, while writers support the writing of characters and views. For example, the following calls may be made:

    reader.ReadView(view)



    or



    writer.WriteChar("X")

Often, a programmer needs to read or write complex character sequences, so that working at the level of individual characters is too cumbersome. A higher-level abstraction for reading and writing is clearly desirable. This is the purpose of the *mappers*. A mapper contains a rider that it uses to provide more problem-oriented operations for reading or for writing. Since there exist very different programming problems, different applications may use different mappers, possibly even while working with the same carrier.

For text manipulation, BlackBox provides module *TextMappers*, which defines and implements two text mappers: *formatters* for writing, and *scanners* for reading.

Figure 3-13. Carrier-Rider-Mapper separation

Both scanner and formatter work on the level of Component Pascal symbols: with integer numbers, real numbers, strings, and so on. For example, the following calls may be made:

    scanner.ReadInt(int)



    or



    formatter.WriteReal(3.14)

If there are different mappers for the same kind of carrier, they all have to be implemented in terms of the rider interface. For this reason, the rider/carrier interface is sometimes called a *bottleneck interface*. The reason why riders and mappers are separated is not a one-to-many relation as with carrier and rider - there is a one-to-one relation between mapper and rider -  but independent extensibility: special circumstances may require special mappers. If carriers and mappers adhere to a well-defined rider (bottleneck) interface, it is possible to add new mappers anytime, and to use them with the same carrier.

Even better: the carrier/rider interface can be an abstract interface, such that different *implementations* of it may exist. For example, a file carrier may be implemented differently for floppy disk files, hard disk files, CD-ROM files, and network files. But if they all implement the same bottleneck interface, then *all* file mappers can operate on *all* file implementations! Compare this with a situation where you would have to re-implement all mappers for all carriers: instead of implementing n carriers plus m mappers, you would have to implement n carriers plus n * m mappers! Wherever such an extensibility in two dimensions occurs (carrier/rider and mapper), a bottleneck interface is needed to avoid the so-called *cartesian product problem*, i.e., an explosion of implementations.

Note that a bottleneck interface is not extensible itself, because every extension that cannot be implemented in terms of the bottleneck interface invalidates all its existing implementations. For example, if you extend the interface of a device driver for black-and-white bitmap displays by a color extension, then all existing device driver implementations will have to be updated. Since they probably have been implemented by different companies, this can become a major problem.

The problem can be defused if the new interface extension is specified as optional: then clients must test whether the option is supported. If not, a client either has to signal that it cannot operate in this environment, or it must gracefully degrade by providing a more limited functionality. If the optional interface is supported and used, it acts as a kind of "conspiracy" between the interface implementation and its client. This is admissible, but it clearly reduces the combinations of clients and implementations that are fully functional. This underlines how crucial good bottleneck designs are, in order to avoid the need for later extensions.

Figure 3-14. Extensibility in two dimensions

In terms of design patterns, the Carrier-Rider-Mapper pattern solves the problem of flexible and convenient access to a data carrier, however it is implemented. It can be regarded to consist of two simpler and very basic design patterns: the separation of one object into several objects to allow many-to-one relations; and the separation of one object into two objects to allow for independent extensibility:

*Split an abstraction into two interfaces if several clients may access an instance simultaneously, and if independent state may have to be managed for every client.*

*Split an abstraction into two interfaces, if it needs to be extended independently in two different dimensions.*

The Carrier-Rider-Mapper separation goes back to a research project ["Insight ETHOS: On Object-Orientation in Operating Systems"; Clemens Szyperski; vdf, Zürich, 1992, ISBN 3 7281 1948 2] predating BlackBox. This project used several design patterns and design rules (e.g., avoidance of implementation inheritance) that are also described in *Design Patterns*. In the *Design Patterns* terminology, a Rider-Mapper combination (or Carrier-Mapper combination if there is no rider) forms a *bridge* pattern.

In BlackBox, riders are often created and managed in a particular way. A rider is typically under exclusive control of one client object, because after all, the reason why there can be multiple riders on one carrier (and thus why the two are distinguished) is precisely to allow several clients access to the carrier's data via their private access paths. Since riders are used in such controlled environments, BlackBox usually creates riders in the following way:

    rider := carrier.NewRider(rider);

The idea is that if there already exists an old rider that isn't used anymore, it can be recycled. Recycling is done by the *NewRider* factory method (see next section) if possible. Of course, recycling is only legal if the old rider isn't used anymore for something else (possibly by someone else); that's why it is important that the rider is maintained in a controlled environment.

Typically, a *NewRider* procedure is implemented in the following way:

    PROCEDURE (o: Obj) NewRider (old: Rider): Rider;

        VAR r: RiderImplementation;

    BEGIN

        IF (old # NIL) & (old IS RiderImplementation) THEN    *(* recycle old rider *)*

            r := old(RiderImplementation)

        ELSE    *(* allocate new rider *)*

            NEW(r)

        END;

        ... initialize r ...

        **RETURN** r

    END NewRider;

Listing 3-15. Rider recycling

This approach is used in BlackBox for files, texts, and forms. It can make a considerable difference in efficiency, depending on the kind of application. However, where efficiency is not a concern, it is probably a good idea to omit this mechanism, thereby avoiding any possibility for inadvertant reuse of riders in different contexts simultaneously.

**3.4 Object creation**

The BlackBox Component Framework is mostly a black-box design. It strictly separates interface from implementation. A client can only import, and thus directly use or manipulate, interfaces of objects. The implementations are hidden within the modules. This enforces that module clients adhere to the following design rule (Gamma et.al.):

*Program to an interface, not to an implementation*.

In Component Pascal, object interfaces are represented as abstract record types. An implementation of an abstract record type is a concrete extension of it. Typically, implementations are not exported and thus cannot be extended ("subclassed") in other modules, meaning that implementation inheritance cannot be used. Thus it is also enforced that extension programmers adhere to the following design rule (Gamma et.al.):

*Favor object composition over class inheritance.*

Concrete record types cannot even be instantianted directly: because a client cannot import an object implementation whose type is not exported, it cannot allocate an instance by calling *NEW*. This is desirable, because *NEW* would couple the client code forever to one particular object implementation; reuse of the code for other implementations wouldn't be possible anymore. But how can client code solve the *black-box allocation problem*; i.e., obtain an object implementation without specific knowledge of it?

Figure 3-16. Exported interface records and hidden implementation records

Instead of calling *NEW* it might be possible to call an allocation function instead, a so-called *factory function*. For example, module *TextModels* might export a function *New* that allocates and returns a text model. Internally, it would execute a *NEW* on the non-exported implementation type and possibly perform some initialization. This would be better than clients directly calling *NEW*, because the allocating module can guarantee correct initialization, and a new release of the module could perform allocation or initialization in some different way without affecting clients. However, static factory functions are still too inflexible. A proper solution of the black-box allocation problem requires a level of indirection. There are several approaches to achieve this. They are called creational patterns. BlackBox uses four major creational patterns: prototypes, factory methods, directory objects, and factory managers. They are discussed one by one.

Sometimes it is necessary to create an object of the same type as some other already existing object. In this case, the existing object is called a *prototype*, which provides a factory function or initialization method. BlackBox models can act as prototypes. For example, it is possible to clone a prototype and to let the prototype initialize its clone. This makes sure that the newly created object has exactly the same concrete type as the prototype, and it allows the prototype and its clone(s) to share caches for performance reasons. For example, a BlackBox text model and its clones share the same spill file that they use for buffering temporary data. Sharing avoids the proliferation of open files.

Sometimes there exists an object that can be used to create an object of *another* type. For this purpose, the existing object provides a factory function, a *factory method*. Possibly, the object may provide different factory methods which support different initialization strategies or which create different types of concrete objects.

In BlackBox, a text model provides the procedures *NewReader* and *NewWriter*, which generate new readers and writers. They are rider objects that represent access paths to a text.

Factory methods are appropriate for objects that are implemented simultaneously ("*implementation covariance*"). This is always the case for riders and their carriers.

But this approach is not sufficient for allocating the text models themselves. Where does the first text model come from? BlackBox uses special objects with factory methods, so-called *factory objects*. In BlackBox, factory objects are used in a particular way: they are installed in global variables and may be replaced at run-time, without affecting client code. For historical reasons, we call factory objects which are used for configuration purposes *directory objects*.

For example, module *TextModels* exports the type *Directory* which contains a *New* function, furthermore it exports the two variables *dir* and *stdDir* and the procedure *SetDir*. By default, *TextModels.dir* is used by client code to allocate new empty text models. *TextModels.stdDir* contains the default implementation of a text model, which is mostly useful during the debugging of a new text model implementation. For example, you could use the old directory while developing a new text model implementation. When the new one is ready you install its directory object by calling *SetDir*, thereby upgrading the text subsystem to the new implementation on the fly. From this time, newly created texts will have the new implementation. Texts with the old implementation won't be affected. If the new implementation has errors, the default implementation can be reinstalled by calling *TextModels.SetDir(TextModels.stdDir)*. The text subsystem uses directory objects in this typical way; the same design pattern can be found in other container model abstractions; e.g., for forms models. In BlackBox, it is even used for much simpler models and views; e.g., for *DevMarkers.*

Note that objects allocated via directories are often persistent and their implementation thus long-lived. For this reason, several different implementations appear over time which have to be used simultaneously.

Typically, the normal default directory objects of a subsystem are installed automatically when the subsystem is loaded. If other directory objects should be installed, a *Config* module is used to set up the desired directory object configuration upon startup.

The name "directory object" comes from another use of the directory design pattern: in module *Files*, a file can be created by using the directory object *Files.dir*. But a *Files.Directory* also provides means to find out more about the current configuration of the file system (e.g., procedure *Files.Directory.FileList*).

File directories are interesting also because they show that a replacement directory may sometimes need to forward to the replaced directory object. For example, if the new directory implements special memory files, which are distinguished by names that begin with "M:\", then the new directory must check at each file lookup whether the name begins with the above pattern. If so, it is a memory file. If not, it is a normal file and must be handled by the old directory object.

There may exist several directory objects simultaneously. For example, if a service uses files in a particular way, it may provide its own file directory object which by default forwards to the file system's standard directory object.

Multiple directory objects and chaining of directory objects are part of the directory design pattern. This chaining part of the pattern is cascadable, if a new directory object accesses the most recent old one, instead of *stdDir*.

Note that unlike typical factory classes, directories are rarely extended.

Figure 3-17. Forwarding between two directory objects

Directory objects are a particularly appropriate solution when the existence of one dominating implementation of an extensible black-box type can be expected, without excluding the existence of other implementations.

On the other hand, there could also be several special directory objects for the same implementation but for different purposes. For example, there may be a special text directory object for texts that have a special default ruler useful for program texts. A program editor would then create new program text via this directory object rather than via the standard directory of the text subsystem.

A *registry* would be a valuable addition to directory objects. A registry is a persistent central database which stores configuration information, such as the particular directory objects to install upon startup. The problem with registries is that they contradict the decentral nature of software components, which leads to management problems such as finding registry entries that are no longer valid.

We have seen that allocation via one level of indirection is key to solving the black-box allocation problem. Directory objects are a solution involving global state. This is only desirable if this state changes rarely, and if no parallel activities occur (threads). For example, in server environments a state-free solution is often preferable. This can be achieved by parameterizing the client: the client must be passed a factory object as parameter. This makes it possible to determine the exact nature of an implementation at the "top" of a hierarchy of procedure calls, always passing the factory object "downward". This is useful since the top-level is typically much less reusable than lower-level code.

As an example, a file transfer dialog box may allow to specify the file transfer protocol along with information such as remote address, communication speed, and so on. The file transfer protocol, which indicates the implementation of a communication object, can be passed from the dialog box down to the communication software, where it is used to allocate a suitable communication object (e.g., a TCP stream object).

Instead of passing a factory object, its symbolic name may be passed. At the "bottom", a service interprets this name; if necessary loads the module which implements the corresponding factory function; and then creates an object using the factory function. In this way, the service acts as a *factory manager*.

This approach is used for some BlackBox services, in particular for the *Sql* and *Comm* subsystems. In both cases, the name of a module can be passed when allocating a new object. This module is expected to provide suitable factory functions. For *Sql*, the modules *SqlOdbc* and *SqlOdbc3* provide appropriate object implementations and factory functions. For *Comm*, the module *CommTCP* provides a suitable object implementation.

Factory managers are often more suitable for creating non-persistent objects, while directory objects are often more suitable for creating persistent objects.

