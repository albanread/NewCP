<a id="7.9"></a>**ComObject Example**

This example implements a simple OLE server which provides the following OLE object:

This object implements the four interfaces *IUnknown, IOleObject, IDataObject *and* IPersistStorage *and uses several other interfaces. Beside of this, the module also implements a factory object which can be registered and which provides the creation of new objects of the above type.

In order to implement the four interfaces, four Component Pascal records are defined as subtypes of the particular interface records. These objects refer to each other in the following way:

The* *record which implements the *IUnknown* interface contains the object data. The references to the other interface records are necessary in order to implement the *QueryInterface* method. The other interfaces need a reference to the *IUnknown* object to access the object data as well as to implement their *QueryInterface*.

Note that other definitions would be possible also. Instead of such a symmetric approach, the interface *IOleObject* could play the role of the main interface and administrate the object's instance variables.

        Object = POINTER TO RECORD (COM.IUnknown)

            ioo: IOleObject;

            ido: IDataObject;

            ips: IPersistStorage;

            *(* and other fields *)*

        END;

        IOleObject = POINTER TO RECORD (WinOle.IOleObject)

            obj: Object

        END;

        IDataObject = POINTER TO RECORD (WinOle.IDataObject)

            obj: Object

        END;

        IPersistStorage = POINTER TO RECORD (WinOle.IPersistStorage)

            obj: Object

        END;

*Object creation*

New objects are created through the method *CreateInstance* of the class factory. This method allocates the objects, defines the relation between the involved interfaces and returns a pointer to the requested interface.

The implementation of the *QueryInterface* method of the three interfaces *IOleObject*, *IDataObject* and *IPersistStorage* is simplified by using hidden references to the main record. These hidden references are generated with the two-argument NEW.

    VAR new: Object;

    NEW(new); NEW(new.ioo, new); NEW(new.ido, new); NEW(new.ips, new);

    new.ioo.obj := new; new.ido.obj := new; new.ips.obj := new;

All *QueryInterface * calls are thus forwarded to the *QueryInterface* method of *Object.* The explicit references to *Object* are only used to access the instance variables of the object. Instance variables which would be used by the implementation of one interface only could also be stored in the corresponding record directly.

If one of the interfaces *IOleObject*, *IDataObject* or *IPersistStorage* is requested through *QueryInterface*, then the reference count of the particular record as well as the reference count of the main object is incremented. The latter one is incremented since the requested interface contains a (hidden) reference to the main object.

As an alternative, the four records could also be allocated with the one-argument NEW, but then *QueryInterface *calls would have to be forwarded to the main object by hand for the implementations of the interfaces *IOleObject, IDataObject *and* IPersistStorage. *The following procedure shows how that would be done.

    PROCEDURE (this: IOleObject) QueryInterface (VAR iid: COM.GUID;

                                                                                    VAR int: COM.IUnknown): COM.RESULT;

    BEGIN RETURN this.obj.QueryInterface(iid, int)

    END QueryInterface;



*QueryInterface*

The second interesting aspect is the implementation of *QueryInterface *of *Object. *For all the interfaces that the object supports, the COM.QUERY function is called one by one. If the *IUnknown *interface is requested, then always the same interface pointer is returned, although all involved interfaces support the *IUnknown* methods. This is a requirement of the COM specification.

    PROCEDURE (this: Object) QueryInterface (IN iid: COM.GUID; OUT int: COM.IUnknown): COM.RESULT;

    BEGIN

        IF COM.QUERY(this, iid, int)

            OR COM.QUERY(this.ioo, iid, int)

            OR COM.QUERY(this.ido, iid, int)

            OR COM.QUERY(this.ips, iid, int) THEN RETURN WinApi.S_OK

        ELSE RETURN WinApi.E_NOINTERFACE

        END

*Registry*

With the command *ComObject.Register* the factory can be registered. As soon as it is registered it can be requested with a call to the procedure *CoGetClassObject* (it has to be requested from a local server, namely the running BlackBox).

    VAR factory: WinOle.IClassFactory; res: COM.RESULT;

        res := WinOle.CoGetClassObject("{00010001-1000-11cf-adf0-444553540000}",

                                                            WinOle.CLSCTX_LOCAL_SERVER, 0,

                                                            COM.ID(factory), factory)

If the object is also registered in the Windows registry, then the object can be inserted in any OLE container using the Insert Object menu entry, provided that the factory has been registered using the command *ComObject.Register.* The necessary registry file [<u>Com/Reg/Object.reg</u>](../Reg/Object.reg.odc.md) is given below. Use the REGEDIT tool to update the registry. Adjust path names if necessary! Note that you cannot insert it into a BlackBox document of the BlackBox which is the server for this object, because then the object would be inserted in-process, and thus the in-process handler would not be used. However, you can insert this object in a document of another running BlackBox.

Once the smiley object has successfully been inserted into an OLE container, you can inspect with the interface inspector how many interfaces the client has requested.

REGEDIT

HKEY_CLASSES_ROOT\BlackBox.Object = BlackBox Object

HKEY_CLASSES_ROOT\BlackBox.Object\CLSID = {00010001-1000-11cf-adf0-444553540000}

HKEY_CLASSES_ROOT\BlackBox.Object\Insertable

HKEY_CLASSES_ROOT\CLSID\{00010001-1000-11cf-adf0-444553540000} = BlackBox Object

HKEY_CLASSES_ROOT\CLSID\{00010001-1000-11cf-adf0-444553540000}\ProgID = BlackBox.Object

HKEY_CLASSES_ROOT\CLSID\{00010001-1000-11cf-adf0-444553540000}\LocalServer32 = C:\BlackBox\BlackBox.exe

HKEY_CLASSES_ROOT\CLSID\{00010001-1000-11cf-adf0-444553540000}\InProcHandler32 = ole32.dll

HKEY_CLASSES_ROOT\CLSID\{00010001-1000-11cf-adf0-444553540000}\Insertable

HKEY_CLASSES_ROOT\CLSID\{00010001-1000-11cf-adf0-444553540000}\DefaultIcon = C:\BlackBox\BlackBox.exe,0

HKEY_CLASSES_ROOT\CLSID\{00010001-1000-11cf-adf0-444553540000}\DataFormats\GetSet\0 = 3,1,32,1

HKEY_CLASSES_ROOT\CLSID\{00010001-1000-11cf-adf0-444553540000}\MiscStatus = 16

HKEY_CLASSES_ROOT\CLSID\{00010001-1000-11cf-adf0-444553540000}\AuxUserType\2 = BlackBox Object

HKEY_CLASSES_ROOT\CLSID\{00010001-1000-11cf-adf0-444553540000}\AuxUserType\3 = BlackBox

It would also be possible to generate a new executable which acts as server for this BlackBox object. You find a similar example in ComKoalaExe. In particular, the main event loop has to be implemented.

If you intend to link this object implementation into a DLL, then the additional interfaces *IViewObject2*, *IOleCache2*, and *IRunnableObject* would have to be implemented.

[<u>ComObject  sources</u>](../Mod/Object.odc.md)

