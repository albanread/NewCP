MODULE LocalConstArrayDim;
(* Repro for the local-CONST-as-array-dimension bug. Module-level
   CONSTs flow through the array bound correctly, but a CONST
   declared inside a procedure does not — the alloca ends up
   sized for 0 (or some other wrong value) and accesses trap.

   Once fixed, all four procedures must return their first
   element after a write/read round-trip. *)

CONST ModN = 4;

(* Module-level CONST as array bound — known good. *)
PROCEDURE ModuleConstDim*(): INTEGER;
  VAR a: ARRAY ModN OF INTEGER;
BEGIN
  a[0] := 7; a[1] := 8; a[2] := 9; a[3] := 10;
  RETURN a[0] + a[1] + a[2] + a[3]   (* expect 34 *)
END ModuleConstDim;

(* Local CONST as array bound — currently broken. *)
PROCEDURE LocalConstDim*(): INTEGER;
  CONST N = 4;
  VAR a: ARRAY N OF INTEGER;
BEGIN
  a[0] := 7; a[1] := 8; a[2] := 9; a[3] := 10;
  RETURN a[0] + a[1] + a[2] + a[3]   (* expect 34 *)
END LocalConstDim;

(* Local CONST used in a value position (already works). *)
PROCEDURE LocalConstValue*(): INTEGER;
  CONST N = 4;
BEGIN
  RETURN N * N                       (* expect 16 *)
END LocalConstValue;

(* Local CONST as LEN of an open-array iteration. *)
PROCEDURE LocalConstLen*(): INTEGER;
  CONST N = 4;
  VAR a: ARRAY N OF INTEGER;
BEGIN
  RETURN LEN(a)                      (* expect 4 *)
END LocalConstLen;

END LocalConstArrayDim.
