MODULE BareCallXmodBase;
(* Defines a record + parameterless method.  Used by
   BareCallXmodProbe to test bare method-call dispatch when
   the receiver type is imported from another module. *)

TYPE
    R* = RECORD x*: INTEGER END;

PROCEDURE (VAR r: R) Touch* (), NEW;
BEGIN r.x := 42 END Touch;

END BareCallXmodBase.
