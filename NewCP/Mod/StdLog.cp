MODULE StdLog;
(*
   First slice of the BlackBox `StdLog` port.

   BB's StdLog (~650 lines) opens a dedicated TextViews.View as
   the system log — a stop-the-world capture for `Out.String`
   redirection and runtime trap messages.  Config.Setup's last
   line is `Dialog.Call("StdLog.Open", "", res)` which makes
   the system log window visible at startup.

   This slice surfaces every public procedure (Char / Int /
   Real / String / Bool / Set / IntForm / RealForm / Tab / Ln /
   Para / View / ViewForm / ParamMsg / Msg / New / Open /
   Clear) and routes the textual ones through Console so the
   log output is visible even before the TextViews-backed
   window is wired up.  The TextModels-backed buffer / view-
   side machinery is deferred.

   Deferred: every TextModels-backed body — `text`, `buf`,
   `defruler`, `dir`, `out`, `subOut` and the `Flush` /
   embedded-view paths.  Once StdDialog.Open actually puts a
   window on screen, this slice grows the TextView wiring.
*)

    IMPORT Console, Strings, Views;


    PROCEDURE Char* (ch: CHAR);
    BEGIN
        Console.WriteChar(ch)
    END Char;

    PROCEDURE Int* (i: LONGINT);
        VAR buf: ARRAY 32 OF CHAR;
    BEGIN
        Strings.IntToString(i, buf);
        Console.WriteString(buf)
    END Int;

    PROCEDURE Real* (x: REAL);
    BEGIN
        Console.WriteReal(x)
    END Real;

    PROCEDURE String* (IN str: ARRAY OF CHAR);
    BEGIN
        Console.WriteString(str)
    END String;

    PROCEDURE Bool* (x: BOOLEAN);
    BEGIN
        IF x THEN Console.WriteString("TRUE")
        ELSE      Console.WriteString("FALSE")
        END
    END Bool;

    PROCEDURE Set* (x: SET);
        VAR buf: ARRAY 32 OF CHAR;
    BEGIN
        Strings.SetToString(x, buf);
        Console.WriteString(buf)
    END Set;

    (** BB's IntForm width-pads and bases the int.  We don't
        have the formatter wired up yet; for the welcome-page
        path we just emit decimal. *)
    PROCEDURE IntForm* (x: LONGINT; base, minWidth: INTEGER; fillCh: CHAR; showBase: BOOLEAN);
    BEGIN
        Int(x)
    END IntForm;

    (** BB's RealForm width-pads and prec-formats the real. *)
    PROCEDURE RealForm* (x: REAL; precision, minW, expW: INTEGER; fillCh: CHAR);
    BEGIN
        Real(x)
    END RealForm;

    PROCEDURE Tab*;
    BEGIN
        Console.WriteChar(09X)
    END Tab;

    PROCEDURE Ln*;
    BEGIN
        Console.WriteLn
    END Ln;

    PROCEDURE Para*;
    BEGIN
        Console.WriteLn
    END Para;

    (** Embed a View into the log.  Stub — until the
        TextView buffer is real, embedded views aren't
        rendered. *)
    PROCEDURE View* (v: Views.View);
    BEGIN
    END View;

    PROCEDURE ViewForm* (v: Views.View; w, h: INTEGER);
    BEGIN
    END ViewForm;

    (** ParamMsg: substitute &0/&1/&2 in `msg` with `p0`/`p1`/
        `p2`.  Stub: emit verbatim. *)
    PROCEDURE ParamMsg* (IN msg, p0, p1, p2: ARRAY OF CHAR);
    BEGIN
        Console.WriteString(msg);
        Console.WriteLn
    END ParamMsg;

    PROCEDURE Msg* (IN msg: ARRAY OF CHAR);
    BEGIN
        Console.WriteString(msg);
        Console.WriteLn
    END Msg;

    (** Clear the log.  Stub — until we have a real buffer to
        clear. *)
    PROCEDURE Clear*;
    BEGIN
        Console.WriteString("[StdLog.Clear]");
        Console.WriteLn
    END Clear;

    (** Build a fresh log view and stamp it as `text`.
        Deferred until the TextModels-backed buffer is wired. *)
    PROCEDURE New*;
    BEGIN
    END New;

    (** Open the system log window — Config.Setup's final
        action.  Deferred until the TextView opens through
        StdDialog. *)
    PROCEDURE Open*;
    BEGIN
        Console.WriteString("[StdLog.Open]");
        Console.WriteLn
    END Open;

END StdLog.
