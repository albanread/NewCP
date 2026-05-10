MODULE M_Module_ForwardReference;
    PROCEDURE Outer (x: INTEGER): INTEGER;
    BEGIN RETURN Inner(x) * 7 END Outer;

    PROCEDURE Inner (x: INTEGER): INTEGER;
    BEGIN RETURN x + 4 END Inner;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN Outer(3)      (* Inner(3)*7 = 7*7 = 49 *)
    END Run;
END M_Module_ForwardReference.
