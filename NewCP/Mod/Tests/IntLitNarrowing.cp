MODULE IntLitNarrowing;
(* Integer literal must adapt to the static type of the assignment
   target. CP allows `x := 0` for any integer-typed x because the
   literal is polymorphic until the context fixes its type. The
   bug was that integer literals always inferred as INTEGER (i64)
   and the rank-based compat check then refused INTEGER -> BYTE
   even when the value clearly fits.

   These procs return SYSTEM-free constants so the test only
   exercises the type-checking + lowering path, not any runtime
   narrowing. *)

PROCEDURE LitToByte* (): INTEGER;
    VAR x: BYTE;
BEGIN
    x := 0;
    x := 200;
    RETURN x          (* expect 200 *)
END LitToByte;

PROCEDURE LitToShortInt* (): INTEGER;
    VAR x: SHORTINT;
BEGIN
    x := 100;
    RETURN x          (* expect 100 *)
END LitToShortInt;

PROCEDURE LitOutOfRange* (): INTEGER;
    VAR x: BYTE;
BEGIN
    (* Compile-time constant 999 doesn't fit in BYTE. Sema should
       still reject this even after the polymorphism fix. We don't
       run this proc; it's here only so its diagnostics can be
       inspected manually. Intentionally commented out so this
       module compiles cleanly:
         x := 999;
    *)
    x := 1;
    RETURN x
END LitOutOfRange;

END IntLitNarrowing.
