** Direct-To-COM Compiler**

<a id="1"></a>**Introduction**

<a id="1.1"></a>**COM**

Microsoft's Component Object Model (COM) is a language and compiler independent interface standard for object interoperability on the binary level. COM specifies how objects implemented in one component can access services provided by objects implemented in another component. The services provided by objects are grouped in *interfaces*, which are sets of semantically related functions. For example, an ActiveX control is a COM object that extends OLE, by implementing concrete implementations of the necessary OLE interfaces. COM makes it possible that an extended software component can be updated without invalidating extending components, e.g., a new release of OLE can be introduced without breaking existing ActiveX controls.

In such an evolving, decentrally constructed system it is difficult to decide whether an object is still used or whether it can be removed from the system. The client of an object does not know whether other clients still have access to the same object, and a server does not know whether a client passed its reference to other clients. This lack of global knowledge can make it hard to determine whether an object is no longer referenced, and thus could be released. If an object is released prematurely, then unpredictable errors will occur. The problem is critical in component software environments, because an erroneous component of one vendor may destroy objects implemented by other vendors' components. It may even take some time until the destroyed objects begin to visibly malfunction. For the end user, it is next to impossible to determine which vendor is to blame.

As a consequence, some form of automatic garbage collection mechanism must be provided. In contrast to closed systems, this is a necessity with component software, and not merely an option or a convenience feature. COM uses reference counting as a rather simple form of garbage collection. Whenever a client gets access to a COM object, it must inform the object that a new reference exists. As soon as the client releases the COM object, it must inform the object that a reference will disappear. A COM object counts its references and thus can control its own lifetime.

<a id="1.2"></a>**Direct-To-COM Compiler**

The problem with reference counting is that it depends on the discipline of the programmer. The situation is aggravated by additional subtle rules about who is responsible for the management of the reference counts if COM objects are passed as function arguments. Reliable software construction requires the management of reference counts to be automated.

The Direct-To-COM compiler for Component Pascal completely automates memory management. The compiler and its small run-time system (module *Kernel*) hides the reference counting mechanism, i.e., a program need not call nor implement the *AddRef* and *Release* methods.

For programmers, the integration of COM into a strongly typed, garbage-collected language has two major advantages. Firstly, it brings all the amenities of automatic garbage collection to COM. Secondly, it makes the handling of COM interfaces typesafe.

<a id="1.3"></a>**BlackBox Component Builder**

BlackBox Component Builder is the name of the integrated development environment (IDE) of the Direct-To-COM compiler. It provides a program editor, the compiler, and a symbolic debugger. It also contains a *multiplatform component framework*, which simplifies the implementation of platform-independent software components. However, it is assumed that you won't need access to the BlackBox APIs, since they isolate software from non-portable COM interfaces, while you want to do COM programming.

Note

It is possible to do COM programming while also using the component framework, e.g. to access some functionality for which there is no framework counterpart. In such a case, you can always access the full framework documentation via the help screen.

<a id="1.4"></a>**Getting Started**

How should you start to get acquainted with this product? Note that all documentation is available on-line, and can be reached from the help screen. After the<u> [Installation</u>](../System/Docu/User-Man.odc.md)<u> </u>of the BlackBox Component Builder, we suggest that you start with the introduction text<u> [A Brief History of Pascal</u>](Tut-A.odc.md).

Then move on to the chapters of the user guide which deal with text editing and with the development tools ([<u>Text Subsystem</u>](../Text/Docu/User-Man.odc.md) and [<u>Dev Subsystem</u>](../Dev/Docu/User-Man.odc.md)). After you thus have gained a working knowledge of BlackBox, it is time to delve into the COM-specific parts of the IDE and the Component Pascal compiler extensions for COM.

The section [<u>How to Develop new COM objects</u>](DTC-HowTo.odc.md) (p. 20) summarizes the main points of the user guide as far as it pertains to COM development. Then proceed with the various example implementations of COM objects, which demonstrate all basic aspects of COM programming. Knowledge of the Windows APIs (e.g. OLE) is assumed.

<a id="1.5"></a>**Further Reading**

We recommend the following books on COM, OLE and ActiveX:

    Kraig Brockschmidt,

    *Inside OLE,*

*    *2nd Edition, Microsoft Press, 1995.



    David Chappell,

    *Understanding ActiveX and OLE,*

*    *Microsoft Press, 1996, ISBN 1-57231-216-5.

    Adam Denning,

    *OLE Controls Inside Out,*

*    *Microsoft Press, 1995.



    Dale Rogerson,

*    Inside COM,*

    Microsoft Press, 1997, ISBN 1-57231-349-8,



    Don Box,

*    Creating Components with DCOM and C++,*

    Addison Wesley, 1997.



    Microsoft

    *COM Home Page, *

    http://www.microsoft.com/com

<a id="1.6"></a>**Changes since Release 1.2**

The upgrade of BlackBox 1.2 to BlackBox 1.3 also made the upgrade of the Direct-To-COM compiler necessary. However, the new language Component Pascal now supports many extensions which have been introduced in the language DTC Oberon which was used with DTC Release 1.2. In particular, the following features now belong to Component Pascal and are no longer special to DTC:

[in] sysflag for VAR parameters

instead of the [in] sysflag, which specified read-only VAR parameters, the IN parameter mode can now be used. IN parameters can neither be written nor passed as actual parameter to a formal VAR parameter. An IN parameter can only be read, or passed as actual parameter to another IN parameter. As with the [in] parameters in DTC Release 1.2, constants strings can be passed as actual parameters of a Component Pascal IN parameters.



[out] sysflag for VAR parameters

instead of the [out] sysflag, the Component Pascal OUT parameter mode can be used to mark *out* parameters. Actual pointer parameters to formal OUT parameters are initialized to NIL upon function call. If a COM interface method is called remotely, the actual parameters of OUT parameters are not passed upon procedure entry and their values on the server side are not defined.

LONGCHAR

Component Pascal now supports 16 bit characters (CHAR), which replaces type LONGCHAR of DTC Release 1.2. The predefined procedure LCHR is now called CHR.

LARGEINT

Component Pascal now supports 64 bit integers (LONGINT) which is equivalent to the type LARGEINT of DTC Release 1.2. The predefined procedure LENTIER is now called ENTIER.

FINALIZE Method

The FINALIZE method for (tagged) records is now part of Component Pascal. This method is called before the object is collected by the garbage collector. The RELEASE method is still unique to COM objects.

TERMINATE

Support for module finalization is now provided by Component Pascal. Whenever a module is unloaded, the CLOSE section of the module body is executed.

Compiler Implementation Restrictions

some of the compiler restrictions have been removed in the new release. In particular, it is now possible to specify a function call as actual value for a formal parameter of type "pointer to interface", and function calls with interface pointer results may now be used as arguments for pointer comparison operations.

