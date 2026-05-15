MODULE Out;
(* BB-faithful Out — the standard textual-output module.

   BB's Out routes through StdLog (a text view).  Until StdLog
   ports, this slice routes through Console.cp instead — same
   public surface, output lands on the host's stdout (or wherever
   Console is captured).  Once StdLog is up and the view-side
   formatting machinery (TextMappers.Formatter.WriteIntForm /
   WriteRealForm) ports, Out's bodies switch back to the
   StdLog-via-Formatter path without changing the surface.

   The 0FFX (`digitspace`) "non-breaking-space" character BB uses
   for the padding in formatted ints / reals is preserved at the
   signature level; the Console-routed bodies just emit it as a
   regular space to stdout for now. *)

IMPORT Console;

CONST
    digitspace* = 08FX;

(* `Open` in BB raises StdLog.  With no StdLog, the call is a
   no-op — Console output is always available. *)
PROCEDURE Open*;
BEGIN
    (* no-op — Console is always live *)
END Open;

PROCEDURE Char* (ch: CHAR);
BEGIN
    Console.WriteChar(ch)
END Char;

PROCEDURE Ln*;
BEGIN
    Console.WriteLn
END Ln;

PROCEDURE String* (IN str: ARRAY OF CHAR);
BEGIN
    Console.WriteString(str)
END String;

(* BB signature: `Int(i: LONGINT; n: INTEGER)` — `n` is the minimum
   field width.  We pad with space (not 8FX) since Console writes
   raw to stdout.  Negative n in BB means left-align; we accept
   any n but always right-align in this slice. *)
PROCEDURE Int* (i: INTEGER; n: INTEGER);
    VAR digits: ARRAY 24 OF CHAR;
        v, k, j, sign: INTEGER;
BEGIN
    sign := 1;
    v := i;
    IF v < 0 THEN sign := -1; v := -v END;
    k := 0;
    REPEAT
        digits[k] := CHR(ORD("0") + (v MOD 10));
        v := v DIV 10;
        INC(k)
    UNTIL v = 0;
    IF sign < 0 THEN digits[k] := "-"; INC(k) END;
    (* `k` now holds the digit count.  Pad to total width `n` with
       spaces.  Crucially we need a SEPARATE counter for the
       padding so `k` stays as "how many digits to emit" — without
       that, the digit-reverse loop walks past the end of valid
       chars and emits whatever was in the uninitialised slots. *)
    j := k;
    WHILE j < n DO
        Console.WriteChar(" ");
        INC(j)
    END;
    (* Emit digits reversed (they were stored
       least-significant-first). *)
    WHILE k > 0 DO
        DEC(k);
        Console.WriteChar(digits[k])
    END
END Int;

PROCEDURE Real* (x: REAL; n: INTEGER);
BEGIN
    (* Padding semantics matching Int.  Console.WriteReal does
       the actual formatting — width-honoring real output is a
       follow-up once TextMappers.Formatter.WriteRealForm ports. *)
    WHILE n > 16 DO Console.WriteChar(" "); DEC(n) END;
    Console.WriteReal(x)
END Real;

END Out.
