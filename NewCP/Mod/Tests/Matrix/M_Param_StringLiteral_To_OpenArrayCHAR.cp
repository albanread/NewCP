MODULE M_Param_StringLiteral_To_OpenArrayCHAR;
    PROCEDURE CountChars (IN s: ARRAY OF CHAR): INTEGER;
    BEGIN RETURN LEN(s) END CountChars;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN CountChars("hello")            (* 5 chars + trailing 0X = 6 *)
    END Run;
END M_Param_StringLiteral_To_OpenArrayCHAR.
