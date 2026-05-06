MODULE NestedTest;

PROCEDURE Outer*(x: INTEGER): INTEGER;
    VAR local: INTEGER;

    PROCEDURE Inner(y: INTEGER): INTEGER;
    BEGIN
        RETURN local + y
    END Inner;

BEGIN
    local := x * 2;
    RETURN Inner(1)
END Outer;

BEGIN
END NestedTest.
