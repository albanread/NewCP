**Appendix A: A Brief History of Pascal**

**Algol**

The language Component Pascal is the culmination of several decades of research. It is the youngest member of the Algol family of languages. Algol, defined in 1960, was the first high-level language with a readable, *structured*, and systematically defined syntax. While successful as a notation for mathematical algorithms, it lacked important data types, such as pointers or characters.

**Pascal**

In the late sixties, several proposals for an evolutionary successor to Algol were developed. The most successful one was Pascal, defined in 1970 by Prof. Niklaus Wirth at ETH Zürich, the Swiss Federal Institute of Technology. Besides cleaning up or leaving out some of Algol's more obscure features, Pascal added the capability to define new data types out of simpler existing ones. Pascal also supported *dynamic data structures*; i.e., data structures which can grow and shrink while a program is running. Pascal received a big boost when ETH released a Pascal compiler that produced a simple intermediate code for a virtual  machine (P-code), instead of true native code for a particular machine. This simplified porting Pascal to other processor architectures considerably, because only a new P-code interpreter needed be written for this purpose, not a whole new compiler. One of these projects had been undertaken at the University of California, San Diego. Remarkably, this implementation (UCSD Pascal) didn't require a large and expensive mainframe computer, it ran on the then new Apple II personal computers. This gave Pascal a second important boost. The third one came when Borland released TurboPascal, a fast and inexpensive compiler, and integrated development environment for the IBM PC. Later, Borland revived its version of Pascal when it introduced the rapid application development environment Delphi.

Pascal has greatly influenced the design and evolution of many other languages, from Ada to Visual Basic.

**Modula-2**

In the mid-seventies, inspired by a sabbatical at the Xerox Palo Alto Research Center *PARC*, Wirth started a project to develop a new workstation computer. This workstation should be completely programmable in a high-level language, thus the language had to provide direct access to the underlying hardware. Furthermore, it had to support *team programming* and modern software engineering principles, such as *abstract data types*. These requirements led to the programming language Modula-2 (1979). Modula-2 retained the successful features of Pascal, and added a module system as well as a controlled way to circumvent the language's type system when doing *low-level programming*; e.g., when implementing device drivers. Modules could be added to the operating system at run-time. In fact, the whole operating system consisted of a collection of modules, without a distinguished kernel or similar artefact. Modules could be compiled and loaded separately, with complete *type and version checking of their interfaces*.

Modula-2 has made inroads in particular into safety-critical areas, such as traffic control systems.

**Simula, Smalltalk, and Cedar**

Wirth's interest remained with desktop computers, however, and again an important impulse came from Xerox PARC. PARC was the place where the workstation, the laser printer, the local area network, the bitmap display, and many other enabling technologies have been invented. Also, PARC adopted and made popular several older and barely known technologies, like the mouse, interactive graphics, and *object-oriented programming*. The latter concept, if not the term, was first applied to a high-level language in Simula (1966), another member of the Algol language family. As its name suggests, Simula used object-orientation primarily for simulation purposes. Xerox PARC's Smalltalk language (1983), however, used it for about *anything*. The Smalltalk project broke new ground also in user interface design: the graphical user interface (GUI) as we know it today was developed for the Smalltalk system.

At PARC, these ideas influenced other projects, e.g., the Cedar language, a Pascal-style language. Like Smalltalk and later Oberon, Cedar was not only the name of a language but also of an operating system. The Cedar operating system was impressive and powerful, but also complex and unstable.

**Oberon**

The Oberon project was initiated in 1985 at ETH by Wirth and his colleague Jürg Gutknecht. It was an attempt to distill the essence of Cedar into a comprehensive, but still comprehensible, workstation operating system. The resulting system became very small and efficient, working well with only 2 MB of RAM and 10 MB of disk space. An important reason for the small size of the Oberon system was its component-oriented design: instead of integrating all desirable features into one monolithic software colossus, the less frequently used software components (modules) could be implemented as extensions of the core system. Such components were only loaded when they were actually needed, and they could be shared by all applications.

Wirth realized that component-oriented programming required some features of object-oriented programming, such as *information hiding*, *late binding*, and *polymorphism*.

Information hiding was the great strength of Modula-2. Late binding was supported by Modula-2 in the form of procedure variables. However, polymorphism was lacking. For this reason, Wirth added *type extension*: a record type could be declared as an extension of another record type. An extended type could be used wherever one of its base types might be used.

But component-oriented programming is more than object-oriented programming. In a component-based system, a component may share its data structures with arbitrary other components, about which it doesn't know anything. These components usually don't know about each other's existence either. Such mutual ignorance makes the management of dynamic data structures, in particular the correct deallocation of unused memory, a *fundamentally* more difficult problem than in closed software systems. Consequently, it must be left to the language implementation to find out when memory is not used anymore, in order to safely reclaim it for later use. A system service which performs such an automatic storage reclamation is called a *garbage collector*. Garbage collection prevents two of the most evasive and downright dangerous programming errors: *memory leaks* (not giving free unused memory) and *dangling pointers* (releasing memory too early). Dangling pointers let one component destroy data structures that belong to other components. Such a violation of *type safety* must be prevented, because component-based systems may contain many third-party components of unknown quality (e.g., downloaded from the  Internet).

While Algol-family languages always had a reputation of being safe, complete type safety (and thus garbage collection) still was a quantum leap forward. It also was the reason why complete compatibility with Modula-2 was not possible. The resulting revision of Modula-2 was called the same way as the system: Oberon.

Oberon's module system, like the one of Modula-2, provided information hiding for entire collections of types, not only for individual objects. This allowed to define and guarantee invariants spanning several cooperating objects. In other words: it allowed developers to invent higher-level safety mechanisms, by building on the basic *module safety* and type safety provided by a good Oberon implementation.

Orthodox object-oriented programming languages such as Smalltalk had neglected both typing (by not supporting types) and information hiding (by restricting it to objects and classes), which was a major step backwards as far as software engineering is concerned. Oberon reconciled the worlds of object-oriented and modular programming.

As a final requirement of component-oriented programming, it had to be possible to *dynamically load* new components. In Oberon, the unit of loading was the same as the unit of compilation: a module.

**Component Pascal**

In 1992, a cooperation with Prof. H. P. Mössenböck led to a few additions to the original Oberon language ("Oberon-2"). It became the de-facto standard of the language.

In 1997, the ETH spin-off Oberon microsystems, Inc. (with Wirth on its board of directors) made some small extensions to Oberon-2 and called it Component Pascal, to better express its focus (component-oriented programming) and its origin (Pascal). It is the industrial-strength version of Oberon, so to say.

The main thrust of the enhancements compared to Oberon-2 was to give the designer of a framework (i.e., of module interfaces that define abstract classes for a particular problem domain) more control over the framework's custom safety properties. The benefit is that it becomes easier to ascertain the integrity of a large component-based system, which is particularly important during iterative design cycles when the framework is being developed, and later when the system architecture must be refactored to enable further evolution and maintenance.

**BlackBox**

Oberon microsystems developed the *BlackBox Component Framework* starting in 1992 (originally it was called Oberon/F). This component-oriented framework is written in Component Pascal, and simplifies the development of graphical user interface components. It comes bundled with several BlackBox extension components, including a word processor, a visual designer, an SQL database access facility, an integrated development environment, and the Component Pascal run-time system. The complete package is an advanced yet light-weight rapid application development (RAD) tool for components, called *BlackBox Component Builder*. It is light-weight because it is completely built out of Component Pascal modules - including the kernel with the garbage collector, and the Component Pascal compiler itself.

