MODULE M_Override_EmptyMethod_WithBody;
    TYPE
        BaseDesc = EXTENSIBLE RECORD touched: INTEGER END;
        Base     = POINTER TO BaseDesc;
        SubDesc  = RECORD (BaseDesc) END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Visit* (), NEW, EMPTY;

    PROCEDURE (s: Sub) Visit*;
    BEGIN s.touched := 41 END Visit;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub; p: Base;
    BEGIN
        NEW(s);
        p := s;
        p.Visit();              (* dispatches to Sub.Visit through Base ptr *)
        RETURN s.touched        (* 41 *)
    END Run;
END M_Override_EmptyMethod_WithBody.
