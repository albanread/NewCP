MODULE WinLoop;

IMPORT HostWindows;

CONST
  MaxHandlers* = 64;
  MaxCloseHandlers = 8;
  EventMax = 128;
  NameMax = 256;
  PayloadMax = 4096;

TYPE
  Handler* = PROCEDURE (name, payload: ARRAY OF SHORTCHAR);

VAR
  count: INTEGER;
  closeCount: INTEGER;
  events: ARRAY MaxHandlers OF ARRAY EventMax OF SHORTCHAR;
  handlers: ARRAY MaxHandlers OF Handler;
  closeHandlers: ARRAY MaxCloseHandlers OF Handler;

PROCEDURE StrEq(a, b: ARRAY OF SHORTCHAR): BOOLEAN;
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (a[i] = b[i]) & (a[i] # 0X) DO INC(i) END;
  RETURN a[i] = b[i]
END StrEq;

PROCEDURE CopyStr(src: ARRAY OF SHORTCHAR; VAR dst: ARRAY OF SHORTCHAR);
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (src[i] # 0X) & (i < EventMax - 1) DO
    dst[i] := src[i];
    INC(i)
  END;
  dst[i] := 0X
END CopyStr;

PROCEDURE Register*(event: ARRAY OF SHORTCHAR; h: Handler);
BEGIN
  IF count >= MaxHandlers THEN RETURN END;
  CopyStr(event, events[count]);
  handlers[count] := h;
  INC(count)
END Register;

PROCEDURE OnClose*(h: Handler);
BEGIN
  IF closeCount >= MaxCloseHandlers THEN RETURN END;
  closeHandlers[closeCount] := h;
  INC(closeCount)
END OnClose;

PROCEDURE Run*;
  VAR
    name: ARRAY NameMax OF SHORTCHAR;
    payload: ARRAY PayloadMax OF SHORTCHAR;
    ok: INTSHORT;
    i: INTEGER;
BEGIN
  LOOP
    ok := HostWindows.WaitNamedEvent(name, payload, -1);
    IF ok # 0 THEN
      IF StrEq(name, "__close_requested") OR StrEq(name, "__host_stopping") THEN
        i := 0;
        WHILE i < closeCount DO
          closeHandlers[i](name, payload);
          INC(i)
        END;
        EXIT
      ELSE
        i := 0;
        WHILE i < count DO
          IF StrEq(events[i], name) THEN
            handlers[i](name, payload)
          END;
          INC(i)
        END
      END
    END
  END
END Run;

BEGIN
  count := 0;
  closeCount := 0
END WinLoop.