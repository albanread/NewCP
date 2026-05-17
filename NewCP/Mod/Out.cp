MODULE Out;
(* BB-faithful Out — the standard textual-output module.
   Routes through StdLog (a text-model-backed window).

   BB-faithful public surface is preserved so callers don't change.
   `Open` raises (or re-raises) the StdLog window; subsequent
   `String` / `Int` / `Real` / `Ln` / `Char` calls append to the
   log model.  Content appears at the next repaint of the log window.

   The 0FFX (`digitspace`) "non-breaking-space" character BB uses
   for the padding in formatted ints / reals is emitted as a plain
   space ('  ') until TextMappers.Formatter.WriteIntForm ports. *)

IMPORT StdLog;

CONST
    digitspace* = 08FX;

(** Open / raise the log window. *)
PROCEDURE Open*;
BEGIN
    StdLog.Open
END Open;

(** Append a single CHAR to the log. *)
PROCEDURE Char* (ch: CHAR);
    VAR s: ARRAY 2 OF CHAR;
BEGIN
    s[0] := ch;
    s[1] := 0X;
    StdLog.String(s)
END Char;

(** Append a line terminator. *)
PROCEDURE Ln*;
BEGIN
    StdLog.Ln
END Ln;

(** Append a wide string. *)
PROCEDURE String* (IN str: ARRAY OF CHAR);
BEGIN
    StdLog.String(str)
END String;

(** Append an integer with minimum field width `n`.
    Right-aligned; padded with spaces on the left.
    Negative `n` is treated as 0 (no padding). *)
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
    (* Pad to width n with spaces. *)
    j := k;
    WHILE j < n DO
        StdLog.SString(" ");
        INC(j)
    END;
    (* Append digits in reverse to StdLog. *)
    WHILE k > 0 DO
        DEC(k);
        Char(digits[k])
    END
END Int;

(** Append a real number formatted as an integer part and decimal
    fraction.  Full IEEE formatting is a follow-up once
    TextMappers.Formatter.WriteRealForm ports.  For now emits the
    integer part followed by ".0". *)
PROCEDURE Real* (x: REAL; n: INTEGER);
    VAR ipart: INTEGER;
BEGIN
    IF x < 0.0 THEN
        StdLog.SString("-");
        x := -x
    END;
    ipart := SHORT(ENTIER(x));
    Int(ipart, 0);
    StdLog.SString(".0")
END Real;

END Out.
