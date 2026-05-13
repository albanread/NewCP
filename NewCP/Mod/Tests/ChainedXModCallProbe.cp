MODULE ChainedXModCallProbe;
(* Repro for deferred_fixes #33: chained-call receiver
   through a cross-module method result.

   `o.Pick()` returns a `ChainedXModCallBase.Inner`, and
   `.Total()` is then called on that — the receiver of the
   second call is the RETURN type of the first call, which
   lives in the imported module.

   In-module chains work (deferred_fixes #22).  Today the
   cross-module version of the same shape falls through to
   the field-stub fallback.
*)

IMPORT ChainedXModCallBase;

PROCEDURE Run* (): INTEGER;
    VAR o: ChainedXModCallBase.Outer;
        i: ChainedXModCallBase.Inner;
BEGIN
    NEW(o);
    NEW(i);
    i.value := 6;
    o.slot := i;
    RETURN o.Pick().Total()         (* 6 * 7 = 42 *)
END Run;

END ChainedXModCallProbe.
