MODULE TestBase;

TYPE
  Typeface* = ARRAY 64 OF CHAR;

  Base* = ABSTRACT RECORD
    typeface-: Typeface;
    size-:     INTEGER;
    style-:    SET;
    weight-:   INTEGER
  END;
  BasePtr* = POINTER TO Base;

PROCEDURE (b: Base) GetBounds* (OUT a, d, w: INTEGER), NEW, ABSTRACT;
PROCEDURE (b: Base) StringWidth* (IN s: ARRAY OF CHAR): INTEGER, NEW, ABSTRACT;
PROCEDURE (b: Base) SStringWidth* (IN s: ARRAY OF SHORTCHAR): INTEGER, NEW, ABSTRACT;
PROCEDURE (b: Base) IsAlien* (): BOOLEAN, NEW, ABSTRACT;

END TestBase.
