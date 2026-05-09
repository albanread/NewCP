MODULE Strings;
(**
	NewCP port of BlackBox `System/Mod/Strings.odc`.

	Two string flavors, both first-class in NewCP:

	  CHAR        32-bit UTF-32 scalar (U+0000 .. U+10FFFF)
	  String      pointer to null-terminated ARRAY OF CHAR

	  SHORTCHAR   8-bit byte (Latin-1 / ASCII / opaque-byte)
	  Shortstring pointer to null-terminated ARRAY OF SHORTCHAR

	Procedure parameters use open arrays (`ARRAY OF CHAR`,
	`ARRAY OF SHORTCHAR`) — the same calling convention works for fixed-
	size buffers and for dereferenced `String` / `Shortstring` values.

	Naming convention used here:
	  - bare names                operate on CHAR / ARRAY OF CHAR
	  - `*Short`                  operate on SHORTCHAR / ARRAY OF SHORTCHAR
	  - `IntToString`             writes wide (CHAR) digits
	  - `IntToShortStr`           writes byte (SHORTCHAR) digits
	  - `Widen` / `Narrow`        bridge byte <-> wide (Latin-1, lossless
	                              widening; narrowing flags non-ASCII codes)
	  - `Utf8ToString` / `StringToUtf8`
	                              bridge byte <-> wide via UTF-8 (lossless
	                              for any Unicode scalar)

	Differences from the BlackBox original:
	- Zero imports. BlackBox imports Kernel (Unicode tables, UTF-8 helpers)
	  and Math (real-number formatting). Neither exists in NewCP yet:
	    * Char classification is ASCII-only (code points < 128). For
	      SHORTCHAR the same ASCII rules apply (Latin-1 letters >= 80H
	      report as non-alpha until a real table is wired in).
	    * UTF-8 conversion is implemented inline.
	    * Real-number conversions (RealToString*, StringToReal) HALT(99)
	      until a Math module is ported.
	- INTEGER is 64-bit in NewCP, so StringToInt and StringToLInt have the
	  same range. StringToLInt forwards to StringToInt.

	Original module change log preserved at the bottom for provenance.
*)

	CONST
		charCode* = -1;
		decimal* = 10;
		hexadecimal* = -2;
		roman* = -3;
		digitspace* = 08FX;
		showBase* = TRUE;
		hideBase* = FALSE;

		minLongIntRev = "9223372036854775808";	(* abs(MIN(LONGINT)) for 64-bit *)

	VAR
		digits: ARRAY 17 OF CHAR;


	(* ------------------------------------------------------------------ *)
	(* Integer conversions                                                  *)
	(* ------------------------------------------------------------------ *)

	PROCEDURE IntToString* (x: LONGINT; OUT s: ARRAY OF CHAR);
		VAR j, k: INTEGER; ch: CHAR; a: ARRAY 32 OF CHAR;
	BEGIN
		IF x # MIN(LONGINT) THEN
			IF x < 0 THEN s[0] := "-"; k := 1; x := -x ELSE k := 0 END;
			j := 0;
			REPEAT
				a[j] := CHR(x MOD 10 + ORD("0"));
				x := x DIV 10;
				INC(j)
			UNTIL x = 0
		ELSE
			(* x = MIN(LONGINT) — cannot negate; emit reversed-digit constant *)
			s[0] := "-"; k := 1;
			j := 0;
			WHILE minLongIntRev[j] # 0X DO
				a[j] := minLongIntRev[j];
				INC(j)
			END
		END;
		ASSERT(k + j < LEN(s), 23);
		REPEAT
			DEC(j); ch := a[j]; s[k] := ch; INC(k)
		UNTIL j = 0;
		s[k] := 0X
	END IntToString;

	PROCEDURE IntToStringForm* (x: LONGINT; form, minWidth: INTEGER; fillCh: CHAR;
														showBase: BOOLEAN; OUT s: ARRAY OF CHAR);
		VAR base, i, j, k, si: INTEGER;
			mSign: BOOLEAN;
			a: ARRAY 128 OF CHAR;
			c1, c5, c10: CHAR;
	BEGIN
		ASSERT((form = charCode) OR (form = hexadecimal) OR (form = roman) OR ((form >= 2) & (form <= 16)), 20);
		ASSERT(minWidth >= 0, 22);
		IF form = charCode THEN base := 16
		ELSIF form = hexadecimal THEN base := 16
		ELSE base := form
		END;

		IF form = roman THEN
			ASSERT((x > 0) & (x < 3999), 21);
			base := 1000; i := 0; mSign := FALSE;
			WHILE (base > 0) & (x > 0) DO
				IF base = 1 THEN c1 := "I"; c5 := "V"; c10 := "X"
				ELSIF base = 10 THEN c1 := "X"; c5 := "L"; c10 := "C"
				ELSIF base = 100 THEN c1 := "C"; c5 := "D"; c10 := "M"
				ELSE c1 := "M"
				END;
				k := SHORT(x DIV base); x := x MOD base;
				IF k IN {4, 9} THEN a[i] := c1; INC(i) END;
				IF k IN {4 .. 8} THEN a[i] := c5; INC(i) END;
				IF k = 9 THEN
					a[i] := c10; INC(i)
				ELSIF k IN {1 .. 3, 6 .. 8} THEN
					j := k MOD 5;
					REPEAT a[i] := c1; INC(i); DEC(j) UNTIL j = 0
				END;
				base := base DIV 10
			END
		ELSIF (form = hexadecimal) OR (form = charCode) THEN
			i := 0; mSign := FALSE;
			IF showBase THEN DEC(minWidth) END;
			REPEAT
				a[i] := digits[x MOD base]; x := x DIV base; INC(i)
			UNTIL (x = 0) OR (x = -1) OR (i = LEN(a));
			IF x = -1 THEN fillCh := "F" END
		ELSE
			IF x < 0 THEN
				i := 0; mSign := TRUE; DEC(minWidth);
				REPEAT
					IF x MOD base = 0 THEN
						a[i] := digits[0]; x := x DIV base
					ELSE
						a[i] := digits[base - x MOD base]; x := x DIV base + 1
					END;
					INC(i)
				UNTIL (x = 0) OR (i = LEN(a))
			ELSE
				i := 0; mSign := FALSE;
				REPEAT
					a[i] := digits[x MOD base]; x := x DIV base; INC(i)
				UNTIL (x = 0) OR (i = LEN(a))
			END;
			IF showBase THEN
				DEC(minWidth);
				IF base < 10 THEN DEC(minWidth) ELSE DEC(minWidth, 2) END
			END
		END;
		si := 0;
		IF mSign & (fillCh = "0") & (si < LEN(s)) THEN
			s[si] := "-"; INC(si); mSign := FALSE
		END;
		WHILE minWidth > i DO
			IF si < LEN(s) THEN s[si] := fillCh; INC(si) END;
			DEC(minWidth)
		END;
		IF mSign & (si < LEN(s)) THEN s[si] := "-"; INC(si) END;
		IF form = roman THEN
			j := 0;
			WHILE j < i DO
				IF si < LEN(s) THEN s[si] := a[j]; INC(si) END;
				INC(j)
			END
		ELSE
			REPEAT
				DEC(i);
				IF si < LEN(s) THEN s[si] := a[i]; INC(si) END
			UNTIL i = 0
		END;
		IF showBase & (form # roman) THEN
			IF (form = charCode) & (si < LEN(s)) THEN
				s[si] := "X"; INC(si)
			ELSIF (form = hexadecimal) & (si < LEN(s)) THEN
				s[si] := "H"; INC(si)
			ELSIF (form < 10) & (si < LEN(s) - 1) THEN
				s[si] := "%"; s[si + 1] := digits[base]; INC(si, 2)
			ELSIF si < LEN(s) - 2 THEN
				s[si] := "%"; s[si + 1] := digits[base DIV 10]; s[si + 2] := digits[base MOD 10]; INC(si, 3)
			END
		END;
		IF si < LEN(s) THEN s[si] := 0X ELSE HALT(23) END
	END IntToStringForm;

	PROCEDURE StringToInt* (IN s: ARRAY OF CHAR; OUT x: LONGINT; OUT res: INTEGER);
		(* res = 0 ok, 1 overflow, 2 syntax error *)
		CONST hexLimit = MAX(LONGINT) DIV 8 + 1;
		VAR i, j, k, ndig: INTEGER;
			ch, top: CHAR;
			neg: BOOLEAN;
			base: INTEGER;
	BEGIN
		res := 0; i := 0; ch := s[0];
		WHILE (ch # 0X) & (ch <= " ") OR (ch = 8BX) OR (ch = 8FX) OR (ch = 0A0X) DO
			INC(i); ch := s[i]
		END;
		j := i; top := "0";
		WHILE (ch # 0X) & (ch # "H") & (ch # "X") & (ch # "%") DO
			IF ch > top THEN top := ch END;
			INC(j); ch := s[j]
		END;
		IF (ch = "H") OR (ch = "X") THEN
			x := 0; ch := s[i];
			IF ("0" <= ch) & (ch <= "9") OR ("A" <= ch) & (ch <= "F") THEN
				WHILE ch = "0" DO INC(i); ch := s[i] END;
				ndig := 0;
				WHILE (res = 0) & (("0" <= ch) & (ch <= "9") OR ("A" <= ch) & (ch <= "F")) DO
					IF ch < "A" THEN k := ORD(ch) - ORD("0")
					ELSE k := ORD(ch) - ORD("A") + 10
					END;
					IF ndig < 16 THEN
						x := x MOD hexLimit;
						IF x >= hexLimit DIV 2 THEN x := x - hexLimit END;
						x := x * 16 + k; INC(i); ch := s[i]
					ELSE
						res := 1
					END;
					INC(ndig)
				END;
				IF res = 0 THEN
					IF (ch # "H") & (ch # "X") OR (s[i + 1] # 0X) THEN res := 2 END
				END
			ELSE
				res := 2
			END
		ELSE
			IF ch = "%" THEN
				INC(j); ch := s[j]; base := 0;
				IF ("0" <= ch) & (ch <= "9") THEN
					k := ORD(ch) - ORD("0");
					REPEAT
						base := base * 10 + k;
						INC(j); ch := s[j]; k := ORD(ch) - ORD("0")
					UNTIL (ch < "0") OR (ch > "9") OR (base > (MAX(INTEGER) - k) DIV 10);
					IF ("0" <= ch) & (ch <= "9") THEN base := 0 END
				END
			ELSE
				base := 10
			END;

			IF (base < 2) OR (base > 16) THEN
				res := 2
			ELSIF (base <= 10) & (ORD(top) < base + ORD("0"))
				OR (base > 10) & (ORD(top) < base - 10 + ORD("A")) THEN
				x := 0; ch := s[i]; neg := FALSE;
				IF ch = "-" THEN INC(i); ch := s[i]; neg := TRUE
				ELSIF ch = "+" THEN INC(i); ch := s[i]
				END;
				WHILE (ch # 0X) & (ch <= " ") DO INC(i); ch := s[i] END;
				IF ("0" <= ch) & (ch <= "9") OR ("A" <= ch) & (ch <= "F") THEN
					IF ch <= "9" THEN k := ORD(ch) - ORD("0") ELSE k := ORD(ch) - ORD("A") + 10 END;
					WHILE (("0" <= ch) & (ch <= "9") OR ("A" <= ch) & (ch <= "F")) & (res = 0) DO
						IF x >= (MIN(LONGINT) + (base - 1) + k) DIV base THEN
							x := x * base - k; INC(i); ch := s[i];
							IF ch <= "9" THEN k := ORD(ch) - ORD("0") ELSE k := ORD(ch) - ORD("A") + 10 END
						ELSE
							res := 1
						END
					END
				ELSE
					res := 2
				END;
				IF res = 0 THEN
					IF ~neg THEN
						IF x > MIN(LONGINT) THEN x := -x ELSE res := 1 END
					END;
					IF (ch # 0X) & (ch # "%") THEN res := 2 END
				END
			ELSE
				res := 2
			END
		END
	END StringToInt;

	PROCEDURE StringToLInt* (IN s: ARRAY OF CHAR; OUT x: LONGINT; OUT res: INTEGER);
		(* INTEGER = LONGINT in NewCP; forward to StringToInt *)
	BEGIN
		StringToInt(s, x, res)
	END StringToLInt;


	(* ------------------------------------------------------------------ *)
	(* Real-number conversions — stubs until Math is ported                *)
	(* ------------------------------------------------------------------ *)

	PROCEDURE RealToString* (x: REAL; OUT s: ARRAY OF CHAR);
	BEGIN
		HALT(99)	(* TODO: needs Math.Exponent / Math.Mantissa / Math.IntPower *)
	END RealToString;

	PROCEDURE RealToStringForm* (x: REAL; precision, minW, expW: INTEGER;
															fillCh: CHAR; OUT s: ARRAY OF CHAR);
	BEGIN
		HALT(99)	(* TODO: needs Math *)
	END RealToStringForm;

	PROCEDURE StringToReal* (IN s: ARRAY OF CHAR; OUT x: REAL; OUT res: INTEGER);
	BEGIN
		HALT(99)	(* TODO: needs Math.IntPower *)
	END StringToReal;


	(* ------------------------------------------------------------------ *)
	(* Set conversions                                                      *)
	(* ------------------------------------------------------------------ *)

	PROCEDURE AppendChar (ch: CHAR; VAR pos: INTEGER; OUT s: ARRAY OF CHAR);
	BEGIN
		IF pos < LEN(s) - 1 THEN
			s[pos] := ch; INC(pos); s[pos] := 0X
		ELSE
			HALT(23)
		END
	END AppendChar;

	PROCEDURE AppendInt (n: INTEGER; VAR pos: INTEGER; OUT s: ARRAY OF CHAR);
		VAR buf: ARRAY 32 OF CHAR; i: INTEGER;
	BEGIN
		IntToString(n, buf);
		i := 0;
		WHILE buf[i] # 0X DO
			AppendChar(buf[i], pos, s); INC(i)
		END
	END AppendInt;

	PROCEDURE SetToString* (set: SET; OUT s: ARRAY OF CHAR);
		VAR i, lo, pos: INTEGER; first: BOOLEAN;
	BEGIN
		pos := 0; s[0] := "{"; INC(pos);
		first := TRUE; i := 0;
		WHILE i <= 31 DO
			IF i IN set THEN
				IF ~first THEN AppendChar(",", pos, s); AppendChar(" ", pos, s) END;
				lo := i;
				WHILE (i <= 31) & (i IN set) DO INC(i) END;
				AppendInt(lo, pos, s);
				IF i - 1 > lo THEN
					AppendChar(".", pos, s); AppendChar(".", pos, s);
					AppendInt(i - 1, pos, s)
				END;
				first := FALSE
			ELSE
				INC(i)
			END
		END;
		AppendChar("}", pos, s)
	END SetToString;

	PROCEDURE NextChar (IN s: ARRAY OF CHAR; VAR i: INTEGER; OUT ch: CHAR);
	BEGIN
		ch := s[i]; INC(i);
		WHILE (ch # 0X) & (ch <= " ") DO ch := s[i]; INC(i) END
	END NextChar;

	PROCEDURE ParseInt (IN s: ARRAY OF CHAR; VAR i: INTEGER; OUT n: INTEGER; OUT res: INTEGER);
		VAR ch: CHAR; v: INTEGER;
	BEGIN
		res := 0; v := 0; ch := s[i];
		IF (ch < "0") OR (ch > "9") THEN res := 2; RETURN END;
		WHILE ("0" <= ch) & (ch <= "9") DO
			v := v * 10 + (ORD(ch) - ORD("0"));
			INC(i); ch := s[i]
		END;
		n := v
	END ParseInt;

	PROCEDURE ParseRange (IN s: ARRAY OF CHAR; VAR i: INTEGER;
											OUT lo, hi: INTEGER; OUT res: INTEGER);
		VAR ch: CHAR;
	BEGIN
		ParseInt(s, i, lo, res);
		IF res # 0 THEN RETURN END;
		hi := lo;
		ch := s[i];
		WHILE (ch # 0X) & (ch <= " ") DO INC(i); ch := s[i] END;
		IF (ch = ".") & (s[i + 1] = ".") THEN
			INC(i, 2);
			ch := s[i];
			WHILE (ch # 0X) & (ch <= " ") DO INC(i); ch := s[i] END;
			ParseInt(s, i, hi, res)
		END
	END ParseRange;

	PROCEDURE StringToSet* (IN s: ARRAY OF CHAR; OUT set: SET; OUT res: INTEGER);
		VAR i, lo, hi, k: INTEGER; ch: CHAR;
	BEGIN
		set := {}; res := 0; i := 0;
		NextChar(s, i, ch);
		IF ch # "{" THEN res := 2; RETURN END;
		NextChar(s, i, ch);
		IF ch = "}" THEN RETURN END;
		DEC(i);	(* push back; ParseRange wants the digit *)
		LOOP
			(* skip leading WS already done above for "{}", but redo for digits *)
			ch := s[i];
			WHILE (ch # 0X) & (ch <= " ") DO INC(i); ch := s[i] END;
			ParseRange(s, i, lo, hi, res);
			IF res # 0 THEN RETURN END;
			IF (lo < 0) OR (hi > 31) OR (lo > hi) THEN res := 2; RETURN END;
			FOR k := lo TO hi DO INCL(set, k) END;
			ch := s[i];
			WHILE (ch # 0X) & (ch <= " ") DO INC(i); ch := s[i] END;
			IF ch = "}" THEN EXIT END;
			IF ch # "," THEN res := 2; RETURN END;
			INC(i)
		END
	END StringToSet;


	(* ------------------------------------------------------------------ *)
	(* Character classification (ASCII-only)                                *)
	(* ------------------------------------------------------------------ *)

	PROCEDURE IsUpper* (ch: CHAR): BOOLEAN;
	BEGIN
		RETURN ("A" <= ch) & (ch <= "Z")
	END IsUpper;

	PROCEDURE Upper* (ch: CHAR): CHAR;
	BEGIN
		IF ("a" <= ch) & (ch <= "z") THEN
			RETURN CHR(ORD(ch) - 32)
		END;
		RETURN ch
	END Upper;

	PROCEDURE IsLower* (ch: CHAR): BOOLEAN;
	BEGIN
		RETURN ("a" <= ch) & (ch <= "z")
	END IsLower;

	PROCEDURE Lower* (ch: CHAR): CHAR;
	BEGIN
		IF ("A" <= ch) & (ch <= "Z") THEN
			RETURN CHR(ORD(ch) + 32)
		END;
		RETURN ch
	END Lower;

	PROCEDURE IsAlpha* (ch: CHAR): BOOLEAN;
	BEGIN
		RETURN (("A" <= ch) & (ch <= "Z")) OR (("a" <= ch) & (ch <= "z"))
	END IsAlpha;

	PROCEDURE IsNumeric* (ch: CHAR): BOOLEAN;
	BEGIN
		RETURN ("0" <= ch) & (ch <= "9")
	END IsNumeric;

	PROCEDURE IsAlphaNumeric* (ch: CHAR): BOOLEAN;
	BEGIN
		RETURN IsAlpha(ch) OR IsNumeric(ch)
	END IsAlphaNumeric;

	PROCEDURE IsIdentStart* (ch: CHAR): BOOLEAN;
	BEGIN
		RETURN IsAlpha(ch) OR (ch = "_")
	END IsIdentStart;

	PROCEDURE IsIdent* (ch: CHAR): BOOLEAN;
	BEGIN
		RETURN IsAlpha(ch) OR IsNumeric(ch) OR (ch = "_")
	END IsIdent;

	PROCEDURE Valid* (ch: CHAR): BOOLEAN;
		(* TRUE for any code point representable in CP source text *)
	BEGIN
		RETURN (ch >= " ") OR (ch = 9X) OR (ch = 0AX) OR (ch = 0DX)
	END Valid;

	PROCEDURE ToUpper* (IN in: ARRAY OF CHAR; OUT out: ARRAY OF CHAR);
		VAR i: INTEGER;
	BEGIN
		i := 0;
		WHILE (in[i] # 0X) & (i < LEN(out) - 1) DO
			out[i] := Upper(in[i]); INC(i)
		END;
		out[i] := 0X
	END ToUpper;

	PROCEDURE ToLower* (IN in: ARRAY OF CHAR; OUT out: ARRAY OF CHAR);
		VAR i: INTEGER;
	BEGIN
		i := 0;
		WHILE (in[i] # 0X) & (i < LEN(out) - 1) DO
			out[i] := Lower(in[i]); INC(i)
		END;
		out[i] := 0X
	END ToLower;


	(* ------------------------------------------------------------------ *)
	(* UTF-8                                                                *)
	(* ------------------------------------------------------------------ *)

	PROCEDURE Utf8ToString* (IN in: ARRAY OF SHORTCHAR; OUT out: ARRAY OF CHAR;
												OUT res: INTEGER);
		(* res = 0 ok, 1 dst overflow, 2 malformed UTF-8 *)
		VAR i, j: INTEGER; b, b2: INTEGER; cp: INTEGER;
	BEGIN
		res := 0; i := 0; j := 0;
		WHILE in[i] # 0X DO
			IF j >= LEN(out) - 1 THEN res := 1; out[LEN(out) - 1] := 0X; RETURN END;
			b := ORD(in[i]); INC(i);
			IF b < 80H THEN
				cp := b
			ELSIF b < 0C0H THEN
				res := 2; RETURN	(* unexpected continuation byte *)
			ELSIF b < 0E0H THEN
				cp := b MOD 32;
				b2 := ORD(in[i]); INC(i);
				IF (b2 DIV 64) # 2 THEN res := 2; RETURN END;
				cp := cp * 64 + (b2 MOD 64)
			ELSIF b < 0F0H THEN
				cp := b MOD 16;
				b2 := ORD(in[i]); INC(i);
				IF (b2 DIV 64) # 2 THEN res := 2; RETURN END;
				cp := cp * 64 + (b2 MOD 64);
				b2 := ORD(in[i]); INC(i);
				IF (b2 DIV 64) # 2 THEN res := 2; RETURN END;
				cp := cp * 64 + (b2 MOD 64)
			ELSIF b < 0F8H THEN
				cp := b MOD 8;
				b2 := ORD(in[i]); INC(i);
				IF (b2 DIV 64) # 2 THEN res := 2; RETURN END;
				cp := cp * 64 + (b2 MOD 64);
				b2 := ORD(in[i]); INC(i);
				IF (b2 DIV 64) # 2 THEN res := 2; RETURN END;
				cp := cp * 64 + (b2 MOD 64);
				b2 := ORD(in[i]); INC(i);
				IF (b2 DIV 64) # 2 THEN res := 2; RETURN END;
				cp := cp * 64 + (b2 MOD 64)
			ELSE
				res := 2; RETURN
			END;
			out[j] := CHR(cp); INC(j)
		END;
		out[j] := 0X
	END Utf8ToString;

	PROCEDURE StringToUtf8* (IN in: ARRAY OF CHAR; OUT out: ARRAY OF SHORTCHAR;
												OUT res: INTEGER);
		(* res = 0 ok, 1 dst overflow *)
		VAR i, j, cp: INTEGER;

		PROCEDURE Put (b: INTEGER): BOOLEAN;
		BEGIN
			IF j >= LEN(out) - 1 THEN
				out[LEN(out) - 1] := 0X; res := 1;
				RETURN FALSE
			END;
			out[j] := SHORT(CHR(b)); INC(j);
			RETURN TRUE
		END Put;

	BEGIN
		res := 0; i := 0; j := 0;
		WHILE in[i] # 0X DO
			cp := ORD(in[i]); INC(i);
			IF cp < 80H THEN
				IF ~Put(cp) THEN RETURN END
			ELSIF cp < 800H THEN
				IF ~Put(0C0H + cp DIV 64) THEN RETURN END;
				IF ~Put(080H + cp MOD 64) THEN RETURN END
			ELSIF cp < 10000H THEN
				IF ~Put(0E0H + cp DIV 4096) THEN RETURN END;
				IF ~Put(080H + (cp DIV 64) MOD 64) THEN RETURN END;
				IF ~Put(080H + cp MOD 64) THEN RETURN END
			ELSE
				IF ~Put(0F0H + cp DIV 262144) THEN RETURN END;
				IF ~Put(080H + (cp DIV 4096) MOD 64) THEN RETURN END;
				IF ~Put(080H + (cp DIV 64) MOD 64) THEN RETURN END;
				IF ~Put(080H + cp MOD 64) THEN RETURN END
			END
		END;
		out[j] := 0X
	END StringToUtf8;


	(* ------------------------------------------------------------------ *)
	(* Substring operations                                                 *)
	(* ------------------------------------------------------------------ *)

	PROCEDURE Find* (IN s: ARRAY OF CHAR; IN pat: ARRAY OF CHAR;
									 from: INTEGER; OUT pos: INTEGER);
		(* pos = -1 if not found *)
		VAR i, j: INTEGER;
	BEGIN
		IF pat[0] = 0X THEN pos := from; RETURN END;
		i := from;
		WHILE s[i] # 0X DO
			j := 0;
			WHILE (pat[j] # 0X) & (s[i + j] = pat[j]) DO INC(j) END;
			IF pat[j] = 0X THEN pos := i; RETURN END;
			INC(i)
		END;
		pos := -1
	END Find;

	PROCEDURE Extract* (IN s: ARRAY OF CHAR; from, len: INTEGER; OUT out: ARRAY OF CHAR);
		(* Copy s[from .. from+len-1] into out, NUL-terminated. Stops at end of s. *)
		VAR i, n: INTEGER;
	BEGIN
		IF from < 0 THEN from := 0 END;
		IF len < 0 THEN len := 0 END;
		n := 0; i := 0;
		(* advance to `from` *)
		WHILE (i < from) & (s[i] # 0X) DO INC(i) END;
		WHILE (n < len) & (s[i] # 0X) & (n < LEN(out) - 1) DO
			out[n] := s[i]; INC(n); INC(i)
		END;
		out[n] := 0X
	END Extract;

	PROCEDURE Replace* (VAR s: ARRAY OF CHAR; from, len: INTEGER; IN repl: ARRAY OF CHAR);
		(* Replace s[from..from+len-1] with repl, in place. Truncates if dst too small. *)
		VAR slen, rlen, tail, shift, i: INTEGER;
	BEGIN
		IF from < 0 THEN from := 0 END;
		IF len < 0 THEN len := 0 END;
		slen := 0; WHILE s[slen] # 0X DO INC(slen) END;
		IF from > slen THEN from := slen END;
		IF from + len > slen THEN len := slen - from END;
		rlen := 0; WHILE repl[rlen] # 0X DO INC(rlen) END;

		tail := slen - from - len;
		(* New unclamped length = from + rlen + tail; capacity = LEN(s) - 1. *)
		IF from + rlen >= LEN(s) THEN
			(* Replacement alone fills the buffer *)
			rlen := LEN(s) - 1 - from;
			IF rlen < 0 THEN rlen := 0 END;
			tail := 0
		ELSIF from + rlen + tail >= LEN(s) THEN
			tail := LEN(s) - 1 - from - rlen
		END;

		shift := rlen - len;
		IF shift > 0 THEN
			(* move tail right; iterate from end *)
			i := tail - 1;
			WHILE i >= 0 DO
				s[from + rlen + i] := s[from + len + i]; DEC(i)
			END
		ELSIF shift < 0 THEN
			(* move tail left *)
			i := 0;
			WHILE i < tail DO
				s[from + rlen + i] := s[from + len + i]; INC(i)
			END
		END;
		(* splice replacement *)
		i := 0;
		WHILE i < rlen DO s[from + i] := repl[i]; INC(i) END;
		s[from + rlen + tail] := 0X
	END Replace;


	(* ------------------------------------------------------------------ *)
	(* Length helpers                                                       *)
	(* ------------------------------------------------------------------ *)

	PROCEDURE Length* (IN s: ARRAY OF CHAR): INTEGER;
		(* Number of CHAR units before the terminating 0X. NOT a UTF-8
		   byte count — for that, convert to SHORTCHAR via StringToUtf8. *)
		VAR i: INTEGER;
	BEGIN
		i := 0; WHILE s[i] # 0X DO INC(i) END; RETURN i
	END Length;

	PROCEDURE LengthShort* (IN s: ARRAY OF SHORTCHAR): INTEGER;
		(* Number of bytes before the terminating 0X. *)
		VAR i: INTEGER;
	BEGIN
		i := 0; WHILE s[i] # 0X DO INC(i) END; RETURN i
	END LengthShort;


	(* ------------------------------------------------------------------ *)
	(* Bridge: SHORTCHAR <-> CHAR (Latin-1)                                 *)
	(*   Widen: every byte becomes the same code point (lossless).          *)
	(*   Narrow: code points <= 0FFH become bytes; anything wider is        *)
	(*           replaced by '?' and `res` is set to 1.                     *)
	(* For Unicode-faithful conversion use Utf8ToString / StringToUtf8.     *)
	(* ------------------------------------------------------------------ *)

	PROCEDURE Widen* (IN in: ARRAY OF SHORTCHAR; OUT out: ARRAY OF CHAR);
		VAR i: INTEGER;
	BEGIN
		i := 0;
		WHILE (in[i] # 0X) & (i < LEN(out) - 1) DO
			out[i] := CHR(ORD(in[i])); INC(i)
		END;
		out[i] := 0X
	END Widen;

	PROCEDURE Narrow* (IN in: ARRAY OF CHAR; OUT out: ARRAY OF SHORTCHAR;
									OUT res: INTEGER);
		(* res = 0 ok, 1 some chars >= 100H were replaced by '?',
		   2 dst overflow *)
		VAR i, j, cp: INTEGER;
	BEGIN
		res := 0; i := 0; j := 0;
		WHILE in[i] # 0X DO
			IF j >= LEN(out) - 1 THEN
				out[LEN(out) - 1] := 0X; res := 2; RETURN
			END;
			cp := ORD(in[i]);
			IF cp < 100H THEN
				out[j] := SHORT(CHR(cp))
			ELSE
				out[j] := 3FX;	(* '?' *)
				IF res = 0 THEN res := 1 END
			END;
			INC(i); INC(j)
		END;
		out[j] := 0X
	END Narrow;


	(* ------------------------------------------------------------------ *)
	(* Integer conversions — SHORTCHAR variants                             *)
	(* These are byte-string twins of IntToString / IntToStringForm /        *)
	(* StringToInt. Useful for Console.WriteShortString and byte I/O.       *)
	(* ------------------------------------------------------------------ *)

	PROCEDURE IntToShortStr* (x: LONGINT; OUT s: ARRAY OF SHORTCHAR);
		VAR j, k: INTEGER; ch: SHORTCHAR; a: ARRAY 32 OF SHORTCHAR;
	BEGIN
		IF x # MIN(LONGINT) THEN
			IF x < 0 THEN s[0] := 2DX (* '-' *); k := 1; x := -x ELSE k := 0 END;
			j := 0;
			REPEAT
				a[j] := SHORT(CHR(x MOD 10 + ORD("0")));
				x := x DIV 10;
				INC(j)
			UNTIL x = 0
		ELSE
			s[0] := 2DX; k := 1;
			j := 0;
			WHILE minLongIntRev[j] # 0X DO
				a[j] := SHORT(minLongIntRev[j]);
				INC(j)
			END
		END;
		ASSERT(k + j < LEN(s), 23);
		REPEAT
			DEC(j); ch := a[j]; s[k] := ch; INC(k)
		UNTIL j = 0;
		s[k] := 0X
	END IntToShortStr;

	PROCEDURE IntToShortStrForm* (x: LONGINT; form, minWidth: INTEGER;
															 fillCh: SHORTCHAR; showBase: BOOLEAN;
															 OUT s: ARRAY OF SHORTCHAR);
		(* SHORTCHAR twin of IntToStringForm — same algorithm, byte buffers. *)
		VAR base, i, j, k, si: INTEGER;
			mSign: BOOLEAN;
			a: ARRAY 128 OF SHORTCHAR;
			c1, c5, c10: SHORTCHAR;
			sdig: SHORTCHAR;
	BEGIN
		ASSERT((form = charCode) OR (form = hexadecimal) OR (form = roman) OR ((form >= 2) & (form <= 16)), 20);
		ASSERT(minWidth >= 0, 22);
		IF form = charCode THEN base := 16
		ELSIF form = hexadecimal THEN base := 16
		ELSE base := form
		END;

		IF form = roman THEN
			ASSERT((x > 0) & (x < 3999), 21);
			base := 1000; i := 0; mSign := FALSE;
			WHILE (base > 0) & (x > 0) DO
				IF base = 1 THEN c1 := 49X; c5 := 56X; c10 := 58X (* I V X *)
				ELSIF base = 10 THEN c1 := 58X; c5 := 4CX; c10 := 43X (* X L C *)
				ELSIF base = 100 THEN c1 := 43X; c5 := 44X; c10 := 4DX (* C D M *)
				ELSE c1 := 4DX
				END;
				k := SHORT(x DIV base); x := x MOD base;
				IF k IN {4, 9} THEN a[i] := c1; INC(i) END;
				IF k IN {4 .. 8} THEN a[i] := c5; INC(i) END;
				IF k = 9 THEN
					a[i] := c10; INC(i)
				ELSIF k IN {1 .. 3, 6 .. 8} THEN
					j := k MOD 5;
					REPEAT a[i] := c1; INC(i); DEC(j) UNTIL j = 0
				END;
				base := base DIV 10
			END
		ELSIF (form = hexadecimal) OR (form = charCode) THEN
			i := 0; mSign := FALSE;
			IF showBase THEN DEC(minWidth) END;
			REPEAT
				a[i] := SHORT(digits[x MOD base]); x := x DIV base; INC(i)
			UNTIL (x = 0) OR (x = -1) OR (i = LEN(a));
			IF x = -1 THEN fillCh := 46X (* 'F' *) END
		ELSE
			IF x < 0 THEN
				i := 0; mSign := TRUE; DEC(minWidth);
				REPEAT
					IF x MOD base = 0 THEN
						a[i] := SHORT(digits[0]); x := x DIV base
					ELSE
						a[i] := SHORT(digits[base - x MOD base]); x := x DIV base + 1
					END;
					INC(i)
				UNTIL (x = 0) OR (i = LEN(a))
			ELSE
				i := 0; mSign := FALSE;
				REPEAT
					a[i] := SHORT(digits[x MOD base]); x := x DIV base; INC(i)
				UNTIL (x = 0) OR (i = LEN(a))
			END;
			IF showBase THEN
				DEC(minWidth);
				IF base < 10 THEN DEC(minWidth) ELSE DEC(minWidth, 2) END
			END
		END;
		si := 0;
		IF mSign & (fillCh = 30X (* '0' *)) & (si < LEN(s)) THEN
			s[si] := 2DX; INC(si); mSign := FALSE
		END;
		WHILE minWidth > i DO
			IF si < LEN(s) THEN s[si] := fillCh; INC(si) END;
			DEC(minWidth)
		END;
		IF mSign & (si < LEN(s)) THEN s[si] := 2DX; INC(si) END;
		IF form = roman THEN
			j := 0;
			WHILE j < i DO
				IF si < LEN(s) THEN s[si] := a[j]; INC(si) END;
				INC(j)
			END
		ELSE
			REPEAT
				DEC(i);
				IF si < LEN(s) THEN s[si] := a[i]; INC(si) END
			UNTIL i = 0
		END;
		IF showBase & (form # roman) THEN
			IF (form = charCode) & (si < LEN(s)) THEN
				s[si] := 58X (* 'X' *); INC(si)
			ELSIF (form = hexadecimal) & (si < LEN(s)) THEN
				s[si] := 48X (* 'H' *); INC(si)
			ELSIF (form < 10) & (si < LEN(s) - 1) THEN
				sdig := SHORT(digits[base]);
				s[si] := 25X (* '%' *); s[si + 1] := sdig; INC(si, 2)
			ELSIF si < LEN(s) - 2 THEN
				s[si] := 25X;
				s[si + 1] := SHORT(digits[base DIV 10]);
				s[si + 2] := SHORT(digits[base MOD 10]);
				INC(si, 3)
			END
		END;
		IF si < LEN(s) THEN s[si] := 0X ELSE HALT(23) END
	END IntToShortStrForm;

	PROCEDURE ShortStrToInt* (IN s: ARRAY OF SHORTCHAR; OUT x: LONGINT;
													 OUT res: INTEGER);
		(* Decimal / hex / base%-prefixed integer parser, byte input. *)
		CONST hexLimit = MAX(LONGINT) DIV 8 + 1;
		VAR i, j, k, ndig: INTEGER;
			ch, top: SHORTCHAR;
			neg: BOOLEAN;
			base: INTEGER;
	BEGIN
		res := 0; i := 0; ch := s[0];
		WHILE (ch # 0X) & (ch <= 20X (* ' ' *)) DO
			INC(i); ch := s[i]
		END;
		j := i; top := 30X;	(* '0' *)
		WHILE (ch # 0X) & (ch # 48X) & (ch # 58X) & (ch # 25X) DO	(* H X % *)
			IF ch > top THEN top := ch END;
			INC(j); ch := s[j]
		END;
		IF (ch = 48X) OR (ch = 58X) THEN
			x := 0; ch := s[i];
			IF (30X <= ch) & (ch <= 39X) OR (41X <= ch) & (ch <= 46X) THEN
				WHILE ch = 30X DO INC(i); ch := s[i] END;
				ndig := 0;
				WHILE (res = 0) & ((30X <= ch) & (ch <= 39X) OR (41X <= ch) & (ch <= 46X)) DO
					IF ch < 41X THEN k := ORD(ch) - ORD("0")
					ELSE k := ORD(ch) - ORD("A") + 10
					END;
					IF ndig < 16 THEN
						x := x MOD hexLimit;
						IF x >= hexLimit DIV 2 THEN x := x - hexLimit END;
						x := x * 16 + k; INC(i); ch := s[i]
					ELSE
						res := 1
					END;
					INC(ndig)
				END;
				IF res = 0 THEN
					IF (ch # 48X) & (ch # 58X) OR (s[i + 1] # 0X) THEN res := 2 END
				END
			ELSE
				res := 2
			END
		ELSE
			IF ch = 25X THEN
				INC(j); ch := s[j]; base := 0;
				IF (30X <= ch) & (ch <= 39X) THEN
					k := ORD(ch) - ORD("0");
					REPEAT
						base := base * 10 + k;
						INC(j); ch := s[j]; k := ORD(ch) - ORD("0")
					UNTIL (ch < 30X) OR (ch > 39X) OR (base > (MAX(INTEGER) - k) DIV 10);
					IF (30X <= ch) & (ch <= 39X) THEN base := 0 END
				END
			ELSE
				base := 10
			END;

			IF (base < 2) OR (base > 16) THEN
				res := 2
			ELSIF (base <= 10) & (ORD(top) < base + ORD("0"))
				OR (base > 10) & (ORD(top) < base - 10 + ORD("A")) THEN
				x := 0; ch := s[i]; neg := FALSE;
				IF ch = 2DX THEN INC(i); ch := s[i]; neg := TRUE
				ELSIF ch = 2BX (* '+' *) THEN INC(i); ch := s[i]
				END;
				WHILE (ch # 0X) & (ch <= 20X) DO INC(i); ch := s[i] END;
				IF (30X <= ch) & (ch <= 39X) OR (41X <= ch) & (ch <= 46X) THEN
					IF ch <= 39X THEN k := ORD(ch) - ORD("0") ELSE k := ORD(ch) - ORD("A") + 10 END;
					WHILE ((30X <= ch) & (ch <= 39X) OR (41X <= ch) & (ch <= 46X)) & (res = 0) DO
						IF x >= (MIN(LONGINT) + (base - 1) + k) DIV base THEN
							x := x * base - k; INC(i); ch := s[i];
							IF ch <= 39X THEN k := ORD(ch) - ORD("0") ELSE k := ORD(ch) - ORD("A") + 10 END
						ELSE
							res := 1
						END
					END
				ELSE
					res := 2
				END;
				IF res = 0 THEN
					IF ~neg THEN
						IF x > MIN(LONGINT) THEN x := -x ELSE res := 1 END
					END;
					IF (ch # 0X) & (ch # 25X) THEN res := 2 END
				END
			ELSE
				res := 2
			END
		END
	END ShortStrToInt;


	(* ------------------------------------------------------------------ *)
	(* Char classification (SHORTCHAR / ASCII)                              *)
	(* SHORTCHAR is 8-bit; the rules below cover ASCII (< 80X). Latin-1     *)
	(* upper-half letters are not classified yet — would need a 256-entry   *)
	(* table.                                                                *)
	(* ------------------------------------------------------------------ *)

	PROCEDURE IsUpperShort* (ch: SHORTCHAR): BOOLEAN;
	BEGIN
		RETURN (41X <= ch) & (ch <= 5AX)
	END IsUpperShort;

	PROCEDURE UpperShort* (ch: SHORTCHAR): SHORTCHAR;
	BEGIN
		IF (61X <= ch) & (ch <= 7AX) THEN
			RETURN SHORT(CHR(ORD(ch) - 32))
		END;
		RETURN ch
	END UpperShort;

	PROCEDURE IsLowerShort* (ch: SHORTCHAR): BOOLEAN;
	BEGIN
		RETURN (61X <= ch) & (ch <= 7AX)
	END IsLowerShort;

	PROCEDURE LowerShort* (ch: SHORTCHAR): SHORTCHAR;
	BEGIN
		IF (41X <= ch) & (ch <= 5AX) THEN
			RETURN SHORT(CHR(ORD(ch) + 32))
		END;
		RETURN ch
	END LowerShort;

	PROCEDURE IsAlphaShort* (ch: SHORTCHAR): BOOLEAN;
	BEGIN
		RETURN ((41X <= ch) & (ch <= 5AX)) OR ((61X <= ch) & (ch <= 7AX))
	END IsAlphaShort;

	PROCEDURE IsNumericShort* (ch: SHORTCHAR): BOOLEAN;
	BEGIN
		RETURN (30X <= ch) & (ch <= 39X)
	END IsNumericShort;

	PROCEDURE IsAlphaNumericShort* (ch: SHORTCHAR): BOOLEAN;
	BEGIN
		RETURN IsAlphaShort(ch) OR IsNumericShort(ch)
	END IsAlphaNumericShort;

	PROCEDURE IsIdentStartShort* (ch: SHORTCHAR): BOOLEAN;
	BEGIN
		RETURN IsAlphaShort(ch) OR (ch = 5FX (* '_' *))
	END IsIdentStartShort;

	PROCEDURE IsIdentShort* (ch: SHORTCHAR): BOOLEAN;
	BEGIN
		RETURN IsAlphaShort(ch) OR IsNumericShort(ch) OR (ch = 5FX)
	END IsIdentShort;

	PROCEDURE ToUpperShort* (IN in: ARRAY OF SHORTCHAR;
													OUT out: ARRAY OF SHORTCHAR);
		VAR i: INTEGER;
	BEGIN
		i := 0;
		WHILE (in[i] # 0X) & (i < LEN(out) - 1) DO
			out[i] := UpperShort(in[i]); INC(i)
		END;
		out[i] := 0X
	END ToUpperShort;

	PROCEDURE ToLowerShort* (IN in: ARRAY OF SHORTCHAR;
													OUT out: ARRAY OF SHORTCHAR);
		VAR i: INTEGER;
	BEGIN
		i := 0;
		WHILE (in[i] # 0X) & (i < LEN(out) - 1) DO
			out[i] := LowerShort(in[i]); INC(i)
		END;
		out[i] := 0X
	END ToLowerShort;


	(* ------------------------------------------------------------------ *)
	(* Substring operations — SHORTCHAR variants                            *)
	(* ------------------------------------------------------------------ *)

	PROCEDURE FindShort* (IN s: ARRAY OF SHORTCHAR; IN pat: ARRAY OF SHORTCHAR;
											 from: INTEGER; OUT pos: INTEGER);
		(* pos = -1 if not found *)
		VAR i, j: INTEGER;
	BEGIN
		IF pat[0] = 0X THEN pos := from; RETURN END;
		i := from;
		WHILE s[i] # 0X DO
			j := 0;
			WHILE (pat[j] # 0X) & (s[i + j] = pat[j]) DO INC(j) END;
			IF pat[j] = 0X THEN pos := i; RETURN END;
			INC(i)
		END;
		pos := -1
	END FindShort;

	PROCEDURE ExtractShort* (IN s: ARRAY OF SHORTCHAR; from, len: INTEGER;
													OUT out: ARRAY OF SHORTCHAR);
		VAR i, n: INTEGER;
	BEGIN
		IF from < 0 THEN from := 0 END;
		IF len < 0 THEN len := 0 END;
		n := 0; i := 0;
		WHILE (i < from) & (s[i] # 0X) DO INC(i) END;
		WHILE (n < len) & (s[i] # 0X) & (n < LEN(out) - 1) DO
			out[n] := s[i]; INC(n); INC(i)
		END;
		out[n] := 0X
	END ExtractShort;

	PROCEDURE ReplaceShort* (VAR s: ARRAY OF SHORTCHAR; from, len: INTEGER;
													IN repl: ARRAY OF SHORTCHAR);
		VAR slen, rlen, tail, shift, i: INTEGER;
	BEGIN
		IF from < 0 THEN from := 0 END;
		IF len < 0 THEN len := 0 END;
		slen := 0; WHILE s[slen] # 0X DO INC(slen) END;
		IF from > slen THEN from := slen END;
		IF from + len > slen THEN len := slen - from END;
		rlen := 0; WHILE repl[rlen] # 0X DO INC(rlen) END;

		tail := slen - from - len;
		IF from + rlen >= LEN(s) THEN
			rlen := LEN(s) - 1 - from;
			IF rlen < 0 THEN rlen := 0 END;
			tail := 0
		ELSIF from + rlen + tail >= LEN(s) THEN
			tail := LEN(s) - 1 - from - rlen
		END;

		shift := rlen - len;
		IF shift > 0 THEN
			i := tail - 1;
			WHILE i >= 0 DO
				s[from + rlen + i] := s[from + len + i]; DEC(i)
			END
		ELSIF shift < 0 THEN
			i := 0;
			WHILE i < tail DO
				s[from + rlen + i] := s[from + len + i]; INC(i)
			END
		END;
		i := 0;
		WHILE i < rlen DO s[from + i] := repl[i]; INC(i) END;
		s[from + rlen + tail] := 0X
	END ReplaceShort;


	(* ------------------------------------------------------------------ *)
	(* Module init                                                          *)
	(* ------------------------------------------------------------------ *)

	PROCEDURE Init;
	BEGIN
		digits := "0123456789ABCDEF"
	END Init;

BEGIN
	Init
END Strings.

(*
Original BlackBox change log (preserved):
- 20141027, center #19, full Unicode support for Component Pascal identifiers added
- 20150130, center #27, Adding SET conversion to the module Strings
- 20150130, center #28, Fixing a bug in Strings.Replace in case of truncation
*)
