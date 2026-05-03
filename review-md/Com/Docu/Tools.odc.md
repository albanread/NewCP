<a id="7.10"></a>**ComTools**

ComTools is a library containing routines for the handling of often used and partially unsafe data structures. These data structures are unsafe because they contain C unions or C pointers (not interface pointers) or both.

ComTools can be imported as a tool package or its source can be used as starting point for customized implementations.

*String handling*

Strings are pointers to arrays of unspecified length. They must be allocated and deallocated manually.

    PROCEDURE **NewString** (IN str: ARRAY [untagged] OF CHAR): WinApi.PtrWSTR;

    PROCEDURE **NewEmptyString** (length: INTEGER): WinApi.PtrWSTR;

    PROCEDURE **FreeString** (VAR p: WinApi.PtrWSTR);

    PROCEDURE **NewSString** (IN str: ARRAY [untagged] OF SHORTCHAR): WinApi.PtrSTR;

    PROCEDURE **NewEmptySString** (length: INTEGER): WinApi.PtrSTR;

    PROCEDURE **FreeSString** (VAR p: WinApi.PtrSTR);

*STGMEDIUM*

The STGMEDIUM structure contains a C union and various pointers. The following procedures can be used to properly initialize one of the seven possible STGMEDIUM variants:

    PROCEDURE **GenBitmapMedium** (bitmap: WinApi.HBITMAP; unk: COM.IUnknown;

                                                                VAR sm: WinOle.STGMEDIUM);

    PROCEDURE **GenEMetafileMedium** (emf: WinApi.HENHMETAFILE; unk: COM.IUnknown;

                                                                VAR sm: WinOle.STGMEDIUM);

    PROCEDURE **GenFileMedium** (name: ARRAY OF CHAR; unk: COM.IUnknown;

                                                                VAR sm: WinOle.STGMEDIUM);

    PROCEDURE **GenGlobalMedium** (hg: WinApi.HGLOBAL; unk: COM.IUnknown;

                                                                VAR sm: WinOle.STGMEDIUM);

    PROCEDURE **GenMetafileMedium** (mf: WinApi.HMETAFILEPICT; unk: COM.IUnknown;

                                                                VAR sm: WinOle.STGMEDIUM);

    PROCEDURE **GenStorageMedium** (stg: WinOle.IStorage; unk: COM.IUnknown;

                                                                VAR sm: WinOle.STGMEDIUM);

    PROCEDURE **GenStreamMedium** (stm: WinOle.IStream; unk: COM.IUnknown;

                                                                VAR sm: WinOle.STGMEDIUM);

Safe access to the variable part of the structure should be done by one of these functions:



    PROCEDURE **MediumBitmap** (IN sm: WinOle.STGMEDIUM): WinApi.HBITMAP;

    PROCEDURE **MediumEnhMetafile** (IN sm: WinOle.STGMEDIUM): WinApi.HENHMETAFILE;

    PROCEDURE **MediumFile** (IN sm: WinOle.STGMEDIUM): WinApi.PtrWSTR;

    PROCEDURE **MediumGlobal** (IN sm: WinOle.STGMEDIUM): WinApi.HGLOBAL;

    PROCEDURE **MediumMetafile** (IN sm: WinOle.STGMEDIUM): WinApi.HMETAFILEPICT;

    PROCEDURE **MediumStorage** (IN sm: WinOle.STGMEDIUM): WinOle.IStorage;

    PROCEDURE **MediumStream** (IN sm: WinOle.STGMEDIUM): WinOle.IStream;



STGMEDIUM structures must be released manually. Use the OLE library procedure WinOle.ReleaseStgMedium for that purpose:

    PROCEDURE **ReleaseStgMedium** (VAR [nil] sm: WinOle.STGMEDIUM);

*FORMATETC generation*



This procedure can be used to quickly initialize a FORMATETC structure.

    PROCEDURE **GenFormatEtc** (format: SHORTINT; aspect, tymed: SET; OUT f: WinOle.FORMATETC);



<u>[ComTools  sources</u>](../Mod/Tools.odc.md)

