**DevLinker**

*DevLinker* is the BlackBox linker. It is used to pack several BlackBox code files together into one executable file (*.dll* or *.exe*). It can be used to make independent versions of applications based on the BlackBox Component Framework. It can also be used to produce executables written in Component Pascal which don't relate to the BlackBox framework or use only a few BlackBox modules like the *Kernel*.

The linker can be started with one of the commands described below. Each of the commands needs a parameter text with the syntax:

<destFile> := {<module> {option}} {idNumber <resourceFile>}.

*destFile* is the name of the executable file to be created.

*module* is a Component Pascal module, the code file is loaded from the corresponding directory.

*option* is one of the following characters:

    $ main module: the body of this module is called when the program starts.

    + identifies the kernel. A kernel must be present if the standard function *NEW* is used in some module. The kernel must export the procedures *NewRec* and *NewArr*.

    # interface module: the exported procedures of this module are added to the export list.

    See the description of the individual commands for a list of the legal options.

*resourceFile* is the name of a resource file. Currently icons (*.ico*), cursors (*.cur*), bitmaps (.*bmp*), windows resource files (.*res*), and type libraries (.*tlb*) are supported. The resource files are loaded from the *Rsrc* or *Win/Rsrc* directory.

*idNumber* is an integer used to reference the corresponding resource from the program.

The module list must be sorted such that an imported module textually precedes the module importing it. This rule also applies to the implicit kernel import when using *NEW*.

**DevLinker.Link**

Links a module set containing a dynamic module loader to an exe file.

At startup, the body of the main module is called.

Initialization and termination of the other modules is not done by the loader, it must be done by the runtime system (typically the loader).

BlackBox itself is linked with this command.

Legal options: $ +

**DevLinker.LinkExe**

Links an unextensible module set to an exe file (i.e., no loader is included).

At startup, the bodies of all modules are called in the correct order.

When the last body terminates, the terminators (CLOSE sections) of all modules are called in reverse order.

No runtime system is needed for initialization and termination.

Legal options: +

**DevLinker.LinkDll**

Links an unextensible module set to a dll file.

When the dll is attached to a process, the bodies of all modules are called in the correct order.

When the dll is released from the process, the terminators (CLOSE sections) of all modules are called in reverse order.

No runtime system is needed for initialization and termination.

Legal options: + #

**DevLinker.LinkDynDll**

(rarely used, present for completeness)

Links a module set containing a dynamic module loader to a dll file.

When the dll is attached to a process, the body of the main module is called.

When the dll is released from the process, the terminator (CLOSE section) of the main module is called.

Initialization and termination of the other modules must be done by the runtime system.

Legal options: $ + #

The reason for the different commands for static and dynamic systems is that there is no statically defined initialization sequence in a system that includes a dynamic loader. In BlackBox the *Kernel* (which is the lowest module in the module hierarchy) is specified as the main module. The body of the kernel then calls the bodies of all linked modules dynamically in the correct sequence. If there are no calls to the dynamic loader (via *Dialog.Call*) in the module bodies, the modules are initialized in the order in which they appear in the parameter text.

**Examples**

Standard BlackBox:

DevLinker.Link

BlackBox.exe := Kernel$+ Files HostFiles StdLoader

1 Applogo.ico 2 Doclogo.ico 3 SFLogo.ico 4 CFLogo.ico 5 DtyLogo.ico

1 Move.cur 2 Copy.cur 3 Link.cur 4 Pick.cur 5 Stop.cur 6 Hand.cur 7 Table.cur

Fully linked redistributable part of BlackBox:

DevLinker.Link MyBlackBox.exe :=

Kernel$+ Files HostFiles StdLoader Math Strings Dates Meta Dialog Services

Fonts Ports Stores Log Converters Sequencers Models Printers Views

Controllers Properties Printing Mechanisms Containers

Documents Windows StdCFrames Controls StdDialog StdApi StdCmds StdInterpreter

HostRegistry HostFonts HostPorts OleData HostMechanisms HostWindows

HostPrinters HostClipboard HostCFrames HostDialog HostCmds

HostMenus HostPictures TextModels TextRulers TextSetters TextViews

TextControllers TextMappers StdLog TextCmds

FormModels FormViews FormControllers FormGen FormCmds

StdFolds StdLinks StdDebug HostTextConv HostMail

StdMenuTool StdClocks StdStamps StdLogos StdCoder StdScrollers

OleStorage OleServer OleClient StdETHConv In Out XYplane Init

1 applogo.Ico 2 doclogo.Ico 3 SFLogo.ico 4 CFLogo.ico 5 DtyLogo.ico

1 Move.cur 2 Copy.cur 3 Link.cur 4 Pick.cur 5 Stop.cur 6 Hand.cur 7 Table.cur

Simple independent application:

DevLinker.LinkExe

Simple.exe := Simple 1 applogo.Ico ~

Simple DLL:

DevLinker.LinkDll

Mydll.dll := Mydll# ~

MODULE Simple;

*(* simple windows application *)*

    IMPORT S := SYSTEM, WinApi;

    CONST

        message = "Hello World";

        iconId = 1;

    VAR

        instance: WinApi.HINSTANCE;

        mainWnd: WinApi.HWND;

    PROCEDURE WndHandler (wnd, msg, wParam, lParam: INTEGER): INTEGER;

        VAR res: INTEGER; ps: WinApi.PAINTSTRUCT; dc: WinApi.HDC;

    BEGIN

        IF msg = WinApi.WM_DESTROY THEN

            WinApi.PostQuitMessage(0)

        ELSIF msg = WinApi.WM_PAINT THEN

            dc := WinApi.BeginPaint(wnd, ps);

            res := WinApi.TextOut(dc, 50, 50, message, LEN(message));

            res := WinApi.EndPaint(wnd, ps)

        ELSIF msg = WinApi.WM_CHAR THEN

            res := WinApi.Beep(800, 200)

        ELSE

            **RETURN** WinApi.DefWindowProc(wnd, msg, wParam, lParam)

        END;

        **RETURN** 0

    END WndHandler;

    PROCEDURE OpenWindow;

        VAR class: WinApi.WNDCLASS; res: INTEGER;

    BEGIN

        class.hCursor := WinApi.LoadCursor(0, S.VAL(WinApi.PtrSTR,

                                                                            WinApi.IDC_ARROW));

        class.hIcon := WinApi.LoadIcon(instance, S.VAL(WinApi.PtrSTR, iconId));

        class.lpszMenuName := NIL;

        class.lpszClassName := "Simple";

        class.hbrBackground := WinApi.GetStockObject(WinApi.WHITE_BRUSH);

        class.style := WinApi.CS_VREDRAW + WinApi.CS_HREDRAW

                        (* + WinApi.CS_OWNDC + WinApi.CS_PARENTDC *);

        class.hInstance := instance;

        class.lpfnWndProc := WndHandler;

        class.cbClsExtra := 0;

        class.cbWndExtra := 0;

        res := WinApi.RegisterClass(class);

        mainWnd := WinApi.CreateWindowEx({}, "Simple", "Simple Application",

                                                        WinApi.WS_OVERLAPPEDWINDOW,

                                                        WinApi.CW_USEDEFAULT, WinApi.CW_USEDEFAULT,

                                                        WinApi.CW_USEDEFAULT, WinApi.CW_USEDEFAULT,

                                                        0, 0, instance, 0);

        res := WinApi.ShowWindow(mainWnd, WinApi.SW_SHOWDEFAULT);

        res := WinApi.UpdateWindow(mainWnd);

    END OpenWindow;

    PROCEDURE MainLoop;

        VAR msg: WinApi.MSG; res: INTEGER;

    BEGIN

        WHILE WinApi.GetMessage(msg, 0, 0, 0) # 0 DO

            res := WinApi.TranslateMessage(msg);

            res := WinApi.DispatchMessage(msg);

        END;

        WinApi.ExitProcess(msg.wParam)

    END MainLoop;

BEGIN

    instance := WinApi.GetModuleHandle(NIL);

    OpenWindow;

    MainLoop

END Simple.

MODULE Mydll;

*(* sample module to be linked into a dll *)*

    PROCEDURE **Gcd*** (a, b: INTEGER): INTEGER;

    BEGIN

        WHILE a # b DO

            IF a > b THEN a := a - b ELSE b := b - a END

        END;

        **RETURN** a

    END Gcd;



    PROCEDURE **Lcm*** (a, b: INTEGER): INTEGER;

    BEGIN

        **RETURN** a * b DIV Gcd(a, b)

    END Lcm;



END Mydll.

