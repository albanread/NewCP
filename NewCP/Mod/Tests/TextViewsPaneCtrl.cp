MODULE TextViewsPaneCtrl;
(* End-to-end integration of the concrete editor pane (Pane) with
   the concrete controller (StdCtrl) via the abstract Views.View /
   TextControllers.Controller dispatch chain.

   Exercises:
     - TextViews.dir.New(NIL) — allocates a Pane via PaneDirectory
     - the Pane's BB-faithful display-state methods round-trip
       (DisplayMarks, SetOrigin, SetDefaults via abstract View)
     - dir.New() on TextControllers — allocates a StdCtrl
     - InitView2 binds the StdCtrl to the Pane via the abstract
       Views.View parameter

   This is the first slice where a controller and its view are
   both concrete and bind to each other through the framework's
   abstract dispatch.  Encoded result = number of checks passed
   (max 6). *)

IMPORT TextViews, TextControllers;

PROCEDURE Run* (): INTEGER;
    VAR v: TextViews.View;
        p: TextViews.Pane;
        c: TextControllers.Controller;
        org, dy: INTEGER;
        result: INTEGER;
BEGIN
    result := 0;

    (* 1. Allocate a Pane via the abstract Directory. *)
    v := TextViews.dir.New(NIL);
    IF v # NIL THEN INC(result, 1) END;

    (* 2. Type-guard to the concrete Pane and verify defaults. *)
    p := v(TextViews.Pane);
    IF (p.org = 0) & (p.dy = 0) & ~p.hideMarks THEN INC(result, 2) END;

    (* 3. Display-marks round-trip through abstract View dispatch. *)
    v.DisplayMarks(TRUE);
    IF v.HidesMarks() & p.hideMarks THEN INC(result, 4) END;
    v.DisplayMarks(FALSE);
    IF ~v.HidesMarks() & ~p.hideMarks THEN INC(result, 8) END;

    (* 4. SetOrigin / PollOrigin round-trip. *)
    v.SetOrigin(100, 7);
    v.PollOrigin(org, dy);
    IF (org = 100) & (dy = 7) & (p.org = 100) & (p.dy = 7) THEN
        INC(result, 16)
    END;

    (* 5. Bind a fresh StdCtrl to this Pane via InitView2.  The
       controller's view- field should be set to our Pane. *)
    c := TextControllers.dir.New();
    c.InitView2(v);
    IF c.ThisView() = v THEN INC(result, 32) END;

    RETURN result  (* expect 63 = 1 + 2 + 4 + 8 + 16 + 32 *)
END Run;

END TextViewsPaneCtrl.
