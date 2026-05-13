MODULE PtrReceiverEqProbe;
TYPE
    FooDesc* = RECORD x*: INTEGER END;
    Foo* = POINTER TO FooDesc;

PROCEDURE (a: Foo) Eq* (b: Foo): BOOLEAN, NEW;
BEGIN
    RETURN a = b
END Eq;

PROCEDURE Run* (): INTEGER;
    VAR p, q: Foo;
BEGIN
    NEW(p); NEW(q);
    IF p.Eq(p) THEN
        IF p.Eq(q) THEN RETURN 0 ELSE RETURN 1 END
    ELSE RETURN 2 END
END Run;
END PtrReceiverEqProbe.
