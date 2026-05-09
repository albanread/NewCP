MODULE XMethodBase;
(* Cross-module vtable repro — abstract base with one concrete inherited
   method (Init) and one abstract method (Doubled). *)

TYPE
  BaseDesc* = ABSTRACT RECORD value*: INTEGER END;
  Base*     = POINTER TO BaseDesc;

(* Concrete method on the abstract base. Subclasses inherit this body. *)
PROCEDURE (b: BaseDesc) Init*(v: INTEGER), NEW;
BEGIN
  b.value := v
END Init;

(* Abstract method overridden by every concrete subclass. *)
PROCEDURE (b: BaseDesc) Doubled*(): INTEGER, NEW, ABSTRACT;

END XMethodBase.
