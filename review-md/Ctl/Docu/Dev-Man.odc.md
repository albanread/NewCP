** BlackBox**

**Developer Manual**

**Ctl Subsystem**

**Introduction**

The Ctl subsystem contains tools for writing OLE automation controllers in BlackBox. OLE Automation is a standard for controlling objects implemented in one application (server) from within another application (controller). The interface of a server application is described in a type library. A type library specifies the objects provided by the application and the operations on these objects.

To write a controller in BlackBox, an interface module is used which contains a Component Pascal object for each object in the type library. These objects can be used like other Component Pascal objects in a convenient and typesafe way and hide the details of the automation standard used to communicate with the server application.

**Elements of an interface module**

An automation interface module declares the following elements:

ꀢ Constants

Constant declarations correspond to constant declarations in the type library. They are usually used as values for integer parameters of methods.

ꀢ Alias Types

Alias Types can be declared for all legal automation types (see below).

ꀢ Object Types

Object types are the main elements of an automation interface module. In general, automation objects can have methods and properties (fields). In BlackBox both methods and properties are implemented as methods.

For a property with name *Prop* and type *Type* two methods are declared:

PROCEDURE (this: Object) Prop (): Type;

PROCEDURE (this: Object) PUTProp (val: Type);

The latter is omitted if the property is read-only. The type of a property or method parameter can be any legal automation type (see below).

ꀢ Procedures

For an object type with name *Object*, the following procedures are declared:

PROCEDURE IsObject (v: CtlT.Any): BOOLEAN;

PROCEDURE ThisObject (v: CtlT.Any): Object;

The two serve as type-test and type-guard for automation objects. These procedures can be used in places where otherwise one would call QueryInterface to retrieve the IDispatch interface of an object.

For objects marked as *creatable* in the type library an additional procedure is used for the allocation of a new object of that type:

PROCEDURE NewObject (): Object;

**Legal data types**

Because OLE Automation is a standard used to communicate between applications potentially written in different languages, a restricted set of legal data types must be used.

Basic Types:

BOOLEAN        FALSE or TRUE

BYTE        1 byte

SHORTINT        2 bytes

INTEGER        4 bytes

SHORTREAL        4 bytes

REAL        8 bytes

CtlT.OleCy    (=LONGINT)    8 bytes, scaled by 10000

CtlT.Date    (=REAL)    date & time as a real value, see [<u>CtlT</u>](T.odc.md) for conversion

ARRAY OF CHAR        Unicode strings

CtlT.RESULT    (=INTEGER)    OLE result values (4 bytes)

CtlT.IUnknown        OLE Interfaces (usually not used)

Object Types:

Object types are declared as record extensions of the type *CtlT.Object*.

Arrays:

In Ole Automation, multidimensional arrays of all types above can be used. Only one- and two-dimensional array are supported in BlackBox.

Unspecified Type:

In OLE Automation there is a type covering all other legal automation types. It is used when a property or parameter is not statically typed at all. The type used in BlackBox for this purpose is *CtlT.Any*. The type *CtlT.Object* is an extension of *CtlT.Any*. Because basic types cannot be extensions of such a type in Component Pascal, a record type which is an extension of *CtlT.Any* is provided for each basic and array type. See [<u>CtlT</u>](T.odc.md) for details.

**Example**

The following is a short example relating plain OLE programming to the BlackBox approach.

To open a document in word, one has first to create an object for the application. Second, from this object a list of documents can be retrieved. Third, to this list a new document can be added. Below are the first two steps formulated each in C++ and in Component Pascal using the [<u>Word9</u>](Word9.odc.md) interface module.

Creating a new application, C++ version:

        ::CoInitialize(NULL);

        CLSID clsid;

        CLSIDFromProgID(L"Word.Application", &clsid);

        IUnknown* pUnk;

        HRESULT hr = ::CoCreateInstance(clsid, NULL, CLSCTX_SERVER,

                                        IID_IUnknown, (void**) &pUnk);

        IDispatch* pDispApp;

        hr = pUnk->QueryInterface(IID_IDispatch, (void**)&pDispApp);

Creating a new application, BlackBox version:

            VAR app: CtlWord9._Application;

        ...

            app := CtlWord9.NewApplication()

Retrieving the list of documents, C++ version:

        DISPPARAMS dpNoArgs = {NULL, NULL, 0, 0};

        VARIANT vResult;

        OLECHAR FAR* szFunction;

        IDispatch* pDispDocs;

        DISPID dispid_Docs;

        szFunction = OLESTR("Documents");

        hr = pDispApp->GetIDsOfNames (IID_NULL, &szFunction, 1,

                                       LOCALE_USER_DEFAULT, &dispid_Docs);

        hr = pDispApp->Invoke (dispid_Docs, IID_NULL,

                               LOCALE_USER_DEFAULT, DISPATCH_PROPERTYGET,

                               &dpNoArgs, &vResult, NULL, NULL);

        pDispDocs = vResult.pdispVal;

Retrieving the list of documents, BlackBox version:

            VAR docs: CtlWord9.Documents;

        ...

            docs := app.Documents()

**Callback interfaces**

In contrast to normal interfaces, a callback interface is implemented by the controller application and called by the server. In an automation interface, module callback interfaces are declared as abstract objects which are extensions of *CtlT.OutObject*. Such objects must be implemented in the controlling program and connected to the corresponding server object. Connection is done using *CtlT.Connect* for normal automation objects or *OleViews.Connect* for a server object contained in a BlackBox View.

**Error Handling**

There are two general error sources: An automation method call can fail because of an exception in the server application, or because of an out-of-memory situation. An exception in the server raises a trap 10 in the automation interface module. Two string variables visible in the trap window, namely *source* and *description*,* *show the available information about where and why the exception occurred. A third variable *param* holds the number of the parameter if the error belongs to a parameter value. An out-of-memory condition leads to a trap 11.

**Available Automation interfaces**

The following table contains all automation interfaces currently available in BlackBox. The modules were generated automatically from the corresponding type libraries. The semantics of the objects contained in these modules are explained in the corresponding help files which are all contained on the MS Office CD but are not installed by default.

*Module Name    Controlled Application    Help File*

CtlExcel5    MS Excel 5.0    VBA_XL.HLP

CtlExcel8*    MS Excel 8.0    VBAXL8.HLP

CtlExcel9    MS Excel 9.0    VBAXL9.CHM

CtlWord8*    MS Word 8.0    VBAWRD8.HLP

CtlWord9    MS Word 9.0    VBAWRD9.CHM

CtlOutlook8*    MS Outlook 8.0    VBAOUTL.HLP

CtlOutlook9    MS Outlook 9.0    VBAOUTL9.CHM

CtlPowerPoint8*    MS PowerPoint 8.0    VBAPPT8.HLP

CtlPowerPoint9    MS PowerPoint 9.0    VBAPPT9.CHM

CtlAccess8*    MS Access 8.0    ACVBA80.HLP

CtlAccess9    MS Access 9.0    ACMAIN9.CHM

CtlGraph8*    MS Graph 8.0    VBAGRP8.HLP

CtlGraph9    MS Graph 9.0    VBAGRP9.CHM

CtlOffice    MS Office 9.0    VBAOFF9.CHM / VBAOFF8.HLP

CtlOfficeBinder    MS Binder 9.0    VBABDR8.HLP

CtlMSForms    MS Forms 2.0    FM20.HLP

CtlDAO35*    MS Data Access Objects 3.5    DAO35.HLP

CtlDAO36    MS Data Access Objects 3.6    DAO36.HLP

CtlADODB    MS ActiveX Data Objects 2.0    -

CtlVBIDE    MS Visual Basic    VEENOB3.HLP

CtlStdType    Standard OLE Types    -

The modules marked with * used to be without a number in the previous release of BlackBox. Because Microsoft changed the interfaces of these products for Office 9.0, the names of the modules have been changed accordingly so that users can choose which interface to use.

CtlVBIDE and CtlStdType contain types imported by other modules and are usually not used directly.

