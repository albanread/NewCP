<a id="7.11"></a>**ComEnum**

ComEnum is a library containing implementations for the following standard enumerators:

    - IEnumUnknown

    - IEnumString

    - IEnumFormatEtc

    - IEnumOleVerb

To use one of the enumerators, import the corresponding creation procedure and call it with your actual data.

    PROCEDURE **CreateIEnumFORMATETC** (num: INTEGER;

                                                                    IN format: ARRAY OF INTEGER;

                                                                    IN aspect, tymed: ARRAY OF SET;

                                                                    OUT enum: WinOle.IEnumFORMATETC);



Creates an *IEnumFormatEtc* enumerator with *num* entries. The enumerated entries (of type *FORMATETC*) are initialized from the values in *format[i]*, *aspect[i]*, and *tymed[i]*.



    PROCEDURE **CreateIEnumOLEVERB** (num: INTEGER;

                                                                    IN verb: ARRAY OF INTEGER;

                                                                    IN name: ARRAY OF ARRAY OF CHAR;

                                                                    IN flags, attribs: ARRAY OF SET;

                                                                    OUT enum: WinOle.IEnumOLEVERB);



Creates an *IEnumOleVerb* enumerator with *num* entries. The enumerated entries (of type *OLEVERB*) are initialized from the values in *verb[i]*, *name[i]*, *flags[i]*, and *attribs[i]*.



    PROCEDURE **CreateIEnumString** (num: INTEGER;

                                                                    IN data: ARRAY OF ARRAY OF CHAR;

                                                                    OUT enum: WinOle.IEnumString);



Creates an *IEnumFormatEtc* enumerator with *num* entries. The enumerated entries (of type *WinApi.PtrWSTR*) are initialized from the strings in *data[i]*.



    PROCEDURE **CreateIEnumUnknown** (num: INTEGER;

                                                                    IN data: ARRAY OF COM.IUnknown;

                                                                    OUT enum: WinOle.IEnumUnknown);



Creates an *IEnumFormatEtc* enumerator with *num* entries. The enumerated entries (of type *IUnknown*) are initialized from the values in *data[i]*.

Alternatively, the source code of the module can be used as a starting point for customized implementations or implementations of other enumerators.

<u>[ComEnum  sources</u>](../Mod/Enum.odc.md)

