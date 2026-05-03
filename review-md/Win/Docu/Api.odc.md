**Windows Programming Interfaces**

The *Win* subsystem contains the interface modules for the various Windows programming interfaces. The following table shows the available modules and a short desription of their contents:

*module*    *contents*

WinApi    basic data types, error codes, basic Win32 functionality

WinDlg    common dialog box library

WinCtl    common controls

WinOle    basic COM and OLE interfaces

WinOleDlg    OLE dialogs (OleUI...)

WinOleAut    OLE automation interfaces

WinOleCtl    OLE controls interfaces

WinRpc    remote procedure call functions

WinNet    networking and socket functions

WinMM    multimedia services

WinCmc    Messaging Application Programming Interface (MAPI)

WinSql    Database services (ODBC)

**Naming Conventions**

Names are generally the same as in the corresponding C header and help files. An exception applies to pointers which are always named after the structure they point to with a "Ptr" prefix.

Example:

RECT = RECORD [untagged] left, top, right, bottom: INTEGER END;

PtrRECT = POINTER TO RECT;

**Parameters**

Parameters of pointer type are represented as variable parameters (VAR/IN/OUT) whenever possible.

**Pointers to Pointers**

Pointers to pointers or basic types cannot be declared directly in Component Pascal. A type of the form

p: POINTER TO ARRAY [untagged] OF BaseType;

is used instead. You can use the form p[0] to dereference such a pointer.

