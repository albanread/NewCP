** Direct-To-COM Compiler**

<a id="5"></a>**The Integrated Development Environment**

**Contents**

[<u>Linker</u>](#5.1)

[<u>Interface Browser</u>](#5.2)

[<u>COM Interface Inspector</u>](#5.3)

[<u>Other COM Commands</u>](#5.4)

Besides the compiler (which is described in COM Compiler Extension documentation), the DTC development environment offers some special facilities. Most of them are accessible through the COM menu.

<a id="5.1"></a>**Linker**

With the linker Component Pascal modules can be linked to DLL (in process servers) or to EXE (out of process servers) files. Furthermore it can be decided whether a dynamic module loader should be added or not. This leads to the following four link commands:

    EXE    DLL



unextensible    DevLinker.LinkExe    DevLinker.LinkDll

extensible    DevLinker.Link    DevLinker.LinkDynDll

For further information about the linker we also refer to the [*<u>DevLinker</u>*](../Dev/Docu/Linker.odc.md) Documentation.

*DevLinker.Link*

This command links a module set containing a dynamic module loader to an EXE file.

At startup the body of the main module is called.

Initialization and termination of the dynamically loaded modules must be done by the runtime system.

*DevLinker.LinkExe*

This command links an nonextensible module set to an EXE file.

At startup the bodies of all modules are called in the correct order.

If the last body terminates, the terminators of all modules are called in reverse order.

No runtime system is needed for initialization and termination.

For an example see [*<u>ComKoalaExe</u>*](../Com/Docu/KoalaExe.odc.md).

*DevLinker.LinkDll*

This command links a non-extensible module set to a DLL file.

When the DLL is attached to a process, the bodies of all modules are called in the correct order.

When the DLL is released from the process, the terminators of all modules are called in reverse order.

No runtime system is needed for initialization and termination.

For an example see [*<u>ComKoalaDll</u>*](../Com/Docu/KoalaDll.odc.md).

*DevLinker.LinkDynDll*

(rarely used, present for completeness)

Links a module set containing a dynamic module loader to a DLL file.

When the DLL is attached to a process, the body of the main module is called.

When the DLL is released from the process, the terminator of the main module is called.

Initialization and termination of the dynamically loaded modules must be done by the runtime system.

<a id="5.2"></a>**Interface Browser**

To quickly retrieve the actual definition of an interface, a special interface browser is available under the menu command COM->Interface Info. In contrast to the BlackBox browser, the COM interface browser displays additional information such as the number of the functions in the interface. Note that the first three entries of every interface are used by the functions of the IUnknown interface.

If you select WinOle.IEnumUnknown and execute COM->Interface Info the following window is opened:

<a id="5.3"></a>**COM Interface Inspector**

The COM interface inspector is the most important tool for the development of COM objects. It allows to inspect all currently allocated interface records. The browser is opened with the command COM->Show Interfaces.

For every interface the actual value of its reference count is displayed. After the name and the address of an interface, a diamond mark allows to follow the pointer to the record to which it points (by clicking on the diamond mark). If the interface is anchored globally through a Component Pascal reference, the global anchor is also shown.

With the <u>Update</u> link the interface inspector can be updated, and the <u>All</u> link allows to toggle between the display of all interfaces and of only the non-BlackBox-internal ones.

Interfaces with reference count zero are no longer referenced through an interface pointer, however they might still be referenced by a Component Pascal pointer. Interfaces which are neither referenced through an interface pointer nor through a Component Pascal pointer are garbage collected upon the next run of the garbage collector. The run of the garbage collector can be enforced through the menu command COM->Collect. The collected interfaces disappear when the interface inspector is updated.

Note: If you look at the ComObject example, you will see that every ComObject.Object interface contains Component Pascal pointers to the three interfaces ComObject.IOleObject, ComObject.IDataObject and ComObject.IPersistStorage, i.e. they are not removed by the garbage collector as long as interface pointers refer to a ComObject.Object.

<a id="5.4"></a>**Other COM Commands**

*Show Error*

Almost all COM and OLE API functions and nearly every interface method returns a value of type COM.RESULT. COM.RESULT is an alias type to INTEGER. A result code consists of a severity bit (success or error), a facility code and an error code. If result >= 0, then it is a success code, otherwise it is an error code.

The menu command COM->Show Error can be used to obtain a full text description of the selected result. As input a selected integer is taken. The integer may be given in decimal or hexadecimal notation. E.g. if you select 1236 and execute COM->Show Error, the following window is opened:

For the use in programs the most common error codes are defined in module WinApi.

*New GUID*

The command COM->New GUID generates a text which contains 10 interface GUIDs. It calls the OLE procedure *CoCreateGuid*.

*Collect*

The command COM->Collect explicitly calls the garbage collector.

