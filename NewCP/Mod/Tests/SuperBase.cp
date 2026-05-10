MODULE SuperBase;
(* Cross-module base for SuperXmod.cp's super-call test. *)

    TYPE
        BaseDesc* = EXTENSIBLE RECORD
            traceBase*: INTEGER
        END;
        Base* = POINTER TO BaseDesc;

    PROCEDURE (b: BaseDesc) Bump* (v: INTEGER), NEW, EXTENSIBLE;
    BEGIN
        b.traceBase := b.traceBase + v
    END Bump;

END SuperBase.
