MODULE TextControllersSmoke;
(* First smoke test for the TextControllers slice.

   Verifies the abstract Controller/Directory surface materializes
   correctly when imported from another module and that the public
   CONSTs / message records compile and link. *)

IMPORT TextControllers;

PROCEDURE Run* (): INTEGER;
    VAR cm: TextControllers.SetCaretMsg;
        sm: TextControllers.SetSelectionMsg;
        result: INTEGER;
BEGIN
    cm.pos := 42;
    sm.beg := 1; sm.end := 5;
    (* Encode all four scalar values into one integer so the test
       proves field assignment and field read both round-trip. *)
    result := cm.pos * 10000 + sm.beg * 100 + sm.end;
    (* Also exercise the `none` constant — value should be -1
       per BB. *)
    IF TextControllers.none = -1 THEN
        INC(result)  (* +1 to mark the constant check passed. *)
    END;
    RETURN result
END Run;

END TextControllersSmoke.
