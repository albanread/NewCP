MODULE PtrSet;
(* Pointer-aliased OOP — currently exercises the *allocation* + tagged-header
   path. Full virtual-dispatch tests (Run, Probe* below) remain as fixtures
   for the MCJIT vtable-relocation work; only `Probe` is wired into the test
   harness today. See docs/oop_runtime_status.md. *)

IMPORT SYSTEM;

TYPE
    BoxDesc* = RECORD value: INTEGER END;
    Box*     = POINTER TO BoxDesc;

PROCEDURE (b: BoxDesc) Set*(v: INTEGER), NEW;
BEGIN b.value := v END Set;

(* Verify the BlockHeader was set up: tag field at obj_ptr - 16 is non-zero
   (it's the TypeDesc address with the GC mark bit cleared). Returns 1 for
   pass, -1 for "tag is zero" (allocator didn't write the header). *)
PROCEDURE Probe*(): INTEGER;
    VAR b: Box; addr, tag: INTEGER;
BEGIN
    NEW(b);
    addr := SYSTEM.VAL(INTEGER, b);
    SYSTEM.GET(addr - 16, tag);
    IF tag = 0 THEN RETURN -1 END;
    RETURN 1
END Probe;

(* Walk the dispatch chain manually and return vtable[0]. Should be the
   address of `BoxDesc_Set` after the synthetic init function runs. *)
PROCEDURE ProbeFn0*(): INTEGER;
    VAR b: Box; addr, tag, descPtr, vtablePtr, fnPtr: INTEGER;
BEGIN
    NEW(b);
    addr := SYSTEM.VAL(INTEGER, b);
    SYSTEM.GET(addr - 16, tag);
    descPtr := tag DIV 2 * 2;
    SYSTEM.GET(descPtr + 32, vtablePtr);
    SYSTEM.GET(vtablePtr, fnPtr);
    RETURN fnPtr
END ProbeFn0;

(* Full method dispatch — fixture for the post-MCJIT-fix end-to-end test. *)
PROCEDURE Run*(): INTEGER;
    VAR b: Box;
BEGIN
    NEW(b);
    b.Set(42);
    RETURN b.value
END Run;

END PtrSet.
