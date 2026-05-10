MODULE M_Param_VAR_REAL;
    PROCEDURE Double (VAR x: REAL);
    BEGIN x := x * 2.0 END Double;

    PROCEDURE Run* (): LONGINT;
        VAR x: REAL;
    BEGIN
        x := 5.0;
        Double(x);
        RETURN ENTIER(x)            (* 10 *)
    END Run;
END M_Param_VAR_REAL.
