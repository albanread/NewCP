MODULE M_MultiLevel_Inheritance_Dispatch;
    TYPE
        BaseDesc* = ABSTRACT RECORD tag*: INTEGER END;
        Base*     = POINTER TO BaseDesc;
        MidDesc*  = EXTENSIBLE RECORD (BaseDesc) END;
        Mid*      = POINTER TO MidDesc;
        SubDesc*  = RECORD (MidDesc) END;
        Sub*      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Set* (v: INTEGER), NEW, ABSTRACT;

    PROCEDURE (m: Mid) Set* (v: INTEGER), EXTENSIBLE;
    BEGIN m.tag := v * 10 END Set;

    PROCEDURE (s: Sub) Set* (v: INTEGER);
    BEGIN s.tag := v * 100 + 7 END Set;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub; m: Mid;
    BEGIN
        NEW(s);
        s.Set(1);              (* hits Sub.Set: tag = 107 *)
        m := s;                (* widen pointer; dynamic type still Sub *)
        m.Set(3);              (* virtual: hits Sub.Set: tag = 307 *)
        RETURN s.tag - 170     (* 307 - 170 = 137 *)
    END Run;
END M_MultiLevel_Inheritance_Dispatch.
