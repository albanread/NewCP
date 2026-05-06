MODULE WinDoc;

CONST
  MaxObservers = 64;
  DocIdMax = 128;

TYPE
  Observer* = PROCEDURE (docId, kind, detail: ARRAY OF SHORTCHAR);

VAR
  count: INTEGER;
  docIds: ARRAY MaxObservers OF ARRAY DocIdMax OF SHORTCHAR;
  observers: ARRAY MaxObservers OF Observer;

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
  WHILE (src[i] # 0X) & (i < DocIdMax - 1) DO
    dst[i] := src[i];
    INC(i)
  END;
  dst[i] := 0X
END CopyStr;

PROCEDURE AddObserver*(docId: ARRAY OF SHORTCHAR; p: Observer);
BEGIN
  IF count >= MaxObservers THEN RETURN END;
  CopyStr(docId, docIds[count]);
  observers[count] := p;
  INC(count)
END AddObserver;

PROCEDURE RemoveObserver*(docId: ARRAY OF SHORTCHAR; p: Observer);
  VAR i, j: INTEGER;
BEGIN
  i := 0;
  WHILE i < count DO
    IF StrEq(docIds[i], docId) THEN
      j := i;
      WHILE j < count - 1 DO
        CopyStr(docIds[j + 1], docIds[j]);
        observers[j] := observers[j + 1];
        INC(j)
      END;
      DEC(count)
    ELSE
      INC(i)
    END
  END
END RemoveObserver;

PROCEDURE Notify*(docId, kind, detail: ARRAY OF SHORTCHAR);
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE i < count DO
    IF StrEq(docIds[i], docId) THEN
      observers[i](docId, kind, detail)
    END;
    INC(i)
  END
END Notify;

BEGIN
  count := 0
END WinDoc.