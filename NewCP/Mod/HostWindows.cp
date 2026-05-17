MODULE HostWindows;
(*
   Concrete implementation of the Windows abstract surface.

   Follows the HostXxxSys layer pattern:
     - Windows.cp      — abstract interface (no iGui dependency)
     - HostWindows.cp  — concrete implementation (imports HostWindowsSys)
     - HostWindowsSys.cp — ONLY module that imports iGui for title management

   Pass 1: Scroll is EMPTY; iGui-side resize is deferred.
*)

    IMPORT Documents, Windows, Views, Controllers, TextModels, TextViews, TextControllers, Ports, HostPorts, HostWindowsSys;


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

        (** Drag-selection state.  Set by HandleMouseDown on every
            left-click; HandleMouseDrag updates the selection by
            extending from this anchor to the current click position.
            HandleMouseUp clears isDragging. *)
        dragAnchor: INTEGER;
        isDragging: BOOLEAN;


    (* -- Private helpers -------------------------------------------------- *)

    (** Copy a wide string (ARRAY OF CHAR) into a fixed buffer. *)
    PROCEDURE CopyStr (IN src: ARRAY OF CHAR; VAR dst: ARRAY OF CHAR);
        VAR i: INTEGER;
    BEGIN
        i := 0;
        WHILE (i < LEN(dst) - 1) & (i < LEN(src)) & (src[i] # 0X) DO
            dst[i] := src[i]; INC(i)
        END;
        dst[i] := 0X
    END CopyStr;

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
        CopyStr(title, w.title);
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

    PROCEDURE (w: HostWindow) Repaint*;
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
        CopyStr(title, win.title);
        win.w     := w;
        win.h     := h;

        NEW(pf);
        pf.ConnectTo(win.port);
        pf.SetOffset(0, 0);
        win.frame := pf;

        RETURN win
    END New;


    (* -- Public event-dispatch helpers (called by BbInit/event loop) ------- *)

    (* Caret-follow scroll: adjust pane.org so the caret is in the visible area.
       `windowH` = current window height in DIPs.
       barH / lineH must match the constants in TextViews.Pane.Restore. *)
    PROCEDURE EnsureCaretVisible (pane: TextViews.Pane; windowH: INTEGER);
        CONST barH = 50; lineH = 120;
        VAR carPos, org, pos, n, target, newOrg: INTEGER;
            rd: TextModels.Reader; visLines: INTEGER;
    BEGIN
        IF pane.text = NIL THEN RETURN END;
        IF pane.controller = NIL THEN RETURN END;
        carPos := pane.controller.CaretPos();
        IF carPos = TextControllers.none THEN RETURN END;
        org := pane.org;
        visLines := (windowH - barH) DIV lineH;
        IF visLines < 1 THEN visLines := 1 END;

        (* Case 1: caret is before the visible area — scroll up. *)
        IF carPos < org THEN
            rd := pane.text.NewReader(NIL);
            IF rd = NIL THEN RETURN END;
            rd.SetPos(0); rd.ReadChar();
            pos := 0; newOrg := 0;
            WHILE ~rd.eot & (pos < carPos) DO
                IF (rd.char = TextModels.line) OR (rd.char = TextModels.para) THEN
                    newOrg := rd.Pos()   (* start of the next line *)
                END;
                INC(pos); rd.ReadChar()
            END;
            pane.SetOrigin(newOrg, 0);
            RETURN
        END;

        (* Count line seps in [org, carPos) to find the caret's line number. *)
        rd := pane.text.NewReader(NIL);
        IF rd = NIL THEN RETURN END;
        rd.SetPos(org); rd.ReadChar();
        pos := org; n := 0;
        WHILE ~rd.eot & (pos < carPos) DO
            IF (rd.char = TextModels.line) OR (rd.char = TextModels.para) THEN INC(n) END;
            INC(pos); rd.ReadChar()
        END;

        (* Case 2: caret is below the visible area — scroll down. *)
        IF n >= visLines THEN
            (* New org = start of line (n - visLines + 1) from current org. *)
            target := n - visLines + 1;
            rd.SetPos(org); rd.ReadChar();
            pos := org; newOrg := org;
            WHILE ~rd.eot & (target > 0) DO
                IF (rd.char = TextModels.line) OR (rd.char = TextModels.para) THEN
                    DEC(target);
                    IF target = 0 THEN newOrg := rd.Pos() END
                END;
                INC(pos); rd.ReadChar()
            END;
            pane.SetOrigin(newOrg, 0)
        END
    END EnsureCaretVisible;


    (* Find window by childId, adjust scroll for caret visibility, then repaint. *)
    PROCEDURE PaintChild* (childId: INTEGER);
        VAR w: HostWindow; v: Views.View;
    BEGIN
        w := FindByChildId(childId);
        IF w = NIL THEN RETURN END;
        (* Caret-follow: scroll the pane so the caret is on screen. *)
        IF w.doc # NIL THEN
            v := w.doc.ThisView();
            IF (v # NIL) & (v IS TextViews.Pane) THEN
                EnsureCaretVisible(v(TextViews.Pane), w.h)
            END
        END;
        w.Repaint()
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

    (* Map (x, y) DIPs to a text model position within a Pane.
       Uses the TextViews.Restore rendering geometry:
         barH = 50  — indicator bar at the top
         lineH = 120 — height per rendered line
       Within a line, rough character width ≈ 8 DIPs.
       `pane.org` is honoured: the first visible line is at `pane.org`.
       Returns -1 if the pane has no text. *)
    PROCEDURE HitTestPos (pane: TextViews.Pane; x, y: INTEGER): INTEGER;
        CONST barH = 50; lineH = 120;
        VAR rd: TextModels.Reader;
            lineIdx, pos, lineStart, col, lineLen: INTEGER;
    BEGIN
        IF pane.text = NIL THEN RETURN -1 END;
        IF y < barH + lineH THEN lineIdx := 0
        ELSE lineIdx := (y - barH - lineH) DIV lineH
        END;
        rd := pane.text.NewReader(NIL);
        IF rd = NIL THEN RETURN -1 END;
        (* Start from the scroll origin `pane.org`. *)
        rd.SetPos(pane.org); rd.ReadChar();
        pos := pane.org;
        WHILE (lineIdx > 0) & ~rd.eot DO
            WHILE ~rd.eot & (rd.char # TextModels.line) & (rd.char # TextModels.para) DO
                INC(pos); rd.ReadChar()
            END;
            IF rd.eot THEN lineIdx := 0  (* clamp to last line *)
            ELSE INC(pos); DEC(lineIdx); rd.ReadChar()
            END
        END;
        lineStart := pos;
        col := x DIV 8;
        IF col < 0 THEN col := 0 END;
        lineLen := 0;
        WHILE ~rd.eot & (rd.char # TextModels.line) & (rd.char # TextModels.para) DO
            INC(lineLen); rd.ReadChar()
        END;
        IF col > lineLen THEN col := lineLen END;
        RETURN lineStart + col
    END HitTestPos;

    (* Map a left-mouse-down at (x, y) DIPs to a caret position and set it.
       Records the click position as the drag anchor for subsequent drag events. *)
    PROCEDURE HandleMouseDown* (childId, x, y: INTEGER);
        VAR w: HostWindow; v: Views.View;
            pane: TextViews.Pane; ctrl: TextControllers.Controller;
            pos: INTEGER;
    BEGIN
        w := FindByChildId(childId);
        IF w = NIL THEN RETURN END;
        v := w.doc.ThisView();
        IF (v = NIL) OR ~(v IS TextViews.Pane) THEN RETURN END;
        pane := v(TextViews.Pane);
        Controllers.SetFocusView(v);
        pos := HitTestPos(pane, x, y);
        IF pos < 0 THEN RETURN END;
        IF (pane.controller # NIL) & (pane.controller IS TextControllers.Controller) THEN
            ctrl := pane.controller(TextControllers.Controller);
            ctrl.SetCaret(pos);
            ctrl.SetSelection(pos, pos)
        END;
        dragAnchor := pos;
        isDragging := TRUE
    END HandleMouseDown;

    (* End a drag: clear the isDragging flag so subsequent mouse-move
       events no longer extend the selection. *)
    PROCEDURE HandleMouseUp* (childId: INTEGER);
    BEGIN
        isDragging := FALSE
    END HandleMouseUp;

    (* Update the selection while the left button is dragged.
       Extends from dragAnchor to the current (x, y) position. *)
    PROCEDURE HandleMouseDrag* (childId, x, y: INTEGER);
        VAR w: HostWindow; v: Views.View;
            pane: TextViews.Pane; ctrl: TextControllers.Controller;
            pos: INTEGER;
    BEGIN
        IF ~isDragging THEN RETURN END;
        w := FindByChildId(childId);
        IF w = NIL THEN RETURN END;
        v := w.doc.ThisView();
        IF (v = NIL) OR ~(v IS TextViews.Pane) THEN RETURN END;
        pane := v(TextViews.Pane);
        pos := HitTestPos(pane, x, y);
        IF pos < 0 THEN RETURN END;
        IF (pane.controller # NIL) & (pane.controller IS TextControllers.Controller) THEN
            ctrl := pane.controller(TextControllers.Controller);
            ctrl.SetCaret(pos);
            IF pos <= dragAnchor THEN
                ctrl.SetSelection(pos, dragAnchor)
            ELSE
                ctrl.SetSelection(dragAnchor, pos)
            END
        END
    END HandleMouseDrag;

    (* Route focus to the inner view of the MDI child that gained focus.
       This allows TextControllers.Focus() to find the active controller. *)
    PROCEDURE FocusChild* (childId: INTEGER);
        VAR w: HostWindow; v: Views.View;
    BEGIN
        w := FindByChildId(childId);
        IF w # NIL THEN
            v := w.doc.ThisView();
            Controllers.SetFocusView(v)
        END
    END FocusChild;


    (* Scroll the text pane in childId by `lines` lines (positive = down,
       negative = up).  Adjusts pane.org to skip/back over line separators.
       Silently returns if the view is not a text pane. *)
    PROCEDURE ScrollLines* (childId, lines: INTEGER);
        VAR w: HostWindow; v: Views.View;
            pane: TextViews.Pane;
            rd: TextModels.Reader;
            pos, n: INTEGER;
    BEGIN
        w := FindByChildId(childId);
        IF w = NIL THEN RETURN END;
        v := w.doc.ThisView();
        IF (v = NIL) OR ~(v IS TextViews.Pane) THEN RETURN END;
        pane := v(TextViews.Pane);
        IF pane.text = NIL THEN RETURN END;
        pos := pane.org;
        IF lines > 0 THEN
            (* Scroll down: advance org past `lines` line separators. *)
            rd := pane.text.NewReader(NIL);
            IF rd = NIL THEN RETURN END;
            rd.SetPos(pos); rd.ReadChar();
            n := lines;
            WHILE (n > 0) & ~rd.eot DO
                WHILE ~rd.eot & (rd.char # TextModels.line) & (rd.char # TextModels.para) DO
                    INC(pos); rd.ReadChar()
                END;
                IF ~rd.eot THEN INC(pos); DEC(n); rd.ReadChar() END
            END
        ELSIF lines < 0 THEN
            (* Scroll up by N lines: scan from text start counting
               line separators, find the position of the (lineNo - N)-th
               line where lineNo is the current org's line number.
               This is O(text) but simple and correct. *)
            rd := pane.text.NewReader(NIL);
            IF rd = NIL THEN RETURN END;
            (* Count what line number org is at. *)
            rd.SetPos(0); rd.ReadChar();
            n := 0;  (* current line counter from start *)
            pos := 0;
            WHILE ~rd.eot & (pos < pane.org) DO
                IF (rd.char = TextModels.line) OR (rd.char = TextModels.para) THEN
                    INC(n)
                END;
                INC(pos); rd.ReadChar()
            END;
            (* Target line = max(0, n + lines) where lines is negative. *)
            n := n + lines;
            IF n < 0 THEN n := 0 END;
            (* Scan from start to find start of target line. *)
            rd.SetPos(0); rd.ReadChar();
            pos := 0;
            WHILE ~rd.eot & (n > 0) DO
                IF (rd.char = TextModels.line) OR (rd.char = TextModels.para) THEN
                    DEC(n)
                END;
                INC(pos); rd.ReadChar()
            END
            (* pos is now the start of the target line *)
        END;
        pane.SetOrigin(pos, 0);
        w.Repaint()
    END ScrollLines;


BEGIN
    NEW(stdHostDir);
    Windows.SetDir(stdHostDir);
    dragAnchor := 0;
    isDragging := FALSE
END HostWindows.
