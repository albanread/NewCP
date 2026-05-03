<a id="7.4"></a>**ComPhoneBook Example**

In this example we show the implementation of a COM component which implements a simple phone directory database that manages customer names and telephone numbers. This database can be queried by name and by number.

For our phone directory service we define the interface *ILookup*. This interface contains the methods *LookupByName* and *LookupByNumber*, which return a phone number for a given name, and vice versa. Clients of the phonebook component can access the component's services through this interface only. The interface identifier (IID) of this interface is {C4910D71-BA7D-11CD-94E8-08001701A8A3}. This IID is a globally unique identifier (GUID), i.e., unique in time and space. The definition of this interface <u>[ILookup.idl</u>](../Interfaces/Lookup/Lookup.idl.odc.md) in the Microsoft interface definition language (MIDL) is shown below:

[

    object,

    uuid(c4910d71-ba7d-11cd-94e8-08001701a8a3),

    pointer_default(unique)

]

interface ILookup : IUnknown

{

    import "unknwn.idl";

    HRESULT LookupByName([in] LPTSTR lpName, [out, string] WCHAR **lplpNumber);

    HRESULT LookupByNumber([in] LPTSTR lpNumber, [out, string] WCHAR **lplpName);

}

In the DTC Component Pascal, this interface is defined as an abstract extension of COM.IUnknown marked by its IID. The compiler does prohibit instantiation of objects of this type, and it does also prohibit implementation of the abstract methods, but variables of type *ILookup* can be used as interface pointers as we will see later.

    TYPE

        **ILookup*** = POINTER TO ABSTRACT RECORD

                ["{C4910D71-BA7D-11CD-94E8-08001701A8A3}"] (COM.IUnknown) END;

    PROCEDURE (this: ILookup) **LookupByName***(

        name: WinApi.PtrWSTR; OUT number: WinApi.PtrWSTR): COM.RESULT, NEW, ABSTRACT;

    PROCEDURE (this: ILookup) **LookupByNumber***(

        number: WinApi.PtrWSTR; OUT name: WinApi.PtrWSTR): COM.RESULT, NEW, ABSTRACT;

The implementation of the phone book directory interface is an extension of the abstract interface record ILookup. The phone book is represented as a linear list. The root of the list is stored as private data in the interface record. The two methods *LookupByName* and *LookupByNumber* implement a simple linear search. If no entry is found, a special error code is returned. Module *ComTools* offers auxiliary routines to convert e.g. strings from and to Windows data structures.

    TYPE

        CLookup = POINTER TO RECORD (ILookup)

            phoneBook: Entry

        END;

        Entry = POINTER TO RECORD

            next: Entry;

            name, number: ARRAY 32 OF CHAR

        END;

To allow access to instances of this COM class, a factory object needs to be provided. The *CreateInstance* method creates a new object, initializes it and returns the requested interface (in this example this might be the interface *IUnknown* or *ILookup*). Note, that the implementation of the interface is usually not exported, only the CLSID must be known in order to create new instances of this class.

The procedures  ComPhoneBook.Register and  ComPhoneBook.Unregister can be used to register a reference of the factory object in the COM library, from where it can be requested using the CLSID.

    CONST

        **CLSID*** = "{E67D346C-2A5B-11D0-ADBA-00C01500554E}";



    TYPE

        LookupFactory = POINTER TO RECORD (WinOle.IClassFactory) END;

In module *ComPhoneBookClient* the implementation of a simple client of the phone directory is given. With the Windows procedure *CoCreateInstance* a new instance of the phone directory object is created. If the creation is successful, a pointer to the interface *ILookup* is assigned to the interface pointer *phoneBook, *through which the methods *LookupByName* and *LookupByNumber* can be called.

The combo box shown in the figure below can be opened with the command

    "StdCmds.OpenToolDialog('Com/Rsrc/PhoneBook', 'PhoneBook')"

Through this mask, the phone directory can be accessed.

It is also possible to use this interface across process boundaries. For this purpose, however, marshalling code must be provided in proxy and stub objects, and the reference to the DLL which implements the proxy and stub objects must be registered in the Windows registry.

The proxy and stub code can be generated with the MIDL compiler based on the interface description in the Microsoft interface definition language. The MIDL compiler generates C source code. For the ILookup interface all necessary files are provided in the directory Com/Interfaces/Lookup. The generated DLL is a self registering DLL, i.e., you have to execute the program REGSVR32 on this DLL in order to register it.

Once the proxy and stub DLLs have been registered, it is possible to communicate across process boundaries. You can test this if you start BlackBox in two separate processes, register the factory object in one process using the command  ComPhoneBook.Register and look for phone book entries in the other process through the dialog "StdCmds.OpenToolDialog('Com/Rsrc/PhoneBook', 'PhoneBook')".

If you have DCOM installed, then the two Component Pascal programs can even run on two different machines. The parameter in the call of procedure WinOle.CoCreateInstance needs then to be adjusted.

[<u>ComPhoneBook  sources</u>](../Mod/PhoneBook.odc.md)

