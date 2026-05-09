MODULE XmodSubtypeBase;
(* Provides an abstract pointer-aliased base type for cross-module
   subclassing. Mirrors Files.Locator at minimum. *)

TYPE
    BaseDesc* = ABSTRACT RECORD res*: INTEGER END;
    Base*     = POINTER TO BaseDesc;

PROCEDURE (b: BaseDesc) Greet* (): INTEGER, NEW, ABSTRACT;

END XmodSubtypeBase.
