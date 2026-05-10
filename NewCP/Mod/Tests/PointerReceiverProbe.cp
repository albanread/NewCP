MODULE PointerReceiverProbe;
(* Verify methods can be declared with the BlackBox-style pointer-
   alias receiver (`(s: Sub) Method`) instead of the record-Desc
   form (`(s: SubDesc) Method`).  Sema used to reject the pointer
   form because `find_record_decl` didn't chase pointer aliases to
   their underlying record. *)

    TYPE
        BaseDesc* = ABSTRACT RECORD
            tag*: INTEGER
        END;
        Base* = POINTER TO BaseDesc;

        SubDesc* = RECORD (BaseDesc)
            extra*: INTEGER
        END;
        Sub* = POINTER TO SubDesc;

    (* Receiver typed as the pointer alias `Base`. *)
    PROCEDURE (b: Base) Greet* (v: INTEGER), NEW, ABSTRACT;

    (* Override using the pointer alias `Sub`. *)
    PROCEDURE (s: Sub) Greet* (v: INTEGER);
    BEGIN
        s.tag   := v;
        s.extra := v * 10
    END Greet;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub;
    BEGIN
        NEW(s);
        s.Greet(7);
        RETURN (s.tag * 100) + s.extra
    END Run;

END PointerReceiverProbe.
