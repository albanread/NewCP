MODULE M_Method_DispatchThrough_RecordField;
    TYPE
        ItemDesc = RECORD value: INTEGER END;
        Item     = POINTER TO ItemDesc;
        Bag      = RECORD obj: Item END;

    PROCEDURE (i: Item) Set* (v: INTEGER), NEW;
    BEGIN i.value := v END Set;

    PROCEDURE (i: Item) Get* (): INTEGER, NEW;
    BEGIN RETURN i.value END Get;

    PROCEDURE Run* (): INTEGER;
        VAR bag: Bag;
    BEGIN
        NEW(bag.obj);
        bag.obj.Set(42);
        RETURN bag.obj.Get()
    END Run;
END M_Method_DispatchThrough_RecordField.
