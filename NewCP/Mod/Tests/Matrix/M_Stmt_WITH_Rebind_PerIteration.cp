MODULE M_Stmt_WITH_Rebind_PerIteration;
    TYPE
        BaseDesc = EXTENSIBLE RECORD END;
        Base     = POINTER TO BaseDesc;
        ADesc    = RECORD (BaseDesc) av: INTEGER END;
        A        = POINTER TO ADesc;
        BDesc    = RECORD (BaseDesc) bv: INTEGER END;
        B        = POINTER TO BDesc;

    PROCEDURE Run* (): INTEGER;
        VAR arr: ARRAY 4 OF Base; pa: A; pb: B; p: Base; i, sum: INTEGER;
    BEGIN
        NEW(pa); pa.av :=    1;  arr[0] := pa;
        NEW(pb); pb.bv :=   10;  arr[1] := pb;
        NEW(pa); pa.av :=  100;  arr[2] := pa;
        NEW(pb); pb.bv := 1000;  arr[3] := pb;

        sum := 0;
        FOR i := 0 TO 3 DO
            p := arr[i];
            WITH p: A DO sum := sum + p.av
              |  p: B DO sum := sum + p.bv
            END
        END;
        RETURN sum                                    (* 1 + 10 + 100 + 1000 = 1111 *)
    END Run;
END M_Stmt_WITH_Rebind_PerIteration.
