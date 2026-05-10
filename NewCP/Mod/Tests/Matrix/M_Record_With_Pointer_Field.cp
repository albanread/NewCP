MODULE M_Record_With_Pointer_Field;
    TYPE
        InnerDesc = RECORD value: INTEGER END;
        Inner     = POINTER TO InnerDesc;
        OuterDesc = RECORD ptr: Inner; tag: INTEGER END;
        Outer     = POINTER TO OuterDesc;

    PROCEDURE Run* (): INTEGER;
        VAR o: Outer; score: INTEGER;
    BEGIN
        NEW(o);
        o.tag := 5;
        score := 0;
        IF o.ptr = NIL THEN score := score + 4 END;   (* zero-init NIL *)
        NEW(o.ptr);
        o.ptr.value := 5;
        IF o.ptr # NIL THEN score := score + o.ptr.value END;
        RETURN score                                  (* 4 + 5 = 9 *)
    END Run;
END M_Record_With_Pointer_Field.
