MODULE BareCallXmodProbe;
(* Cross-module bare method-call repro. *)

IMPORT BareCallXmodBase;

PROCEDURE Run* (): INTEGER;
    VAR r: BareCallXmodBase.R;
BEGIN
    r.x := 0;
    r.Touch;     (* bare call — no parens *)
    RETURN r.x
END Run;

END BareCallXmodProbe.
