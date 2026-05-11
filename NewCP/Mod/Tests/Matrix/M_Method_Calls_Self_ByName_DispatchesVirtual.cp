MODULE M_Method_Calls_Self_ByName_DispatchesVirtual;
    TYPE
        BaseDesc = EXTENSIBLE RECORD END;
        Base     = POINTER TO BaseDesc;
        SubDesc  = RECORD (BaseDesc) END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Inner* (): INTEGER, NEW, EXTENSIBLE;
    BEGIN RETURN 1 END Inner;

    PROCEDURE (b: Base) Wrap* (): INTEGER, NEW;
    BEGIN RETURN b.Inner() * 10 END Wrap;

    PROCEDURE (s: Sub) Inner* (): INTEGER;
    BEGIN RETURN 7 END Inner;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub; b: Base;
    BEGIN
        NEW(s);
        b := s;
        RETURN b.Wrap()                               (* Wrap calls b.Inner(); virtual → Sub.Inner = 7; * 10 = 70 *)
    END Run;
END M_Method_Calls_Self_ByName_DispatchesVirtual.
