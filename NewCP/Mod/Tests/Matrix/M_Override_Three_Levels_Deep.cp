MODULE M_Override_Three_Levels_Deep;
    TYPE
        BaseDesc = EXTENSIBLE RECORD v: INTEGER END;
        Base     = POINTER TO BaseDesc;
        MidDesc  = EXTENSIBLE RECORD (BaseDesc) END;
        Mid      = POINTER TO MidDesc;
        SubDesc  = RECORD (MidDesc) END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Set* (n: INTEGER), NEW, EXTENSIBLE;
    BEGIN b.v := n END Set;

    PROCEDURE (m: Mid) Set* (n: INTEGER), EXTENSIBLE;
    BEGIN m.v := n * 10 END Set;

    PROCEDURE (s: Sub) Set* (n: INTEGER);
    BEGIN s.v := n * 100 END Set;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub; p: Base;
    BEGIN
        NEW(s);
        p := s;
        p.Set(42);             (* virtual: Sub.Set → s.v = 4200 *)
        RETURN s.v + 42        (* 4242 *)
    END Run;
END M_Override_Three_Levels_Deep.
