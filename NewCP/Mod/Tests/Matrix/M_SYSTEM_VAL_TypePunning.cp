MODULE M_SYSTEM_VAL_TypePunning;
    IMPORT SYSTEM;

    PROCEDURE Run* (): INTEGER;
        VAR s: SET; n: INTEGER;
    BEGIN
        s := {0, 3, 5};                  (* 1 + 8 + 32 = 41 *)
        n := SYSTEM.VAL(INTEGER, s);
        RETURN n
    END Run;
END M_SYSTEM_VAL_TypePunning.
