MODULE BrkMvcProbe;
(* Uses BRK to inspect the MVC framework's runtime state at three
   key wiring points:

     1. After allocating a fresh Pane via TextViews.dir.New(NIL).
     2. After allocating a StdCtrl via TextControllers.dir.New().
     3. After InitView2 binds the StdCtrl to the Pane.

   At each point BRK(p) dumps the heap block's TypeDesc + payload
   bytes — letting us read off fields by inspecting the hex grid.
   The dump goes to stderr (`cargo test -- --nocapture` shows it).

   Return value 1 just asserts the procedure completed normally;
   the dump is the real product. *)

IMPORT TextViews, TextControllers;

PROCEDURE Run* (): INTEGER;
    VAR v: TextViews.View;
        p: TextViews.Pane;
        c: TextControllers.Controller;
BEGIN
    (* Snapshot 1: fresh Pane, no model bound, no controller. *)
    v := TextViews.dir.New(NIL);
    p := v(TextViews.Pane);
    BRK(p);

    (* Snapshot 2: fresh StdCtrl, no view bound. *)
    c := TextControllers.dir.New();
    BRK(c);

    (* Snapshot 3: after InitView2 binds controller to pane.  Both
       sides should now reference each other — visible in the field
       dump. *)
    c.InitView2(v);
    BRK(c);
    BRK(p);

    RETURN 1
END Run;

END BrkMvcProbe.
