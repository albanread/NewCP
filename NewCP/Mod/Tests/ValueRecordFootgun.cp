MODULE ValueRecordFootgun;
(* Demonstrates that NewCP's "records pass by reference" ABI silently
   breaks BlackBox CP value-mode semantics. The callee declares
   value-mode `r`, expecting a private copy — but writes through `r`
   actually modify the CALLER's record, because at the ABI level we
   passed a reference. Sema should reject this. *)

TYPE
    Box = RECORD value*: INTEGER END;

(* Value-mode parameter — CP semantics say `b` is a private copy. *)
PROCEDURE Mutate (b: Box);
BEGIN
    b.value := 999     (* Should be a write to the local copy only. *)
END Mutate;

PROCEDURE Demo* (): INTEGER;
    VAR caller: Box;
BEGIN
    caller.value := 42;
    Mutate(caller);
    RETURN caller.value     (* CP says 42; NewCP currently returns 999 *)
END Demo;

END ValueRecordFootgun.
