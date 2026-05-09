MODULE PtrAlloc;
(* Probe whether NEW + tagged-allocator works.
   BoxDesc has a method so it gets a TypeDesc and goes through new_rec. *)

TYPE
    BoxDesc* = RECORD value: INTEGER END;
    Box*     = POINTER TO BoxDesc;

(* Method present so the TypeDesc is emitted and Instr::New routes through __newcp_new_rec. *)
PROCEDURE (b: BoxDesc) Touch*(), NEW;
BEGIN
END Touch;

PROCEDURE Run*(): INTEGER;
    VAR b: Box;
BEGIN
    NEW(b);
    b.value := 42;
    RETURN b.value
END Run;

END PtrAlloc.
