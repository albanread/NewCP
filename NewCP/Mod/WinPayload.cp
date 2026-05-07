MODULE WinPayload;

PROCEDURE GetStr*(payload, key: ARRAY OF SHORTCHAR;
                  VAR out: ARRAY OF SHORTCHAR): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END GetStr;

PROCEDURE GetInt*(payload, key: ARRAY OF SHORTCHAR;
                  VAR out: INTEGER): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  out := 0;
  ok := SHORT(0);
  RETURN ok
END GetInt;

PROCEDURE GetBool*(payload, key: ARRAY OF SHORTCHAR;
                   VAR out: INTSHORT): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  out := SHORT(0);
  ok := SHORT(0);
  RETURN ok
END GetBool;

END WinPayload.