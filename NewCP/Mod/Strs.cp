MODULE Strs;
(* Exercises string-constant lowering — ConstStr values must become
   private [N x i8] globals with a GEP ptr to element 0.
   We use a const-string assignment into a local var to trigger ConstStr. *)

IMPORT StrBase;

PROCEDURE Greet*;
BEGIN
    StrBase.Print("hello")
END Greet;

END Strs.
