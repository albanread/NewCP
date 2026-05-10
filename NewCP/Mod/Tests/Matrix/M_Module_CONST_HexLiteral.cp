MODULE M_Module_CONST_HexLiteral;
    CONST mask = 0FFH;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN mask                           (* 255 *)
    END Run;
END M_Module_CONST_HexLiteral.
