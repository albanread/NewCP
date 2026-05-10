MODULE M_Type_Pointer_To_Pointer;
    TYPE
        InnerDesc = RECORD value: INTEGER END;
        Inner     = POINTER TO InnerDesc;
        OuterDesc = RECORD child: Inner END;
        Outer     = POINTER TO OuterDesc;

    PROCEDURE Run* (): INTEGER;
        VAR o: Outer;
    BEGIN
        NEW(o);
        NEW(o.child);
        o.child.value := 77;
        RETURN o.child.value
    END Run;
END M_Type_Pointer_To_Pointer.
