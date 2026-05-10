MODULE M_Method_AbstractToConcrete_ChainedOverride;
    TYPE
        BaseDesc = ABSTRACT RECORD END;
        Base     = POINTER TO BaseDesc;
        MidDesc  = EXTENSIBLE RECORD (BaseDesc) END;
        Mid      = POINTER TO MidDesc;
        SubDesc  = RECORD (MidDesc) END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Pick* (): INTEGER, NEW, ABSTRACT;

    PROCEDURE (m: Mid) Pick* (): INTEGER, EXTENSIBLE;
    BEGIN RETURN 1 END Pick;

    PROCEDURE (s: Sub) Pick* (): INTEGER;
    BEGIN RETURN 999 END Pick;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub; p: Base;
    BEGIN
        NEW(s);
        p := s;
        RETURN p.Pick()
    END Run;
END M_Method_AbstractToConcrete_ChainedOverride.
