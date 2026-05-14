MODULE TextControllersStdCtrl;
(* End-to-end probe for the concrete StdCtrl + StdDirectory bodies.

   Exercises the full controller-allocation + caret/selection
   round-trip path through the abstract Controller / Directory
   interfaces — i.e. the actual virtual dispatch chain the
   framework uses, not just direct field access.

   Verifies:
     - `dir.New()` lands on `StdDirectory.NewController({})` which
       returns a fresh StdCtrl;
     - the fresh controller reports `carPos = none` (BB-faithful
       "no caret" sentinel);
     - `c.SetCaret(pos)` writes through to `carPos`;
     - `c.SetSelection(beg, end)` writes through to selBeg/selEnd;
     - `c.GetSelection` reads back the same values.
*)

IMPORT TextControllers;

PROCEDURE Run* (): INTEGER;
    VAR c: TextControllers.Controller;
        beg, end: INTEGER;
        result: INTEGER;
BEGIN
    result := 0;

    (* Allocate through the abstract directory — virtual dispatch
       reaches StdDirectory.NewController. *)
    c := TextControllers.dir.New();
    IF c = NIL THEN RETURN -1 END;

    (* Fresh controller carries the BB-faithful "no caret" sentinel. *)
    IF c.CaretPos() = TextControllers.none THEN
        INC(result, 1)
    END;

    (* Caret round-trip via virtual dispatch. *)
    c.SetCaret(42);
    IF c.CaretPos() = 42 THEN
        INC(result, 10)
    END;

    (* SetCaret(none) returns to the no-caret state. *)
    c.SetCaret(TextControllers.none);
    IF c.CaretPos() = TextControllers.none THEN
        INC(result, 100)
    END;

    (* Selection round-trip. *)
    c.SetSelection(7, 19);
    c.GetSelection(beg, end);
    IF (beg = 7) & (end = 19) THEN
        INC(result, 1000)
    END;

    (* Empty selection (beg = end) is legal and round-trips. *)
    c.SetSelection(3, 3);
    c.GetSelection(beg, end);
    IF (beg = 3) & (end = 3) THEN
        INC(result, 10000)
    END;

    RETURN result  (* expect 11111 if every check passes *)
END Run;

END TextControllersStdCtrl.
