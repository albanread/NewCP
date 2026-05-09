MODULE XmodSubtype;
(* Blocker 2 repro: a concrete subclass declared here extends an
   abstract base from another module. Returning the subclass pointer
   where the base pointer is expected should typecheck. *)

IMPORT XmodSubtypeBase;

TYPE
    StubDesc* = RECORD (XmodSubtypeBase.BaseDesc)
        marker-: INTEGER
    END;
    Stub* = POINTER TO StubDesc;

(* Override of an abstract method declared in the imported base.
   Must NOT carry the NEW attribute — sema previously flagged this
   as "newly introduced method Greet must use NEW" because the
   override-detection walk only looked at the local module's methods. *)
PROCEDURE (s: StubDesc) Greet* (): INTEGER;
BEGIN RETURN 7 END Greet;

PROCEDURE Make* (): XmodSubtypeBase.Base;
    VAR s: Stub;
BEGIN
    NEW(s);
    RETURN s
END Make;

PROCEDURE TouchInheritedField* (): INTEGER;
    VAR s: Stub;
BEGIN
    NEW(s);
    s.res := 99;
    RETURN s.res
END TouchInheritedField;

END XmodSubtype.
