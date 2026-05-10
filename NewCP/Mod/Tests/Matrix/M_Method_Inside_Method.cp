MODULE M_Method_Inside_Method;
    TYPE
        BoxDesc = EXTENSIBLE RECORD x: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) Raw* (): INTEGER, NEW;
    BEGIN RETURN b.x END Raw;

    PROCEDURE (b: Box) Scaled* (factor: INTEGER): INTEGER, NEW;
    BEGIN RETURN b.Raw() * factor END Scaled;

    PROCEDURE Run* (): INTEGER;
        VAR b: Box;
    BEGIN
        NEW(b);
        b.x := 50;
        RETURN b.Scaled(4)                    (* 50 * 4 = 200 *)
    END Run;
END M_Method_Inside_Method.
