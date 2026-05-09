MODULE TestExtend;

IMPORT TestBase;

TYPE
  Derived*    = RECORD (TestBase.Base) END;
  DerivedPtr* = POINTER TO Derived;

PROCEDURE (d: Derived) GetBounds* (OUT a, dn, w: INTEGER);
BEGIN a := 0; dn := 0; w := 0 END GetBounds;

PROCEDURE (d: Derived) StringWidth* (IN s: ARRAY OF CHAR): INTEGER;
BEGIN RETURN 0 END StringWidth;

PROCEDURE (d: Derived) SStringWidth* (IN s: ARRAY OF SHORTCHAR): INTEGER;
BEGIN RETURN 0 END SStringWidth;

PROCEDURE (d: Derived) IsAlien* (): BOOLEAN;
BEGIN RETURN FALSE END IsAlien;

END TestExtend.
