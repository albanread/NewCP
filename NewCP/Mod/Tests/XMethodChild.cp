MODULE XMethodChild;
(* Concrete subclass of XMethodBase.BaseDesc.

   Calls the *inherited* concrete Init via virtual dispatch — the slot in
   ChildDesc's vtable should point at XMethodBase.BaseDesc.Init's body,
   which lives in another JIT module. Today the JIT vtable patcher leaves
   that slot pointing at __newcp_unimpl_method_trap. *)

IMPORT XMethodBase;

TYPE
  ChildDesc* = RECORD (XMethodBase.BaseDesc) END;
  Child*     = POINTER TO ChildDesc;

(* Override the abstract Doubled with a concrete implementation. *)
PROCEDURE (c: ChildDesc) Doubled*(): INTEGER;
BEGIN
  RETURN c.value * 2
END Doubled;

PROCEDURE Test*(): INTEGER;
  VAR c: Child;
BEGIN
  NEW(c);
  c.Init(21);             (* inherited — body in XMethodBase *)
  RETURN c.Doubled()      (* override here, expect 42 *)
END Test;

END XMethodChild.
