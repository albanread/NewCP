<a id="7.5"></a>**ComPhoneBookActiveX Example**

In this example a simple ActiveX control is implemented. The control offers the same functionality as the phone book in the ComPhoneBook example, but in this example, clients communicate through a dispatch interface. The control will be linked into a DLL and can be used as an ActiveX control from any client which properly supports the ActiveX standard.

The dispatch interface also has to be described in a MIDL definition. This definition is stored in file <u>[Phone.idl</u>](../Interfaces/DPhoneBook/phone.idl.odc.md) and shown below:

[

  uuid(C4910D73-BA7D-11CD-94E8-08001701A8A3),

  version(1.0),

  helpstring("PhoneBook ActiveX Control")

]

library PhoneBookLib {

    importlib("stdole32.tlb");

    [

      uuid(C4910D72-BA7D-11CD-94E8-08001701A8A3),

      helpstring("Dispatch interface for PhoneBook ActiveX Control")

    ]

    dispinterface DPhoneBook {

        properties:

        methods:

            [id(1)]

            BSTR LookupByName(BSTR name);

            [id(2)]

            BSTR LookupByNumber(BSTR number);

    };

    [

      uuid(E67D346B-2A5B-11D0-ADBA-00C01500554E),

      helpstring("PhoneBook ActiveX  Control")

    ]

    coclass PhoneBook {

        [default] dispinterface DPhoneBook;

    };

}

With the MIDL compiler, a type library is generated. Once this library has been registered in the Windows registry it can be inspected with the OLE ITypeLib Viewer.

In DTC Component Pascal, this interface is defined as an abstract extension of WinOleAut.IDispatch, the base type of all dispatch interface. This interface contains the following four methods which must be implemented:

    TYPE

        IDispatch = POINTER TO ABSTRACT RECORD ["{00020400-0000-0000-C000-000000000046}"] (COM.IUnknown)

            (this: IDispatch) **GetIDsOfNames** (IN [nil] riid: COM.GUID; IN [nil] rgszNames: WinApi.PtrWSTR; cNames: INTEGER;

                                lcid: WinOle.LCID; OUT [nil] rgDispId: WinOleAut.DISPID): COM.RESULT, NEW, ABSTRACT;

            (this: IDispatch) **GetTypeInfo** (iTInfo: INTEGER; lcid: WinOle.LCID;

                                OUT [nil] ppTInfo: WinOleAut.ITypeInfo): COM.RESULT, NEW, ABSTRACT;

            (this: IDispatch) **GetTypeInfoCount** (OUT [nil] pctinfo: INTEGER): COM.RESULT, NEW, ABSTRACT;

            (this: IDispatch) **Invoke** (dispIdMember: WinOleAut.DISPID; IN riid: COM.GUID; lcid: WinOle.LCID; wFlags: SHORTINT;

                                VAR [nil] pDispParams: WinOleAut.DISPPARAMS; OUT [nil] pVarResult: WinOleAut.VARIANT;

                                OUT [nil] pExcepInfo: WinOleAut.EXCEPINFO; OUT [nil] puArgErr: INTEGER): COM.RESULT, NEW, ABSTRACT

        END;

The implementation of this interface is a concrete extension of the abstract dispatch interface. The main method is the Invoke method. Through this method the functions of the interface are called. The number of the function to be invoced is passed in parameter dispIdMember, and the parameters are passed in the array pDispParams as variant records. The result is returned through parameter pVarResult, a variant record as well. The other three methods of this interface provide information about the interface itself. They can easily be implemented if a type library is available.

Besides the implementation of the dispatch interface, a factory object must be provided. Furthermore, the two global functions DllGetClassObject and DllCanUnloadNow have to be implemented. These two routines are called by the COM library. If the DLL is loaded it is asked to return a reference to the factory object, and with the function DllCanUnloadNow the COM library tests whether the DLL can be unloaded.

If module ComPhoneBookActiveX has been compiled, a DLL can be linked with the command

     DevLinker.LinkDll "Com/phone.dll" := Kernel+ ComTools ComPhoneBookActiveX# ~

This command generates a DLL with a size of 40KBytes only.

In order to load this DLL it must be registered in the Windows registry. Note, that the CLSID for this implementation is different than the CLSID of example ComPhoneBook. Besides the CLSID, the information about the type library must be stored in the registry, and in particular the CLSID entry must contain a reference to the GUID under which the type library is referenced. The necessary registry file [<u>Com/Interfaces/DPhoneBook/phone.reg</u>](../Interfaces/DPhoneBook/phone.reg.odc.md) is given below. Use the REGEDIT tool to update the registry. Adjust path names if necessary!

REGEDIT

HKEY_CLASSES_ROOT\PhoneBook = PhoneBook ActiveX Control

HKEY_CLASSES_ROOT\PhoneBook\CLSID = {E67D346B-2A5B-11D0-ADBA-00C01500554E}

HKEY_CLASSES_ROOT\PhoneBook\TypeLib = {C4910D73-BA7D-11CD-94E8-08001701A8A3}

HKEY_CLASSES_ROOT\CLSID\{E67D346B-2A5B-11D0-ADBA-00C01500554E} = PhoneBook ActiveX Control

HKEY_CLASSES_ROOT\CLSID\{E67D346B-2A5B-11D0-ADBA-00C01500554E}\ProgID = PhoneBook1.0

HKEY_CLASSES_ROOT\CLSID\{E67D346B-2A5B-11D0-ADBA-00C01500554E}\Control

HKEY_CLASSES_ROOT\CLSID\{E67D346B-2A5B-11D0-ADBA-00C01500554E}\Version = 1.0

HKEY_CLASSES_ROOT\CLSID\{E67D346B-2A5B-11D0-ADBA-00C01500554E}\VersionIndependentProgID = PhoneBook

HKEY_CLASSES_ROOT\CLSID\{E67D346B-2A5B-11D0-ADBA-00C01500554E}\TypeLib =

                                                                                                         {C4910D73-BA7D-11CD-94E8-08001701A8A3}

HKEY_CLASSES_ROOT\CLSID\{E67D346B-2A5B-11D0-ADBA-00C01500554E}\InprocServer32 = C:\BlackBox\Com\phone.dll

HKEY_CLASSES_ROOT\CLSID\{E67D346B-2A5B-11D0-ADBA-00C01500554E}\NotInsertable

HKEY_CLASSES_ROOT\TypeLib\{C4910D73-BA7D-11CD-94E8-08001701A8A3}\1.0 = PhoneBook ActiveX Control

HKEY_CLASSES_ROOT\TypeLib\{C4910D73-BA7D-11CD-94E8-08001701A8A3}\1.0\0\win32 =

                                                                                                         C:\BlackBox\Com\Interfaces\DPhoneBook\phone.tlb

HKEY_CLASSES_ROOT\TypeLib\{C4910D73-BA7D-11CD-94E8-08001701A8A3}\1.0\FLAGS = 0

HKEY_CLASSES_ROOT\TypeLib\{C4910D73-BA7D-11CD-94E8-08001701A8A3}\1.0\HELPDIR =

                                                                                                         C:\BlackBox\Com\Interfaces\DPhoneBook

Once this information has been stored, the phonebook ActiveX control is ready to be used. As an example we demonstrate how it can be used through the Internet Explorer. With the <OBJECT> tag an ActiveX control can be embedded in any HTML page. As an example the html page [<u>Com/Rsrc/phone.html</u>](../Rsrc/phone.html.odc.md) is provided. Note, that the only reference to the ActiveX control is its CLSID. The phone book is accessed in the two VBScript commands PhoneBook.LookupByName(NameField.value) and PhoneBook.LookupByNumber(NameField.value).

<HTML>

<HEAD>

<TITLE>ActiveX Demo - Phonebook</TITLE>

</HEAD>

<BODY>

<H1>BlackBox Phone Book Example</H1>

<OBJECT

  **CLASSID="CLSID:E67D346B-2A5B-11D0-ADBA-00C01500554E"**

  ID=PhoneBook

>

</OBJECT>

This page uses a small (40K) non-visual control which provides access to a phone book which also might run on a remote computer.

This ActiveX control has been implemented with the DTC Component Pascal compiler.

<P>

Enter a name or a phone number and press the corresponding button:

<P>

<INPUT   TYPE="text"   NAME = "NameField"   SIZE=20>

<INPUT   TYPE=BUTTON VALUE="Lookup by Name"   NAME="NameBtn" >

<INPUT   TYPE=BUTTON VALUE="Lookup by Number"   NAME="NumberBtn">

<SCRIPT LANGUAGE=VBScript>

Sub NameBtn_Onclick

  alert "Phone number of " & NameField.value & ":    " & **PhoneBook.LookupByName(NameField.value) **

End Sub

Sub NumberBtn_Onclick

  alert "Name of line " & NameField.value & ":    " & **PhoneBook.LookupByNumber(NameField.value) **

End Sub

</SCRIPT>

</BODY>

</HTML>

The following figure shows a screen dump of Internet Explorer if this html page is inspected.

[<u>ComPhoneBookActiveX  sources</u>](../Mod/PhoneBookActiveX.odc.md)

