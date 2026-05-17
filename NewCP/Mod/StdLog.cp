MODULE StdLog;
(*
   Windowed log view — first slice.

   Maintains a shared TextModels.Doc that a TextViews.Pane window
   paints on each natural repaint.  Appending text does NOT force an
   immediate repaint; new content appears at the next iGui EvPaint
   event dispatched by the host event loop.

   Public interface mirrors Log.cp but routes through a real text
   model so a full editor pane can display (and eventually scroll)
   the output.

   Pass 1 limitations:
     - No auto-scroll after Ln (deferred to full TextViews layout slice).
     - No "clear" that notifies the view (Clear just resets the writer
       position; the view repaint will show the truncated content).
     - Importing this module does NOT auto-open the window; callers
       must invoke StdLog.Open explicitly.
*)

    IMPORT Documents, Windows, TextModels, TextViews;

    CONST
        winWidth  = 700;
        winHeight = 400;

    VAR
        logDoc: TextModels.Doc;
        logWin: Windows.Window;
        logWr:  TextModels.Writer;


    (* ---- Private: ensure model+writer exist ----------------------------- *)

    PROCEDURE EnsureModel;
    BEGIN
        IF logDoc = NIL THEN
            NEW(logDoc);
            logWr := logDoc.NewWriter(NIL)
        END;
        IF logWr = NIL THEN
            logWr := logDoc.NewWriter(NIL)
        END
    END EnsureModel;


    (* ---- Private: repaint log window if open --------------------------- *)

    PROCEDURE RepaintLog;
    BEGIN
        IF (logWin # NIL) & logWin.IsValid() THEN
            logWin.Repaint()
        END
    END RepaintLog;


    (* ---- Public API ----------------------------------------------------- *)

    (** Open (or re-use) the log window.
        If the window already exists and is still valid this is a no-op.
        The window title is "Log". *)
    PROCEDURE Open*;
        VAR view: TextViews.View; doc: Documents.Document;
    BEGIN
        (* Re-use the existing window if it is still valid. *)
        IF (logWin # NIL) & logWin.IsValid() THEN RETURN END;
        EnsureModel;
        IF TextViews.dir = NIL THEN RETURN END;
        view := TextViews.dir.New(logDoc);
        IF view = NIL THEN RETURN END;
        IF Documents.dir = NIL THEN RETURN END;
        doc := Documents.dir.New(view, winWidth, winHeight);
        IF doc = NIL THEN RETURN END;
        logWin := Windows.Open(doc, "Log", winWidth, winHeight)
    END Open;


    (** Append a wide-character string to the log. *)
    PROCEDURE String* (IN s: ARRAY OF CHAR);
        VAR i: INTEGER;
    BEGIN
        EnsureModel;
        i := 0;
        WHILE (i < LEN(s)) & (s[i] # 0X) DO
            logWr.WriteChar(s[i]); INC(i)
        END;
        RepaintLog
    END String;


    (** Append a short (SHORTCHAR / ASCII) string to the log. *)
    PROCEDURE SString* (IN s: ARRAY OF SHORTCHAR);
        VAR i: INTEGER;
    BEGIN
        EnsureModel;
        i := 0;
        WHILE (i < LEN(s)) & (s[i] # 0X) DO
            logWr.WriteChar(LONG(s[i])); INC(i)
        END;
        RepaintLog
    END SString;


    (** Append a decimal integer to the log. *)
    PROCEDURE Int* (n: INTEGER);
        VAR buf: ARRAY 24 OF CHAR;
            i, j: INTEGER;
            neg:  BOOLEAN;
    BEGIN
        EnsureModel;
        i := 23; buf[i] := 0X;
        neg := n < 0;
        IF n = 0 THEN
            DEC(i); buf[i] := '0'
        ELSE
            IF neg THEN n := -n END;
            WHILE n > 0 DO
                DEC(i);
                buf[i] := CHR(ORD('0') + n MOD 10);
                n := n DIV 10
            END;
            IF neg THEN DEC(i); buf[i] := '-' END
        END;
        j := i;
        WHILE buf[j] # 0X DO logWr.WriteChar(buf[j]); INC(j) END;
        RepaintLog
    END Int;


    (** Append a line separator (TextModels.line = 0DX) to the log. *)
    PROCEDURE Ln*;
    BEGIN
        EnsureModel;
        logWr.WriteChar(TextModels.line);
        RepaintLog
    END Ln;


    (** Reset the log — allocates a fresh empty model.
        The existing log window (if open) will be invalidated; call
        Open* again after Clear* to reopen with an empty document. *)
    PROCEDURE Clear*;
    BEGIN
        NEW(logDoc);
        logWr  := logDoc.NewWriter(NIL);
        logWin := NIL   (* force re-open on next Open* call *)
    END Clear;


BEGIN
    logDoc := NIL;
    logWin := NIL;
    logWr  := NIL
END StdLog.
