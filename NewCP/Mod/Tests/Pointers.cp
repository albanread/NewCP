MODULE Pointers;
(* Pointer types, NEW, NIL checks. *)

TYPE
    Data* = RECORD
        value* : INTEGER;
        flag*  : BOOLEAN
    END;
    DataPtr* = POINTER TO Data;

PROCEDURE NewData*(v: INTEGER): DataPtr;
    VAR d: DataPtr;
BEGIN
    NEW(d);
    d.value := v;
    d.flag  := FALSE;
    RETURN d
END NewData;

PROCEDURE GetValue*(d: DataPtr): INTEGER;
BEGIN
    ASSERT(d # NIL);
    RETURN d.value
END GetValue;

PROCEDURE SetFlag*(d: DataPtr; f: BOOLEAN);
BEGIN
    ASSERT(d # NIL);
    d.flag := f
END SetFlag;

BEGIN
END Pointers.
