MODULE Log;

CONST TextMax* = 4096;

VAR
  text*: ARRAY TextMax OF SHORTCHAR;
  textLen: INTEGER;

PROCEDURE Open*;
BEGIN
  textLen := 0;
  text[0] := 0X
END Open;

PROCEDURE Clear*;
BEGIN
  textLen := 0;
  text[0] := 0X
END Clear;

PROCEDURE PutStr(s: ARRAY OF SHORTCHAR);
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (s[i] # 0X) & (textLen < TextMax - 1) DO
    text[textLen] := s[i];
    INC(textLen);
    INC(i)
  END;
  text[textLen] := 0X
END PutStr;

PROCEDURE String*(s: ARRAY OF SHORTCHAR);
BEGIN
  PutStr(s)
END String;

PROCEDURE Ln*;
BEGIN
  IF textLen < TextMax - 1 THEN
    text[textLen] := 0AX;
    INC(textLen);
    text[textLen] := 0X
  END
END Ln;

PROCEDURE Int*(n, width: INTEGER);
  VAR
    digits: ARRAY 24 OF SHORTCHAR;
    out: ARRAY 28 OF SHORTCHAR;
    i, j, len: INTEGER;
    neg: BOOLEAN;
BEGIN
  i := 23;
  digits[i] := 0X;
  neg := n < 0;
  IF n = 0 THEN
    DEC(i);
    digits[i] := 30X
  ELSE
    IF neg THEN n := -n END;
    WHILE n > 0 DO
      DEC(i);
      digits[i] := SHORT(CHR(ORD('0') + n MOD 10));
      n := n DIV 10
    END;
    IF neg THEN
      DEC(i);
      digits[i] := 2DX
    END
  END;
  len := 23 - i;
  j := 0;
  WHILE len < width DO
    out[j] := 20X;
    INC(j);
    INC(len)
  END;
  WHILE digits[i] # 0X DO
    out[j] := digits[i];
    INC(i);
    INC(j)
  END;
  out[j] := 0X;
  PutStr(out)
END Int;

PROCEDURE OnClear*(name, payload: ARRAY OF SHORTCHAR);
BEGIN
  Clear
END OnClear;

BEGIN
  textLen := 0;
  text[0] := 0X
END Log.
