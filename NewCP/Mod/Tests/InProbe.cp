MODULE InProbe;
(* Smoke test for the BB-faithful In module.  In's input is driven
   by the focus text — `Controllers.FocusView()` returns NIL in
   this slice (no GUI focus routing yet), so In.Open sets
   Done := FALSE and all the read procs become no-ops.

   What we verify:
     1. The module loads (i.e. compiles + initialises).
     2. Open / Char / Int / LongInt / Real / String are all
        callable without trapping.
     3. With no focus, Done starts FALSE after Open.
     4. The read procs honour Done — they do not flip a CHAR /
        INTEGER OUT param when Done is FALSE.

   Returns:
     0   if Done was unexpectedly TRUE
     1   normal Done=FALSE path with all read procs no-ops
*)

    IMPORT In;

    PROCEDURE Run* (): INTEGER;
        VAR ch: CHAR; i: INTEGER; l: LONGINT; x: REAL;
            str: ARRAY 32 OF CHAR;
            sentinel: INTEGER;
    BEGIN
        In.Open;
        IF In.Done THEN RETURN 0 END;     (* expected: no focus -> Done FALSE *)

        (* Sentinel pattern: every read proc should leave its
           OUT slot untouched when Done is FALSE. *)
        ch := "?";
        i  := -1;
        l  := -2;
        x  := 3.5;
        str[0] := "Z"; str[1] := 0X;
        sentinel := 99;

        In.Char(ch);
        IF ch # "?" THEN RETURN -10 END;
        In.Int(i);
        IF i # -1 THEN RETURN -20 END;
        In.LongInt(l);
        IF l # -2 THEN RETURN -30 END;
        In.Real(x);
        IF x # 3.5 THEN RETURN -40 END;
        In.String(str);
        IF (str[0] # "Z") OR (str[1] # 0X) THEN RETURN -50 END;
        IF sentinel # 99 THEN RETURN -60 END;

        (* All read procs were callable and respected the Done
           flag; module is wired up. *)
        RETURN 1
    END Run;

END InProbe.
