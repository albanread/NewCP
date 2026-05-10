MODULE M_Method_OnInheritedField;
    TYPE
        BaseDesc = ABSTRACT RECORD value*: INTEGER END;
        Base     = POINTER TO BaseDesc;
        SubDesc  = RECORD (BaseDesc) END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE (s: Sub) Doubled* (): INTEGER, NEW;
    BEGIN RETURN s.value * 2 END Doubled;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub;
    BEGIN
        NEW(s);
        s.value := 25;
        RETURN s.Doubled()                    (* 50 *)
    END Run;
END M_Method_OnInheritedField.
