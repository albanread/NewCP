**Platform-Specific Issues (Windows)**

[<u>Module SYSTEM</u>](#1)

[<u>Using DLLs in BlackBox modules</u>](#2)

[<u>Using COM without special Direct-To-COM compiler</u>](#Using COM)

[<u>Windows programming interfaces</u>](../../Win/Docu/Api.odc.md)

[<u>OLE Automation</u>](../../Ctl/Docu/Sys-Map.odc.md)

[<u>Windows-specific information in BlackBox</u>](#3)

[<u>Differences between different Windows versions</u>](#4)

[<u>The BlackBox linker</u>](Linker.odc.md)

[<u>Linking BlackBox applications</u>](#6)

[<u>Startup of BlackBox</u>](#7)

<a id="1"></a>**Module SYSTEM**

Module *SYSTEM* contains certain procedures that are necessary to implement low-level operations. It is strongly recommended to restrict the use of these features to specific low-level modules, as such modules are inherently non-portable and usually unsafe. *SYSTEM* is not considered as part of the language Component Pascal proper.

The procedures contained in module *SYSTEM* are listed in the following table. *v* stands for a variable.* x*, *y*, and *n* stands for expressions. *T* stands for a type. *P* stands for a procedure. *M[a]* stands for memory value at address *a*.

Function procedures

*    Name    Argument types    Result type    Description*

    ADR(v)    any    INTEGER    address of variable v

    ADR(P)    P: PROCEDURE    INTEGER    address of Procedure P

    ADR(T)    T: a record type    INTEGER    address of Descriptor of T

    LSH(x, n)    x, n: integer type*    type of x    logical shift (n > 0: left, n < 0: right)

    ROT(x, n)    x, n: integer type*    type of x    rotation (n > 0: left, n < 0: right)

    TYP(v)    record type    INTEGER    type tag of record variable v

    VAL(T, x)    T, x: any type    T    x interpreted as of type T

    * integer types without LONGINT

Proper procedures

*    Name    Argument types    Description*

    GET(a, v)    a: INTEGER; v: any basic type,    v := M[a]

        pointer type, procedure type

    PUT(a, x)    a: INTEGER; x: any basic type,    M[a] := x

        pointer type, procedure type

    GETREG(n, v)    n: integer constant, v: any basic type,    v := Register n

        pointer type, procedure type

    PUTREG(n, x)    n: integer constant, x: any basic type,    Register n := x

        pointer type, procedure type

    MOVE(a0, a1, n)    a0, a1: INTEGER; n: integer type    M[a1..a1+n-1] :=

            M[a0..a0+n-1]

The register numbers for PUTREG and GETREG are:

0: EAX, 1: ECX, 2: EDX, 3: EBX, 4: ESP, 5: EBP, 6: ESI, 7: EDI.

**Warning**

*VAL*, *PUT*, *PUTREG*, and *MOVE* may crash BlackBox and/or Windows when not used properly.

Never use *VAL* (or *PUT* or *MOVE*) to assign a value to a BlackBox pointer. Doing this would corrupt the garbage collector, with fatal consequences.

*System Flags*

The import of module *SYSTEM* allows to override some default behavior of the compiler by the usage of system flags. System flags are used to configure type- and procedure declarations. The extended syntax is given below.

Type     =    Qualident

        | ARRAY **["[" SysFlag "]"]** [ConstExpr {"," ConstExpr}]

            OF Type

        | RECORD** ["[" SysFlag "]"]** ["("Qualident")"] FieldList

            {";" FieldList} END

        | POINTER **["[" SysFlag "]"]** TO Type

        | PROCEDURE [FormalPars].

ProcDecl    =    PROCEDURE** ["[" SysFlag "]"]** [Receiver] IdentDef

            [FormalPars] ";"

        DeclSeq [BEGIN StatementSeq] END ident.

FPSection    =    [VAR **["[" SysFlag "]"]**] ident {"," ident} ":" Type.

**SysFlag    =    ConstExpr | ident.**

For *SysFlags* either the name of the flag or the corresponding numeric value can be used.

System flags for record types

*Name    Value    Description*

untagged    1    No type tag and no type descriptor is allocated.

        The garbage collector ignores untagged variables.

*        NEW* is not allowed on pointers to untagged variables.

        No type-bound procedures are allowed for the record.

        Pointers to untagged record type or extensions of this

        record type inherit the attribute of being untagged.

        The offsets of the fields are aligned to

        MIN(4-byte, size), where size is the size of the field.

        The size of the record and the offsets of the fields are

        aligned to 32-bit boundaries.

noalign    3    Same as *untagged *but without alignment.

align2    4    Same as *untagged *but with

        MIN(2-byte, size) alignment.

align8    6    Same as *untagged *but with

        MIN(8-byte, size) alignment.

union    7    Untagged record with all fields allocated at offset 0.

        The size of the record is equal to the size of the

        largest field.

        Used to emulate C union types.

System flags for array types

*Name    Value    Description*

untagged    1    No typetag and no type descriptor is allocated.

        The garbage collector ignores untagged variables.

*        NEW* is not allowed on pointers to untagged variables.

        Pointers to this array type inherit the attribute of

        being untagged.

        Only one-dimensional untagged open arrays

        are allowed.

        For open untagged arrays, index bounds are

        not checked.

System flags for pointer types

*Name    Value    Description*

untagged    1    Not traced by the garbage collector.

        No type-bound procedures are allowed for the pointer.

        Must point to an untagged record.

System flags for *VAR* parameters

*Name    Value    Description*

nil    1    *NIL* is accepted as formal parameter.

        Used in interfaces to C

        functions with pointer type parameters.

System flags for procedures

*Name    Value    Description*

code    1    Definition of a Code procedure (see below).

ccall    -10    Procedure uses CCall calling convention.

*Code procedures*

Code procedures make it possible to use special code sequences not generated by the compiler. They are declared using the following special syntax:

ProcDecl    =    PROCEDURE "[" SysFlag "]" IdentDef [FormalPars]

            [ConstExpr {"," ConstExpr}] ";".

The list of constants declared with the procedure is interpreted as a byte string and directly inserted in the code ("in-line") whenever the procedure is called. If a parameter list is supplied, the actual parameters are pushed on the stack from right to left. The first parameter however is kept in a register. If the type of the first parameter is *REAL* or *SHORTREAL*, it is stored in the top floating-point register. Otherwise the parameter (or in the case of a *VAR/IN/OUT* parameter its address) is loaded into EAX. For function procedures the result is also expected to be either in the top floating-point register or in EAX, depending on its type. Be careful when using registers in code procedures. In general, the registers ECX, EDX, ESI, and EDI may be used. Parameters on the stack must be removed by the procedure.

*Examples*

    PROCEDURE [code] Sqrt (x: REAL): REAL        (* Math.Sqrt *)

        0D9H, 0FAH;            (* FSQRT *)

    PROCEDURE [code] Erase (adr, words: INTEGER)    (* erase memory area *)

        089H, 0C7H,            (* MOV EDI, EAX *)

        031H, 0C0H,            (* XOR EAX, EAX *)

        059H,            (* POP ECX *)

        0F2H, 0ABH;            (* REP STOS *)

<a id="2"></a>**Using DLLs in BlackBox modules**

Any 32-bit DLL can be imported in Component Pascal like a normal Component Pascal module. This holds for Windows system modules (kernel, user, gdi, ...) as well as for any custom DLL written in any programming language. Be aware that the safety qualities of Component Pascal (no dangling pointers, strict type-checking, etc.) are lost if you interface to a DLL written in another programming language.

*Interface modules*

Type information about objects imported from a DLL must be present in a symbol file for the compiler to work properly. Such special symbol files can be generated through so-called *interface modules*. An interface module is a Component Pascal module marked by a system flag after the module name. The system flag consists of the name of the corresponding DLL enclosed in square brackets. An interface module can only contain declarations of constants, types, and procedures headings. No code file is generated when an interface module is compiled.

*Name aliasing*

The Component Pascal name of an object imported from a DLL can be different from the corresponding name in the export table of the DLL. To achieve this the DLL name is appended to the Component Pascal name as a system flag. In addition, the system flag may specify a different DLL than the one declared in the module header. This allows to use a single interface module for a whole set of related DLLs. It is also possible to have multiple interface modules referring to the same DLL.  Interface procedures cannot have a body, nor an "END identifier" part, they are only signatures.

The extend syntax for interface modules is given below:

Module     =    MODULE ident **["[" SysString "]"]** ";"

        [ImportList] DeclSeq  [BEGIN StatementSeq] END ident ".".

ProcDecl    =    PROCEDURE** **["[" SysFlag "]"] [Receiver]

        IdentDef **["[" [SysString ","] SysString "]"]** [FormalPars] ";"

        DeclSeq **[**[BEGIN StatementSeq] END ident**]**.

**SysString    =    string.**

There is no aliasing for types and constants, because they are not present in export lists and thus may have arbitrary names.

The following example summarizes the aliasing capabilities:

MODULE MyInterface ["MyDll"];

    PROCEDURE Proc1*;                (* Proc1 from MyDll *)

    PROCEDURE Proc2* ["BlaBla"];                (* BlaBla from MyDll *)

    PROCEDURE Proc3* ["OtherDll", ""];                (* Proc3 from OtherDll *)

    PROCEDURE Proc4* ["OtherDll", "DoIt"];                (* DoIt from OtherDll *)

END MyInterface.

A SysString for a DLL may not contain an entire path name. It should be just the name of the DLL, without the ".dll" suffix. The following search strategy is used:

- BlackBox directory

- Windows\System directory

- Windows directory

*Data types in interface modules*

Always use untagged records as replacements for C structures, in order to avoid the allocation of a type tag for the garbage collector. The system flag [untagged] marks a type as untagged (no type information is available at run-time) with standard alignment rules for record fields (2-byte fields are aligned to 2-byte boundaries, 4-byte or larger fields to 4-byte boundaries). The system flags [noalign], [align2], and [align8] also identify untagged types but with different alignments for record fields.

Like all system flags, "untagged" can only be used if module *SYSTEM* is imported.

Example:

    RECORD [noalign]    *(* untagged, size = 7 bytes *)*

        c: SHORTCHAR;    *(* offset 0, size = 1 byte *)*

        x: INTEGER;    *(* offset 1 , size = 4 bytes *)*

        i: SHORTINT    *(* offset 5, size = 2 bytes *)*

    END

*Procedures*

Component Pascal procedure calls conform to the StdCall calling convention (parameters pushed from right to left, removed by called procedure). If the CCall convention (parameters removed by caller) is needed for some DLL procedures, the corresponding procedure declaration in the interface module must be decorated with the [ccall] system flag.

No special handling is required for callback procedures.

For parameters of type *POINTER TO T* it is often better to use a variable parameter of type *T* rather than to declare a corresponding pointer type. Declare the *VAR* parameter with system flag [nil] if *NIL* must be accepted as legal actual parameter.

Example:

C:

    BOOL MoveToEx(HDC hdc, int X, int Y, LPPOINT lpPoint)

Component Pascal:

    PROCEDURE MoveToEx* (dc: Handle; x, y: INTEGER; VAR [nil] old: Point): Bool

*Correspondence between Component Pascal and C data types*

unsigned char    = SHORTCHAR    (1 byte)

WCHAR    = CHAR    (2 bytes)

signed char    = BYTE    (1 byte)

short    = SHORTINT    (2 bytes)

int    = INTEGER    (4 bytes)

long    = INTEGER    (4 bytes)

LARGE_INTEGER    = LONGINT    (8 bytes)

float    = SHORTREAL    (4 bytes)

double    = REAL    (8 bytes)

Note that Bool is not a data type in C but is defined as int (= *INTEGER*). 0 and 1 must be used for assignments of *FALSE* and *TRUE* and comparisons with 0 have to be used in conditional statements (*IF b # 0 THEN ... END* instead of *IF b THEN ... END*).

Note that it is not possible to unload a DLL from within BlackBox. To avoid having to exit and restart your development environment repeatedly, it is a good idea to test the DLL that you are developing from within another instance of the BlackBox application.

<a id="Using COM"></a>**Using COM without special Direct-To-COM compiler**

Microsoft's Component Object Model (COM) is supported by a special Component Pascal compiler that is available as an add-on product to BlackBox. This compiler makes using COM safer and more convenient. However, for casual use of COM, the approach described in this chapter can be used. It doesn't require a special compiler version. It uses normal untagged records and procedure variables to create COM-style method tables ("vtbl") for objects. The following example shows how it works:

MODULE Ddraw ["DDRAW.DLL"];

TYPE

  GUID = ARRAY 4 OF INTEGER;

  PtrIUnknown = POINTER TO RECORD [untagged]

    vtbl: POINTER TO RECORD [untagged]

      QueryInterface: PROCEDURE (this: PtrIUnknown; IN iid: GUID; OUT obj: PtrIUnknown): INTEGER;

      AddRef: PROCEDURE (this: PtrIUnknown): INTEGER;

      Release: PROCEDURE (this: PtrIUnknown): INTEGER;

    END

  END;

  PtrDirectDraw = POINTER TO RECORD [untagged]

    vtbl: POINTER TO RECORD [untagged]

      QueryInterface: PROCEDURE (this: PtrDirectDraw; IN iid: GUID; OUT obj: PtrIUnknown): INTEGER;

      AddRef: PROCEDURE (this: PtrDirectDraw): INTEGER;

      Release: PROCEDURE (this: PtrDirectDraw): INTEGER;

      Compact: PROCEDURE (this: PtrDirectDraw): INTEGER;

      ...

      SetCooperativeLevel: PROCEDURE (this: PtrDirectDraw; w, x: INTEGER): INTEGER;

      ...

    END

  END;

PROCEDURE DirectDrawCreate* (IN guid: GUID; OUT PDD: PtrDirectDraw; outer: PtrIUnknown) : INTEGER;

END Ddraw.

MODULE Directone;

IMPORT Out, Ddraw, SYSTEM;

CONST

  DDSCL_EXCLUSIVE = 00000010H;

  DDSCL_FULLSCREEN = 00000001H;

PROCEDURE Initialize;

VAR

  Handle, Addr, Res: INTEGER;

  PDD: Ddraw.PtrDirectDraw;

  nul: Ddraw.GUID;

BEGIN

  PDD := NIL;

  nul[0] := 0; nul[1] := 0; nul[2] := 0; nul[3] := 0;

  Res := Ddraw.DirectDrawCreate(nul, PDD, NIL);

  Out.String("Res");  Out.Int(Res, 8);  Out.Ln();

  Res := SYSTEM.VAL(INTEGER, PDD);

  Out.String("Res");  Out.Int(Res, 8);  Out.Ln();

  Res := PDD.vtbl.SetCooperativeLevel(PDD, 0, DDSCL_EXCLUSIVE + DDSCL_FULLSCREEN);

  Out.String("Res");  Out.Int(Res, 8);  Out.Ln();

  Res := PDD.vtbl.Release(PDD)

END Initialize;

BEGIN

  Initialize

END Directone.

Some important points:

ꀢ COM GUIDs are 128 bit entities, not integers.

ꀢ DO NOT use ANYPTR or other BlackBox pointers for COM interface pointers. (BlackBox pointers are garbage collected, COM pointers are referenece counted.)

ꀢ Use pointers to [untagged] records or integers instead.

Be careful to declare all methods in the method table in the correct order with the correct parameter list.

**Windows programming interfaces**

See the [<u>Windows Programming Interfaces</u>](../../Win/Docu/Api.odc.md) Documentation.

**OLE Automation**

See the [<u>OLE Automation Controller</u>](../../Ctl/Docu/Sys-Map.odc.md) documentation.

<a id="3"></a>**Windows-specific information in BlackBox**

*Windows -specific cursors*

The module *HostPorts* exports constants for Windows-specific cursors which can be used like the standard cursors defined in module *Ports*:

    CONST

        resizeHCursor = 16;    *(* cursors used for window resizing *)*

        resizeVCursor = 17;

        resizeLCursor = 18;

        resizeRCursor = 19;

        resizeCursor = 20;

        busyCursor = 21;    *(* busy cursor *)*

        stopCursor = 22;    *(* drag and drop cursors *)*

        moveCursor = 23;

        copyCursor = 24;

        linkCursor = 25;

        pickCursor = 26;    *(* drag and pick cursor *)*



*Windows-specific mouse and keyboard modifiers*

Modifier sets are used in *Controllers.TrackMsg, Controllers.EditMsg, *and* Ports.Frame.Input. *In addition to the platform independant modifiers *Controllers.doubleClick*, *Controllers.extend*, and *Controllers.modify* they contain platform-specific modifiers defined in *HostPorts*:

    CONST

        left = 16;    *(* left mouse button pressed *)*

        middle = 17;    *(* middle mouse button pressed *)*

        right = 18;    *(* right mouse button pressed *)*

        shift = 24;    *(* Shift key pressed *)*

        ctrl = 25;    *(* Control key pressed *)*

        alt = 28;    *(* Alt key pressed *)*

*Window and device context handles*

Many of the functions in the Windows API refer either to a window or to a device context handle. In the Windows BlackBox implementation, both of them are stored in the *HostPorts.Port* record:

    TYPE

        Port = POINTER TO RECORD (Ports.PortDesc)

            dc: WinApi.Handle;

            wnd: WinApi.Handle

        END;



In the usual case where a frame (*Ports.Frame* or *Views.Frame*) is given, the handles can be obtained through one of the following selectors:

        frame.rider(HostPorts.Rider).port.dc

        or

        frame.rider(HostPorts.Rider).port.wnd

If the window handle is null, the port is a printer port.



*Examples*

The following simple DLL definition is a subset of the Windows standard library GDI32:

MODULE GDI ["GDI32"];

    CONST

        WhiteBrush* = 0; BlackBrush* = 4; NullBrush* = 5;

        WhitePen* = 6; BlackPen* = 7; NullPen* = 8;

        PSSolid* = 0; PSDash* = 1; PSDot* = 2;

    TYPE

        Bool* = INTEGER;

        Handle* = INTEGER;

        ColorRef* = INTEGER;

        Point* = RECORD [untagged] x*, y*: INTEGER END;

        Rect* = RECORD [untagged] left*, top*, right*, bottom*: INTEGER END;

    PROCEDURE CreatePen* (style, width: INTEGER; color: ColorRef): Handle;

    PROCEDURE CreateSolidBrush* (color: ColorRef): Handle;

    PROCEDURE GetStockObject* (object: INTEGER): Handle;

    PROCEDURE SelectObject* (dc, obj: Handle): Handle;

    PROCEDURE DeleteObject* (obj: Handle): Bool;

    PROCEDURE Rectangle* (dc: Handle; left, top, right, bottom: INTEGER): Bool;

    PROCEDURE SelectClipRgn* (dc, rgn: Handle): INTEGER;

    PROCEDURE IntersectClipRect* (dc: Handle; left, top, right, bottom: INTEGER): INTEGER;

    PROCEDURE SaveDC* (dc: Handle): INTEGER;

    PROCEDURE RestoreDC* (dc: Handle; saved: INTEGER): Bool;

END GDI.

The following example is a simplified version of the BlackBox standard *DrawRect* routines implemented in *Ports* and *HostPorts*. It uses the GDI interface presented above.

MODULE Ex;

    IMPORT Ports, HostPorts, GDI;

    PROCEDURE DrawRect (f: Ports.Frame; l, t, r, b, s: INTEGER; col: Ports.Color);

        VAR res, h, rl, rt, rr, rb: INTEGER; p: HostPorts.Port; dc, oldb, oldp: GDI.Handle;

    BEGIN

        *(* change local universal coordinates to window pixel coordinates *)*

        l := (f.gx + l) DIV f.unit; t := (f.gy + t) DIV f.unit;

        r := (f.gx + r) DIV f.unit; b := (f.gy + b) DIV f.unit;

        s := s DIV f.unit;

        *(* get device context *)*

        p := f.rider(HostPorts.Rider).port; dc := p.dc;

        *(* set clip region *)*

        IF p.wnd = 0 THEN res := GDI.SaveDC(dc)

        ELSE res := GDI.SelectClipRgn(dc, 0)

        END;

        f.rider.GetRect(rl, rt, rr, rb);

        res := GDI.IntersectClipRect(dc, rl, rt, rr, rb);

*        (* use black as default color *)*

        IF col = Ports.defaultColor THEN col := Ports.black END;

        IF s = 0 THEN s := 1 END;

        IF (s < 0) OR (r-l < 2*s) OR (b-t < 2*s) THEN    *(* filled rectangle *)*

            INC(r); INC(b);

            oldb := GDI.SelectObject(dc, GDI.CreateSolidBrush(col));

            oldp := GDI.SelectObject(dc, GDI.GetStockObject(GDI.NullPen));

            res := GDI.Rectangle(dc, l, t, r, b);

            res := GDI.DeleteObject(GDI.SelectObject(dc, oldb));

            res := GDI.SelectObject(dc, oldp)

        ELSE    *(* outline rectangle *)*

            h := s DIV 2; INC(l, h); INC(t, h); h := (s-1) DIV 2; DEC(r, h); DEC(b, h);

            oldb := GDI.SelectObject(dc, GDI.GetStockObject(GDI.NullBrush));

            oldp := GDI.SelectObject(dc, GDI.CreatePen(GDI.PSSolid, s, col));

            res := GDI.Rectangle(dc, l, t, r, b);

            res := GDI.SelectObject(dc, oldb);

            res := GDI.DeleteObject(GDI.SelectObject(dc, oldp))

        END;

        IF p.wnd = 0 THEN res := GDI.RestoreDC(dc, -1) END

    END DrawRect;

END Ex.

**Runtime type system**

All dynamically allocated Component Pascal records contain a hidden field, which is called the type tag. The type tag points to the type descriptor that contains all type information needed at runtime. Calls of type-bound procedures, type tests and the garbage collector all use the information which is stored in the type descriptor. The type descriptor is allocated only once for every record type in the system.

Dynamically allocated arrays use a size descriptor to check the bounds of indexes. The size descriptor also contains the type tag of the element type. For every single dynamic array a size descriptor is needed.

<a id="4"></a>**Differences between different Windows versions**

Tree controls look different on Windows NT compared to other versions of Windows. There is a defect in the routine for drawing the background of a Tree control in Windows NT. Hence, BlackBox does not change the color of the background for a disabled or read only tree control under Windows NT, whereas with other windows versions the background is set to gray when the control is disabled or read only.

**The BlackBox linker**

See the [<u>DevLinker</u>](Linker.odc.md) documentation.

<a id="6"></a>**Linking BlackBox applications**

If you want to distribute an application you have written in BlackBox, you may want to link all modules into a single file for distribution. In this case you need to link the framework to your application. To illustrate the necessary actions we will give an example. First duplicate the BlackBox application in the Explorer and rename it accordingly, in our case "Patterns". Adapt the module *Config* to your needs, e.g., delete the call which automatically opens the Log text. Then link the framework with your modules into the new application file. You may need to distribute some additional files with your application such as forms and the *Menu* text.

MODULE Config;

    IMPORT Dialog;

    PROCEDURE Setup*;

        VAR res: INTEGER;

    BEGIN

        Dialog.Call("ObxPatterns.Deposit; StdCmds.Open", "", res)

    END Setup;

END Config.

DevLinker.Link Patterns.exe :=

Kernel$+ Files HostFiles StdLoader Math Sub Strings Dates Meta Dialog Services

Fonts Ports Stores Converters Sequencers Models Printers Views

Controllers Properties Printing Mechanisms Containers

Documents Windows StdCFrames Controls StdDialog StdCmds StdInterpreter

HostRegistry HostFonts HostPorts OleData HostMechanisms HostWindows

HostPrinters HostClipboard HostCFrames HostDialog HostCmds

HostMenus HostPictures TextModels TextRulers TextSetters TextViews

TextControllers TextMappers StdLog TextCmds

FormModels FormViews FormControllers FormGen FormCmds

StdFolds StdLinks StdDebug HostTextConv HostMail

StdMenuTool StdClocks StdStamps StdLogos StdCoder StdScrollers

OleStorage OleServer OleClient StdETHConv Init ObxPatterns Config

1 applogo.Ico 2 doclogo.Ico 3 SFLogo.ico 4 CFLogo.ico 5 DtyLogo.ico

1 Move.cur 2 Copy.cur 3 Link.cur 4 Pick.cur 5 Stop.cur 6 Hand.cur 7 Table.cur ~

<a id="7"></a>**Startup of BlackBox**

The startup of BlackBox happens in several steps.

*Step 1: The operating system starts the application*

BlackBox consists of a small linked application and many unlinked code files (one per module). The linked application consists of at least the BlackBox *Kernel* module. When the operating system starts up BlackBox, it gives control to the module body of *Kernel*.

*Step 2: The kernel loads all prelinked modules*

The kernel initializes its data structures, in particular for memory management and exception handling. Then it executes the module bodies of the other modules which are linked in the application, in the correct order.

Usually, the module *StdLoader* is among the prelinked modules, along with several modules that it needs, in particular *Files* and *HostFiles*. Module *StdLoader* implements the linking loader which can dynamically link and load a module's code file.

*Step 3: Init loads the BlackBox library and core framework*

The standard implementation of module *StdLoader* performs a call to module *Init* in its body. If *Init* isn't linked, this causes the loader to attempt loading the code file *System/Code/Init*. If loading is possible, the modules imported by *Init* are loaded and initialized first, and then *Init* itself is initialized, i.e., its module body is executed.

The standard implementation of *Init* imports all core framework modules and their standard implementations, but not extension subsystems such as the *Text* or *Form* subsystem. These modules are loaded before the body of *Init* performs the following actions:

ꀢ It tries to call *Startup.Setup*.

Usually, module *Startup* does not exist. It could be used to install some services before the main window is opened and before the text subsystem or other subsystems are loaded.

ꀢ It tries to load module *DevDebug*.

*DevDebug* is expected to replace the kernel's rudimentary exception handling facilities by a more convenient tool. Note that the standard implementation of *DevDebug* uses the text subsystem, which thus is loaded also.

If loading of *DevDebug* is not successful, it is attempted to load module *StdDebug*. This is a reduced version of *DevDebug* which can be distributed along with an application.

ꀢ It registers the document file converter (importer/exporter).

This enables BlackBox to open and save standard BlackBox documents.

ꀢ It tries to call *StdMenuTool.UpdateAllMenus*.

This command reads and interprets the *System/Rsrc/Menus* text, and builds up the  menus accordingly. Note that the standard implementation of *StdMenuTool* uses the text subsystem.

ꀢ It tries to call *Config.Setup*.

This is the usual way to configure BlackBox. The standard implementation of *Config.Setup*  installs the standard file and OLE converters and opens the log text.

ꀢ It calls the main event loop (*HostMenus.Run*).

After the event loop is left (because the user wants to quit the application), the kernel is called to clean up, before the application is left entirely.

**Using NEW and garbage collection in your applications**

If you are calling *NEW* in your application and thereby implicitly use the garbage collector, you must link the *Kernel* into the application. The *NEW*-procedure is implemented in the kernel, the compiler just generates the code to call this procedure. So every module using *NEW* has a hidden import of the kernel.

Don't call *WinApi.ExitProcess* directly when "importing" the kernel, call *Kernel.Quit* with parameter 0 instead to assure that occupied system resources get properly released before the application is terminated.

Programs don't need to call the garbage collector explicitly. If the *NEW*-procedure cannot satisfy a request for heap space, it calls the garbage collector internally before allocating a new heap block from the Windows Memory Manager. The garbage collector marks pointers in stack frames and is able to run anytime.

The garbage collector reclaims all heap objects (dynamic record or array variables) that are not used anymore. "Unused" means that the object isn't directly reachable from some "root" pointer, or indirectly via a pointer chain starting from a root pointer. Any global variable which contains a pointer is such a root. If a root pointer isn't *NIL*, then it "anchors" a data structure. A heap object that isn't anchored anymore will eventually be collected by the garbage collector.

To allow the collector to follow pointer chains, there must be some information about the heap objects that are being traversed. In particular, it must be known where there are pointers in the object, if any. All objects of the same type share a common type descriptor, which is created when its defining module is loaded.

All dynamically allocated Component Pascal records contain a hidden field, which is called the type tag. The type tag points to the type descriptor that contains all type information needed at run-time. Method calls, type tests and the garbage collector all use the information which is stored in the type descriptor. The type descriptor is allocated only once for every record type in the system.

Dynamically allocated arrays use a size descriptor to check the bounds of indexes. The size descriptor also contains the type tag of the element type. For every single dynamic array a size descriptor is needed.

The additional storage required to "tag" objects makes their memory layout different from untagged records provided by the operating system or some procedural shared library. Type tags are critical; they must not be damaged, otherwise the garbage collector will crash the system at some point in time.

