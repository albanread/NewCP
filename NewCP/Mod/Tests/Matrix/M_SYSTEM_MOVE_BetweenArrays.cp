MODULE M_SYSTEM_MOVE_BetweenArrays;
    IMPORT SYSTEM;

    PROCEDURE Run* (): INTEGER;
        VAR src, dst: ARRAY 4 OF BYTE; i, sum: INTEGER;
    BEGIN
        src[0] := SHORT(SHORT(SHORT(1)));
        src[1] := SHORT(SHORT(SHORT(2)));
        src[2] := SHORT(SHORT(SHORT(3)));
        src[3] := SHORT(SHORT(SHORT(4)));
        SYSTEM.MOVE(SYSTEM.ADR(src[0]), SYSTEM.ADR(dst[0]), 4);
        sum := 0;
        FOR i := 0 TO 3 DO sum := sum + dst[i] END;
        RETURN sum                                    (* 1+2+3+4 = 10 *)
    END Run;
END M_SYSTEM_MOVE_BetweenArrays.
