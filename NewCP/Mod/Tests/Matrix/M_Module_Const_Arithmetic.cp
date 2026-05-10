MODULE M_Module_Const_Arithmetic;
    CONST
        a = 10;
        b = 11;
        c = a * b;          (* CONST built from earlier CONSTs *)

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN c            (* 10 * 11 = 110 *)
    END Run;
END M_Module_Const_Arithmetic.
