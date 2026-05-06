MODULE StringsTest;
(**
   StringsTest — tests the string operations that Log.cp uses:
   - hex char literals (0X = null, 0AX = newline, 30X = '0', 2DX = '-')
   - SHORTCHAR array copy loops
   - integer-to-string digit extraction
   - open-array string params
*)

IMPORT Console;

(* Copy src into dst (null-terminated, stop before dstCap). *)
PROCEDURE CopyStr(VAR dst: ARRAY OF SHORTCHAR; dstCap: INTEGER; src: ARRAY OF SHORTCHAR);
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (src[i] # 0X) & (i < dstCap - 1) DO
    dst[i] := src[i]; INC(i)
  END;
  dst[i] := 0X
END CopyStr;

(* Format integer n into buf, return length. *)
PROCEDURE IntToStr(n: INTEGER; VAR buf: ARRAY OF SHORTCHAR): INTEGER;
  VAR
    digits: ARRAY 24 OF SHORTCHAR;
    i, j: INTEGER;
    neg: BOOLEAN;
BEGIN
  i := 23; digits[i] := 0X;
  neg := n < 0;
  IF n = 0 THEN
    DEC(i); digits[i] := 30X   (* '0' *)
  ELSE
    IF neg THEN n := -n END;
    WHILE n > 0 DO
      DEC(i);
      digits[i] := SHORT(CHR(ORD('0') + n MOD 10));
      n := n DIV 10
    END;
    IF neg THEN DEC(i); digits[i] := 2DX END  (* '-' *)
  END;
  j := 0;
  WHILE digits[i] # 0X DO buf[j] := digits[i]; INC(i); INC(j) END;
  buf[j] := 0X;
  RETURN j
END IntToStr;

PROCEDURE Run*;
  VAR
    title:  ARRAY 64  OF SHORTCHAR;
    buf:    ARRAY 32  OF SHORTCHAR;
    n:      INTEGER;
BEGIN
  (* 1. hex null literal *)
  title[0] := 0X;
  Console.WriteShortString("null check: [");
  IF title[0] = 0X THEN
    Console.WriteShortString("ok]")
  ELSE
    Console.WriteShortString("FAIL]")
  END;
  Console.WriteLn;

  (* 2. copy a short string into a fixed array *)
  CopyStr(title, 64, "NewCP Log");
  Console.WriteShortString("title=["); Console.WriteShortString(title);
  Console.WriteShortString("]"); Console.WriteLn;

  (* 3. integer-to-string: 0 *)
  n := IntToStr(0, buf);
  Console.WriteShortString("IntToStr(0)=["); Console.WriteShortString(buf);
  Console.WriteShortString("]"); Console.WriteLn;

  (* 4. integer-to-string: positive *)
  n := IntToStr(42, buf);
  Console.WriteShortString("IntToStr(42)=["); Console.WriteShortString(buf);
  Console.WriteShortString("]"); Console.WriteLn;

  (* 5. integer-to-string: negative *)
  n := IntToStr(-7, buf);
  Console.WriteShortString("IntToStr(-7)=["); Console.WriteShortString(buf);
  Console.WriteShortString("]"); Console.WriteLn;

  (* 6. hex newline literal appended *)
  buf[0] := 65X;   (* 'A' *)
  buf[1] := 0AX;   (* LF *)
  buf[2] := 66X;   (* 'B' *)
  buf[3] := 0X;
  Console.WriteShortString("hex-lf=["); Console.WriteShortString(buf);
  Console.WriteShortString("]"); Console.WriteLn
END Run;

END StringsTest.
