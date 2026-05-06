MODULE Factorial;

IMPORT Log, WinView;

PROCEDURE Value*(n: INTEGER): INTEGER;
  VAR
    i: INTEGER;
    result: INTEGER;
BEGIN
  result := 1;
  i := 2;
  WHILE i <= n DO
    result := result * i;
    INC(i)
  END;
  RETURN result
END Value;

PROCEDURE OnRun*(name, payload: ARRAY OF SHORTCHAR);
  VAR result: INTEGER;
BEGIN
  result := Value(20);
  Log.String("20! = ");
  Log.Int(result, 0);
  Log.Ln;
  WinView.Render
END OnRun;

END Factorial.