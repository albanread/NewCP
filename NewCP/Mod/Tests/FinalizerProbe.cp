MODULE FinalizerProbe;
(* Verify that a `Finalize` method declared on a record type runs
   when the GC reclaims an instance of that type.  The test
   allocates N instances, drops every reference, requests a
   collection, and checks the per-process counter the finalizer
   bumps. *)

IMPORT Kernel;

VAR
    finalizeCount-: INTEGER;

TYPE
    BoxDesc* = RECORD
        marker*: INTEGER
    END;
    Box* = POINTER TO BoxDesc;

PROCEDURE (b: BoxDesc) Finalize*, NEW;
BEGIN
    INC(finalizeCount)
END Finalize;

(** Allocate 64 boxes, drop the local references, force a GC, and
    return the delta (number of finalizers that fired during this
    call).  Self-contained so the integration test can observe the
    finalizer side effect through a single `run_function`. *)
PROCEDURE AllocAndDrop* (): INTEGER;
    VAR i, before: INTEGER; b: Box;
BEGIN
    before := finalizeCount;
    i := 0;
    WHILE i < 64 DO
        NEW(b);
        b.marker := i;
        INC(i)
    END;
    b := NIL;
    Kernel.Collect();
    RETURN finalizeCount - before
END AllocAndDrop;

PROCEDURE GetCount* (): INTEGER;
BEGIN RETURN finalizeCount END GetCount;

PROCEDURE ResetCount*;
BEGIN finalizeCount := 0 END ResetCount;

BEGIN
    finalizeCount := 0
END FinalizerProbe.
