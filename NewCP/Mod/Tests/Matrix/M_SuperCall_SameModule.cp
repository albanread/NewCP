MODULE M_SuperCall_SameModule;
    TYPE
        BaseDesc* = EXTENSIBLE RECORD
            n*: INTEGER
        END;
        Base*     = POINTER TO BaseDesc;
        SubDesc*  = RECORD (BaseDesc) END;
        Sub*      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Add* (k: INTEGER), NEW, EXTENSIBLE;
    BEGIN b.n := b.n + k END Add;

    PROCEDURE (s: Sub) Add* (k: INTEGER);
    BEGIN
        s.Add^(k);          (* chain into Base.Add: n := n + k *)
        s.n := s.n + k      (* then double the effect *)
    END Add;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub;
    BEGIN
        NEW(s);
        s.n := 0;
        s.Add(15);          (* 0 + 15 (super) + 15 (override) = 30 *)
        RETURN s.n
    END Run;
END M_SuperCall_SameModule.
