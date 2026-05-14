MODULE InlineFixedArrayProbe;
(* Minimum repro: local var of a cross-module record type
   whose last field is `ARRAY <imported-const> OF INTEGER`.
   `box.tabW[i]` on it should work. *)

IMPORT TextSetters;

PROCEDURE Run* (): INTEGER;
    VAR box: TextSetters.LineBox;
BEGIN
    box.tabW[0] := -3;
    RETURN box.tabW[0]
END Run;

END InlineFixedArrayProbe.
