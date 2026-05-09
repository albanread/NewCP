MODULE MathSmoke;

IMPORT Math, Strings;

PROCEDURE Sqrt9*(): LONGINT;
	(* Math.Sqrt(9.0) -> 3.0; ENTIER returns 3 *)
BEGIN
	RETURN ENTIER(Math.Sqrt(9.0))
END Sqrt9;

PROCEDURE PiTimes2*(): LONGINT;
	(* ENTIER(Math.Pi() * 2) = ENTIER(6.283...) = 6 *)
BEGIN
	RETURN ENTIER(Math.Pi() * 2.0)
END PiTimes2;

PROCEDURE IntPow*(): LONGINT;
	(* Math.IntPower(2.0, 10) = 1024 *)
BEGIN
	RETURN ENTIER(Math.IntPower(2.0, 10))
END IntPow;

PROCEDURE ExponentOf*(): LONGINT;
	(* Math.Exponent(8.0) = 3 because 8 = 1.0 * 2^3 *)
BEGIN
	RETURN Math.Exponent(8.0)
END ExponentOf;

PROCEDURE StringsRoundTrip*(): LONGINT;
	(* Strings.StringToReal should parse "3.14e2" as 314.0; ENTIER -> 314 *)
	VAR x: REAL; res: INTEGER;
BEGIN
	Strings.StringToReal("3.14e2", x, res);
	IF res # 0 THEN RETURN -1 END;
	RETURN ENTIER(x)
END StringsRoundTrip;

END MathSmoke.
