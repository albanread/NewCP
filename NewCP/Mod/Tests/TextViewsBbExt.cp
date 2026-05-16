MODULE TextViewsBbExt;
(*
   BB-faithful TextViews-style slice as a 4-level vtable +
   wire-format workout.

   Type chain:

       Stores.StoreDesc
         └── Views.ViewDesc                (Views)
               └── Containers.ViewDesc     (Containers — adds `model`/`controller`/`alienCtrl`)
                     └── BbViewDesc        (this module — leaf "StdView" stand-in)

   What this exercises that the previous extension tests didn't:

   - Round-trips through `Stores.CopyOf` — that's
     `Externalize → in-memory buffer → Internalize` going through
     EVERY layer's super-call chain.  Proves cross-module method
     dispatch survives a real serialization cycle.

   - The leaf's `Internalize2` / `Externalize2` are called by
     `Containers.View.Internalize` / `.Externalize` — the
     standard BlackBox subclass-extension hook for a view that
     wants to add its own body fields without re-implementing
     the version-stamp dance.

   - Concrete overrides of the ABSTRACT View / Container methods
     (`Restore` on Views, `AcceptableModel` on Containers).

   Returns a packed value confirming each field round-tripped.
*)

    IMPORT Stores, Views, Containers, Models;

    TYPE
        (** Leaf View — the structural analogue of
            `TextViews.StdViewDesc`.  Carries the same trio of
            view-body fields (hideMarks / org / dy) BB's StdView
            does.  No model wiring here yet — the embedded model
            is logically `v.model` inherited from Containers.View
            but staying NIL in this fixture. *)
        BbViewDesc* = RECORD (Containers.ViewDesc)
            hideMarks*: BOOLEAN;
            org*, dy*:  INTEGER
        END;
        BbView* = POINTER TO BbViewDesc;


    (* -- ABSTRACT overrides --------------------------------------------- *)

    PROCEDURE (v: BbViewDesc) AcceptableModel* (m: Containers.Model): BOOLEAN;
    BEGIN
        RETURN m # NIL                         (* any non-NIL model passes *)
    END AcceptableModel;

    PROCEDURE (v: BbViewDesc) Restore* (f: Views.Frame; l, t, r, b: INTEGER);
    BEGIN
        (* EMPTY — no rendering in this fixture. *)
    END Restore;


    (* -- Body Externalize / Internalize hooks --------------------------- *)

    PROCEDURE (v: BbViewDesc) Externalize2* (VAR wr: Stores.Writer);
    BEGIN
        wr.WriteBool(v.hideMarks);
        wr.WriteLong(v.org);
        wr.WriteLong(v.dy)
    END Externalize2;

    PROCEDURE (v: BbViewDesc) Internalize2* (VAR rd: Stores.Reader);
    BEGIN
        rd.ReadBool(v.hideMarks);
        rd.ReadLong(v.org);
        rd.ReadLong(v.dy)
    END Internalize2;


    (* -- Driver ---------------------------------------------------------- *)

    PROCEDURE Run* (): INTEGER;
        VAR orig, copy: BbView;
            cloned: Stores.Store;
            asView: Views.View;
            asCView: Containers.View;
            packed: INTEGER;
    BEGIN
        NEW(orig);
        orig.hideMarks := TRUE;
        orig.org := 42;
        orig.dy  := 17;

        (* Round-trip via Stores.CopyOf — runs Externalize through
           every super-call layer and Internalize back through
           every layer.  Containers.View.Internalize calls our
           Internalize2; Containers.View.Externalize calls our
           Externalize2.  Cross-module chain in motion. *)
        cloned := Stores.CopyOf(orig);
        IF cloned = NIL THEN RETURN -1 END;
        copy := cloned(BbView);

        (* Distinct heap object. *)
        IF copy = orig THEN RETURN -2 END;

        (* Body fields survived the round-trip. *)
        IF copy.hideMarks # TRUE THEN RETURN -3 END;
        IF copy.org # 42 THEN RETURN -4 END;
        IF copy.dy  # 17 THEN RETURN -5 END;

        (* Widening to every base pointer works — proves the
           4-level type chain stitches together at runtime. *)
        asView  := copy;
        asCView := copy;
        IF asView = NIL  THEN RETURN -6 END;
        IF asCView = NIL THEN RETURN -7 END;

        (* Pack the result.  An aliased copy or a desynced wire
           format would have tripped one of the assertions above.
              hideMarks * 1000000  (1*1000000)
              + org * 1000         (42*1000)
              + dy                 (17)
              = 1042017 *)
        packed := 0;
        IF copy.hideMarks THEN packed := packed + 1000000 END;
        packed := packed + copy.org * 1000;
        packed := packed + copy.dy;
        RETURN packed
    END Run;

END TextViewsBbExt.
