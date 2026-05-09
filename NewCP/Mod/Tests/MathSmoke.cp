MODULE MathSmoke;
(**
   End-to-end JIT integration tests for Math + Strings. Each procedure
   returns LONGINT (i64) so the harness can assert via plain integer
   comparison. CP -> CP cross-module calls + CP -> Rust libm bindings
   are exercised through round-trips.
*)

IMPORT Math, Strings;

(* --- Math native module: cross-module REAL parameter passing --- *)

PROCEDURE Sqrt9*(): LONGINT;
BEGIN RETURN ENTIER(Math.Sqrt(9.0)) END Sqrt9;             (* 3 *)

PROCEDURE PiTimes2*(): LONGINT;
BEGIN RETURN ENTIER(Math.Pi() * 2.0) END PiTimes2;         (* 6 *)

PROCEDURE IntPow*(): LONGINT;
BEGIN RETURN ENTIER(Math.IntPower(2.0, 10)) END IntPow;    (* 1024 *)

PROCEDURE ExponentOf*(): LONGINT;
BEGIN RETURN Math.Exponent(8.0) END ExponentOf;            (* 3 since 8 = 1.0 * 2^3 *)

(* --- Strings real-number procs (CHAR family) --- *)

PROCEDURE StringsRoundTrip*(): LONGINT;
	(* Strings.StringToReal parses "3.14e2" as 314.0; ENTIER -> 314 *)
	VAR x: REAL; res: INTEGER;
BEGIN
	Strings.StringToReal("3.14e2", x, res);
	IF res # 0 THEN RETURN -1 END;
	RETURN ENTIER(x)
END StringsRoundTrip;

PROCEDURE RealToStringRoundTrip*(): LONGINT;
	(* Format then parse: 12.5 -> some scientific form -> 12.5; ENTIER -> 12 *)
	VAR buf: ARRAY 32 OF CHAR; x: REAL; res: INTEGER;
BEGIN
	Strings.RealToString(12.5, buf);
	Strings.StringToReal(buf, x, res);
	IF res # 0 THEN RETURN -1 END;
	RETURN ENTIER(x)
END RealToStringRoundTrip;

(* --- Strings real-number procs (SHORTCHAR family) --- *)

PROCEDURE ShortStrToRealCheck*(): LONGINT;
	(* SHORTCHAR parser: "42.5e1" -> 425.0 -> ENTIER 425 *)
	VAR x: REAL; res: INTEGER;
BEGIN
	Strings.ShortStrToReal("42.5e1", x, res);
	IF res # 0 THEN RETURN -1 END;
	RETURN ENTIER(x)
END ShortStrToRealCheck;

PROCEDURE RealToShortStrRoundTrip*(): LONGINT;
	(* Format into SHORTCHAR buf via Narrow, parse back via Widen + StringToReal:
	   7.5 -> "7.5..." -> 7.5 -> ENTIER 7. Exercises both bridge directions. *)
	VAR buf: ARRAY 32 OF SHORTCHAR; x: REAL; res: INTEGER;
BEGIN
	Strings.RealToShortStr(7.5, buf);
	Strings.ShortStrToReal(buf, x, res);
	IF res # 0 THEN RETURN -1 END;
	RETURN ENTIER(x)
END RealToShortStrRoundTrip;

END MathSmoke.
