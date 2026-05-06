MODULE Log;
(**
   Log — Oberon-style log view for the NewCP GUI.

   Maintains a text buffer and a window spec.  Every write procedure
   appends to the buffer then rebuilds the spec via WinSpec and pushes it
   to HostWindows so the log textarea updates immediately.

   Typical usage from App.Run:

     Log.SetTitle("My App");
     Log.Open;                         (* publishes the initial empty window *)
     Log.String("Application started"); Log.Ln;
     Log.Int(answer, 0); Log.Ln;

   The window always contains a single full-height read-only textarea whose
   widget id is "log".  Other modules can write to it freely; App.Run owns
   the event loop and calls HostWindows.WaitNamedEvent independently.
*)

IMPORT WinSpec, HostWindows;

CONST
  TextMax = 4096;   (* maximum log text length in SHORTCHAR units *)
  SpecMax = 10240;  (* large enough for JSON envelope + escaped text  *)

VAR
  text:    ARRAY TextMax OF SHORTCHAR;
  spec:    ARRAY SpecMax OF SHORTCHAR;
  textLen: INTEGER;
  title:   ARRAY 128 OF SHORTCHAR;
  opened:  BOOLEAN;

(* Rebuild the window spec and push it to the host.  Silently skips if
   Open has not been called yet or if the spec buffer is too small. *)
PROCEDURE Flush;
BEGIN
  IF ~opened THEN RETURN END;
  WinSpec.Begin(title);
  WinSpec.AddTextarea("log", "Log", text, 1);
  IF WinSpec.GetSpec(spec) # 0 THEN
    HostWindows.PublishUi(spec)
  END
END Flush;

(**
   SetTitle — set the window title shown in the title bar.
   Must be called before Open to take effect.
*)
PROCEDURE SetTitle*(t: ARRAY OF SHORTCHAR);
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (t[i] # 0X) & (i < 127) DO  (* title is ARRAY 128 OF SHORTCHAR *)
    title[i] := t[i]; INC(i)
  END;
  title[i] := 0X
END SetTitle;

(**
   Open — mark the log as active and publish the initial (empty) window.
   Call once from App.Run before entering the event loop.
*)
PROCEDURE Open*;
BEGIN
  opened := TRUE;
  Flush
END Open;

(**
   Clear — erase all log text and refresh the view.
*)
PROCEDURE Clear*;
BEGIN
  textLen := 0; text[0] := 0X;
  Flush
END Clear;

(* Private helper — appends s to the buffer and flushes.  Used by the
   public String* and Int* procedures to avoid the pervasive String type. *)
PROCEDURE PutStr(s: ARRAY OF SHORTCHAR);
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (s[i] # 0X) & (textLen < TextMax - 1) DO
    text[textLen] := s[i]; INC(textLen); INC(i)
  END;
  text[textLen] := 0X;
  Flush
END PutStr;

(**
   String — append a null-terminated short string and refresh.
*)
PROCEDURE String*(s: ARRAY OF SHORTCHAR);
BEGIN
  PutStr(s)
END String;

(**
   Ln — append a newline character and refresh.
*)
PROCEDURE Ln*;
BEGIN
  IF textLen < TextMax - 1 THEN
    text[textLen] := 0AX; INC(textLen); text[textLen] := 0X
  END;
  Flush
END Ln;

(**
   Int — append integer n right-aligned in a field of at least width chars.
   Pass width = 0 for no padding.
*)
PROCEDURE Int*(n, width: INTEGER);
  VAR
    digits: ARRAY 24 OF SHORTCHAR;
    out:    ARRAY 28 OF SHORTCHAR;
    i, j, len: INTEGER;
    neg: BOOLEAN;
BEGIN
  i := 23; digits[i] := 0X;
  neg := n < 0;
  IF n = 0 THEN
    DEC(i); digits[i] := 30X  (* '0' as SHORTCHAR *)
  ELSE
    IF neg THEN n := -n END;
    WHILE n > 0 DO
      DEC(i);
      digits[i] := SHORT(CHR(ORD('0') + n MOD 10));
      n := n DIV 10
    END;
    IF neg THEN DEC(i); digits[i] := 2DX END  (* '-' as SHORTCHAR *)
  END;
  (* build output: pad then digits *)
  len := 23 - i;
  j := 0;
  WHILE len < width DO out[j] := 20X; INC(j); INC(len) END;  (* ' ' as SHORTCHAR *)
  WHILE digits[i] # 0X DO out[j] := digits[i]; INC(i); INC(j) END;
  out[j] := 0X;
  PutStr(out)
END Int;

BEGIN
  textLen := 0; text[0] := 0X;
  title[0] := 0X;
  opened := FALSE
END Log.
