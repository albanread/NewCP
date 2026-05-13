MODULE AbstractLocalCallProbe;
TYPE
    BaseDesc* = ABSTRACT RECORD END;
    Base* = POINTER TO BaseDesc;

    LeafDesc* = RECORD (BaseDesc) hit*: INTEGER END;
    Leaf* = POINTER TO LeafDesc;

PROCEDURE (b: BaseDesc) Fire*, NEW, ABSTRACT;

PROCEDURE (l: LeafDesc) Fire*;
BEGIN
    l.hit := 99
END Fire;

PROCEDURE Run* (): INTEGER;
    VAR victim: Base; lf: Leaf;
BEGIN
    NEW(lf);
    lf.hit := 0;
    victim := lf;
    victim.Fire;       (* abstract-base method call through local var *)
    RETURN lf.hit
END Run;

END AbstractLocalCallProbe.
