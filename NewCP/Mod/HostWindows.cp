MODULE HostWindows;
(*
   Concrete implementation of the Windows abstract surface.

   Follows the HostXxxSys layer pattern:
     - Windows.cp      — abstract interface (no iGui dependency)
     - HostWindows.cp  — concrete implementation (imports HostWindowsSys)
     - HostWindowsSys.cp — ONLY module that imports iGui for title management

   Pass 1: Scroll is EMPTY; iGui-side resize is deferred.
*)

    IMPORT Documents, Windows, Views, Ports, HostPorts, HostWindowsSys;


    (* -- Types ------------------------------------------------------------ *)

    TYPE
        (* Concrete frame for painting — same pattern as PaneDemo.PaintFrame.
           Views.FrameDesc inherits l, t, r, b with '-' export but Restore
           only reads the l/t/r/b passed as parameters, so zero-init is fine. *)
        PaintFrameDesc = RECORD (Views.FrameDesc) END;
        PaintFrame     = POINTER TO PaintFrameDesc;

        HostWindowDesc = RECORD (Windows.WindowDesc)
            doc:   Documents.Document;
            port:  HostPorts.HostPort;
            frame: PaintFrame;
            title: ARRAY 256 OF CHAR;
            w, h:  INTEGER
        END;
        HostWindow = POINTER TO HostWindowDesc;

        HostDirectoryDesc = RECORD (Windows.DirectoryDesc) END;
        HostDirectory     = POINTER TO HostDirectoryDesc;


    VAR
        stdHostDir: HostDirectory;


    (* -- Private helper --------------------------------------------------- *)

    (* Scan Windows.first list for the HostWindow whose port.childId matches. *)
    PROCEDURE FindByChildId (childId: INTEGER): HostWindow;
        VAR w: Windows.Window;
    BEGIN
        w := Windows.first;
        WHILE w # NIL DO
            IF (w IS HostWindow) & (w(HostWindow).port # NIL)
             & (w(HostWindow).port.childId = childId) THEN
                RETURN w(HostWindow)
            END;
            w := w.next
        END;
        RETURN NIL
    END FindByChildId;


    (* -- HostWindow methods (implement all 8 ABSTRACT from Windows.Window) -- *)

    PROCEDURE (w: HostWindow) IsValid* (): BOOLEAN;
    BEGIN
        RETURN w.port # NIL
    END IsValid;

    PROCEDURE (w: HostWindow) ThisDoc* (): Documents.Document;
    BEGIN
        RETURN w.doc
    END ThisDoc;

    PROCEDURE (w: HostWindow) SetTitle* (IN title: ARRAY OF CHAR);
    BEGIN
        w.title := title;
        IF w.port # NIL THEN
            HostWindowsSys.SetTitle(w.port.childId, title)
        END
    END SetTitle;

    PROCEDURE (w: HostWindow) GetTitle* (OUT title: ARRAY OF CHAR);
    BEGIN
        title := w.title
    END GetTitle;

    PROCEDURE (w: HostWindow) SetSize* (width, height: INTEGER);
    BEGIN
        (* iGui resize deferred — cache only for Pass 1. *)
        w.w := width;
        w.h := height
    END SetSize;

    PROCEDURE (w: HostWindow) GetSize* (OUT width, height: INTEGER);
    BEGIN
        width  := w.w;
        height := w.h
    END GetSize;

    PROCEDURE (w: HostWindow) Scroll* (dx, dy: INTEGER);
    BEGIN
        (* EMPTY — Pass 1 *)
    END Scroll;

    PROCEDURE (w: HostWindow) Close* ();
    BEGIN
        HostPorts.Close(w.port);
        w.port := NIL
    END Close;


    (* -- HostWindow.Repaint ----------------------------------------------- *)

    PROCEDURE (w: HostWindow) Repaint*, NEW;
        VAR ok: INTSHORT;
    BEGIN
        IF (w.port = NIL) OR (w.doc = NIL) THEN RETURN END;
        HostPorts.BeginPaint(w.port);
        w.doc.Restore(w.frame, 0, 0, w.w, w.h);
        ok := HostPorts.SubmitPaint()
    END Repaint;


    (* -- HostDirectory.New ----------------------------------------------- *)

    PROCEDURE (dir: HostDirectory) New* (doc: Documents.Document;
                                         IN title: ARRAY OF CHAR;
                                         w, h: INTEGER): Windows.Window;
        VAR win: HostWindow; pf: PaintFrame; childId: INTEGER;
            shortTitle: ARRAY 256 OF SHORTCHAR; i: INTEGER;
    BEGIN
        (* CHAR → SHORTCHAR for HostPorts.NewPort which expects SHORTCHAR *)
        i := 0;
        WHILE (title[i] # 0X) & (i < 255) DO
            shortTitle[i] := SHORT(title[i]);
            INC(i)
        END;
        shortTitle[i] := 0X;

        NEW(win);
        win.port := HostPorts.NewPort(shortTitle, childId);
        IF win.port = NIL THEN RETURN NIL END;
        win.port.Init(1, FALSE);   (* 1 unit = 1 DIP, no scaling *)
        win.doc   := doc;
        win.title := title;
        win.w     := w;
        win.h     := h;

        NEW(pf);
        pf.ConnectTo(win.port);
        pf.SetOffset(0, 0);
        win.frame := pf;

        RETURN win
    END New;


    (* -- Public event-dispatch helpers (called by BbInit/event loop) ------- *)

    (* Find window by childId and trigger a repaint. *)
    PROCEDURE PaintChild* (childId: INTEGER);
        VAR w: HostWindow;
    BEGIN
        w := FindByChildId(childId);
        IF w # NIL THEN w.Repaint() END
    END PaintChild;

    (* Update size cache and repaint. *)
    PROCEDURE ResizeChild* (childId, width, height: INTEGER);
        VAR w: HostWindow;
    BEGIN
        w := FindByChildId(childId);
        IF w # NIL THEN
            w.w := width; w.h := height;
            w.port.SetSize(width, height);
            w.Repaint()
        END
    END ResizeChild;

    (* Mark window closed — Close() sets port := NIL so FindByChildId
       skips it on subsequent events. *)
    PROCEDURE CloseChild* (childId: INTEGER);
        VAR w: HostWindow;
    BEGIN
        w := FindByChildId(childId);
        IF w # NIL THEN w.Close() END
    END CloseChild;


BEGIN
    NEW(stdHostDir);
    Windows.SetDir(stdHostDir)
END HostWindows.
