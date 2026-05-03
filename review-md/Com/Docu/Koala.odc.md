<a id="7.1"></a>**ComKoala Example**

This example is one of the simplest COM examples. It provides a component which implements the *IUnknown* interface. In DTC Component Pascal, a concrete extension of the base interface must be defined. The two methods AddRef and Release are implemented by the compiler, and for QueryInterface, the default implementation can be taken. Thus, only the following two lines are necessary to implement this simple interface.

    TYPE

        Koala = POINTER TO RECORD (COM.IUnknown) END;

A set of interfaces is implemented in a COM class. A COM class may be identified by a unique class identifier (CLSID) which is also a GUID. A run-time instantiation of a COM class is called a COM object. Each COM class is implemented in a COM server. The server is a piece of code compiled into an executable (EXE file) or into a dynamic link library (DLL file). For every COM class, the server must provide a factory object through which a new instance can be constructed. This factory object can be registered in the Windows registry.

    CONST

        KoalaId = "{00021146-0000-0000-C000-000000000046}"; (* the CLSID *)



    TYPE

        KoalaFactory = POINTER TO RECORD (WinOle.IClassFactory) END;

The interface of the factory supports two methods: *CreateInstance* and *LockServer*. The methods of the *IUnknown* interface are again implemented by the compiler.

Through CreateInstance new Koala objects are requested. The required interface of the new object is passed as parameter iid. The implementation simply creates a new Koala object (using NEW) and asks QueryInterface of the newly generated object to return the requested interface.

    PROCEDURE (this: KoalaFactory) CreateInstance (outer: COM.IUnknown;

                IN [iid] iid: COM.GUID;

                OUT [new] int: COM.IUnknown): COM.RESULT;

        VAR res: COM.RESULT; new: Koala;

    BEGIN

        IF outer # NIL THEN RETURN WinApi.CLASS_E_NOAGGREGATION END;

        NEW(new);

        IF new # NIL THEN RETURN new.QueryInterface(iid, int)

        ELSE RETURN WinApi.E_OUTOFMEMORY

        END

    END CreateInstance;



In module *ComKoalaTst* some test routines are implemented which demonstrate the use of *ComKoala* and which in particular illustrate the usual szenario when creating COM objects.

First, the *ComKoala* server must instantiate a class factory and register it with the command *WinOle.CoRegisterClassObject* under its CLSID. This is implemented in the command

     ComKoala.Register

If you now look at the interface inspector you see that interface *ComKoala.KoalaFactory* is allocated and has a reference count of one (the reference in the registry).

Now the client can request a pointer to the class factory by calling* WinOle.CoGetClassObject.* As second argument we specify that the local server (i.e. BlackBox) has to be used. Now the reference count of the factory has been incremented to two (the reference in the registry and the reference we just created).

     ComKoalaTst.CreateClass

Through the factory object a Koala object can now be generated. The factory creates an instance of *ComKoala.Koala* and returns a pointer to the *IUnknown* interface. The interface browser now shows a new allocated interface.

     ComKoalaTst.CreateInstance



With the commands *ComKoalaTst.AddRef *and *ComKoalaTst.Release* the number of references can be incremented or decremented. Look at the updated interface browser from time to time. Whenever the command *CreateInstance* is called again, a new instance is created.

     ComKoalaTst.AddRef

     ComKoalaTst.Release



The factory reference which is held by module *ComKoalaTst* can be released with the command

     ComKoalaTst.ReleaseClass

and the reference in the Windows registry is removed by calling *WinOle.CoRevokeClassObject. *The latter operation is implemented in the command

     ComKoala.Release

Note that the two references to the factory can be released in any order. As soon as the registry reference is released, no new classes can be generated through *WinOle.CoGetClassObject,* and as soon as the factory reference in module *ComKoalaTst* is released, no new instances can be created.

The implementation can also be tested with the ObjUser.exe program which is provided in the Inside OLE book. Before you can use it, you must register the factory (ComKoala.Register) and you must set the ObjUser.exe program in EXE mode.

[<u>ComKoala  sources</u>](../Mod/Koala.odc.md)

[<u>ComKoalaTst  sources</u>](../Mod/KoalaTst.odc.md)

