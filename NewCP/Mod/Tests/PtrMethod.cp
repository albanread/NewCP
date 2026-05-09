MODULE PtrMethod;
(* Pointer-aliased OOP: simplest possible. *)

TYPE
    BoxDesc* = RECORD value: INTEGER END;
    Box*     = POINTER TO BoxDesc;

PROCEDURE (b: BoxDesc) Get*(): INTEGER, NEW;
BEGIN RETURN b.value END Get;

PROCEDURE (b: BoxDesc) Set*(v: INTEGER), NEW;
BEGIN b.value := v END Set;

PROCEDURE Run*(): INTEGER;
    VAR b: Box;
BEGIN
    NEW(b);
    b.Set(42);
    RETURN b.Get()
END Run;

END PtrMethod.
