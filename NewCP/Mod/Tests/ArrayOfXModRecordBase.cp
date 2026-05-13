MODULE ArrayOfXModRecordBase;
(* Exported types for ArrayOfXModRecordProbe — sits in a
   separate module to force the cross-module path on the
   element type. *)

TYPE
    Entry* = RECORD
        stop*: INTEGER;
        kind*: SET
    END;

    Bag* = RECORD
        len*: INTEGER;
        items*: ARRAY 8 OF Entry
    END;

END ArrayOfXModRecordBase.
