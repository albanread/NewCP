MODULE SingleQuoteCharProbe;
PROCEDURE Run* (): INTEGER;
    VAR ch: CHAR;
BEGIN
    ch := 27X;       (* hex single quote *)
    IF ch = "'" THEN
        RETURN 1
    ELSE
        RETURN 0
    END
END Run;
END SingleQuoteCharProbe.
