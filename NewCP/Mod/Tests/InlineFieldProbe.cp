MODULE InlineFieldProbe;
TYPE
    Outer* = RECORD
        a: INTEGER;
        inner: RECORD x, y: SET END;
        b: INTEGER
    END;

PROCEDURE Run* (): INTEGER;
    VAR o: Outer; r: INTEGER;
BEGIN
    o.a := 1;
    o.inner.x := {0,1};
    o.inner.y := {2};
    o.b := 9;
    r := o.a * 1000 + o.b;
    IF 0 IN o.inner.x THEN r := r + 100 END;
    IF 2 IN o.inner.y THEN r := r + 10 END;
    RETURN r
END Run;
END InlineFieldProbe.
