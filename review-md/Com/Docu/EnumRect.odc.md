<a id="7.6"></a>**ComEnumRect Example**

This is a simple example adapted form the same example in the book "Inside OLE".

The interface *IEnumRECT* (an enumerator for rectangles) is defined and implemented in this module.

The interface consists of the following four methods:

    IEnumRECT = POINTER TO ABSTRACT RECORD

                                             ["{00021140-0000-0000-C000-000000000046}"] (COM.IUnknown)

        (this: IEnumRECT) Next (num: INTEGER; OUT elem: ARRAY [untagged] OF WinApi.RECT;

                                                    OUT [nil] fetched: INTEGER): COM.RESULT, NEW, ABSTRACT;

        (this: IEnumRECT) Skip (num: INTEGER): COM.RESULT, NEW, ABSTRACT

        (this: IEnumRECT) Reset (): COM.RESULT, NEW, ABSTRACT;

        (this: IEnumRECT) Clone (OUT enum: IEnumRECT): COM.RESULT, NEW, ABSTRACT;

    END;

For the implementation, an array of length 15 is used.

As the interface is implemented in one record (*EnumRECT*), the default implementation for *QueryInterface* can be used.

<u>[ComEnumRect  sources</u>](../Mod/EnumRect.odc.md)



