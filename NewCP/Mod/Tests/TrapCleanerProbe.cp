MODULE TrapCleanerProbe;
(* Smoke probe for Kernel.PushTrapCleaner / PopTrapCleaner.

   Exercises the typed-cleaner wrapping pattern (subclass of
   Kernel.TrapCleanerDesc with an override of Cleanup) and
   verifies that balanced Push/Pop runs without error. The
   actual Cleanup invocation on a real trap can't be tested in
   unit-test scope because the trap aborts the process — that
   path is covered by the kernel_sys::tests module on the Rust
   side. *)

IMPORT Kernel;

TYPE
  CleanerDesc* = RECORD (Kernel.TrapCleanerDesc)
    counter*: INTEGER
  END;
  Cleaner*     = POINTER TO CleanerDesc;

(* Override of Kernel.TrapCleanerDesc.Cleanup. Bumps a counter so
   Rust-side run_trap_cleaners harness tests can observe the call,
   though no integration test actually triggers the trap path. *)
PROCEDURE (c: CleanerDesc) Cleanup*;
BEGIN
  INC(c.counter)
END Cleanup;

(** Push then Pop in matching order; no trap fires. Returns 1 on
    clean push/pop balance, 0 if anything goes sideways. *)
PROCEDURE BalancedPushPop*(): INTEGER;
  VAR a, b: Cleaner;
BEGIN
  NEW(a); a.counter := 0;
  NEW(b); b.counter := 0;
  Kernel.PushTrapCleaner(a);
  Kernel.PushTrapCleaner(b);
  Kernel.PopTrapCleaner(b);
  Kernel.PopTrapCleaner(a);
  IF a.counter # 0 THEN RETURN 0 END;     (* Cleanup must NOT have fired *)
  IF b.counter # 0 THEN RETURN 0 END;
  RETURN 1
END BalancedPushPop;

(** Same but with a single cleaner. *)
PROCEDURE SingletonPushPop*(): INTEGER;
  VAR c: Cleaner;
BEGIN
  NEW(c); c.counter := 0;
  Kernel.PushTrapCleaner(c);
  Kernel.PopTrapCleaner(c);
  IF c.counter # 0 THEN RETURN 0 END;
  RETURN 1
END SingletonPushPop;

END TrapCleanerProbe.
