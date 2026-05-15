MODULE TestBareCall;
TYPE
    R = RECORD x: INTEGER END;
PROCEDURE (VAR r: R) Touch, NEW;
BEGIN r.x := 42 END Touch;
PROCEDURE Run* (): INTEGER;
    VAR r: R;
BEGIN
    r.x := 0;
    r.Touch;
    RETURN r.x
END Run;
END TestBareCall.
