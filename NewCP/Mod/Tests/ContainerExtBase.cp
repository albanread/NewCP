MODULE ContainerExtBase;
(*
   3-level cross-module vtable workout via Containers.

   Type chains exercised:

       Stores.StoreDesc                        (Stores)
         └── Models.ModelDesc                  (Models)
               └── Containers.ModelDesc        (Containers)
                     └── MyModelDesc           (this module)

       Stores.StoreDesc                        (Stores)
         └── Views.ViewDesc                    (Views)
               └── Containers.ViewDesc         (Containers)
                     └── MyViewDesc            (this module)

   Three cross-module vtable scenarios:

   1. `Containers.View.InitModel` (NEW, declared in Containers)
      internally calls `v.AcceptableModel(m)` via virtual dispatch.
      Our concrete subclass overrides `AcceptableModel` (ABSTRACT in
      Containers) and the call must land in the subclass body.

   2. `Models.Model.Internalize` is EXTENSIBLE; Containers.Model
      overrides EXTENSIBLE; we override EXTENSIBLE again.  Three
      levels of super-call through three different modules.

   3. Run drives method dispatch through both the leaf-typed
      pointer (static-bound) and a widened `Views.View` base
      pointer (purely vtable-driven).
*)

    IMPORT Stores, Models, Views, Containers;

    TYPE
        (** Concrete leaf model — extends Containers.Model with a
            single counter that each super-call bump observes. *)
        MyModelDesc* = RECORD (Containers.ModelDesc)
            internalizeRuns*: INTEGER
        END;
        MyModel* = POINTER TO MyModelDesc;

        (** Concrete leaf view — extends Containers.View with a
            counter and a "was AcceptableModel called?" flag. *)
        MyViewDesc* = RECORD (Containers.ViewDesc)
            internalizeRuns*: INTEGER;
            acceptableCalls*: INTEGER
        END;
        MyView* = POINTER TO MyViewDesc;


    (* -- Model ABSTRACT overrides ---------------------------------------- *)

    PROCEDURE (m: MyModelDesc) GetEmbeddingLimits* (OUT minW, maxW, minH, maxH: INTEGER);
    BEGIN
        minW := 1;
        maxW := 2;
        minH := 3;
        maxH := 4
    END GetEmbeddingLimits;

    PROCEDURE (m: MyModelDesc) ReplaceView* (old, new: Views.View);
    BEGIN
        (* EMPTY — no embedded views in this test fixture. *)
    END ReplaceView;

    (** Three-level super-call: this -> Containers.ModelDesc.Internalize
        -> Models.ModelDesc.Internalize -> Stores.StoreDesc.Internalize. *)
    PROCEDURE (m: MyModelDesc) Internalize* (VAR rd: Stores.Reader);
    BEGIN
        m.Internalize^(rd);
        m.internalizeRuns := m.internalizeRuns + 1
    END Internalize;


    (* -- View ABSTRACT overrides ----------------------------------------- *)

    (** Containers.View.InitModel calls this through the vtable to
        validate the model before binding.  Records that the call
        happened. *)
    PROCEDURE (v: MyViewDesc) AcceptableModel* (m: Containers.Model): BOOLEAN;
    BEGIN
        v.acceptableCalls := v.acceptableCalls + 1;
        RETURN m # NIL
    END AcceptableModel;

    PROCEDURE (v: MyViewDesc) Restore* (f: Views.Frame; l, t, r, b: INTEGER);
    BEGIN
        (* EMPTY — no rendering in the test fixture. *)
    END Restore;

    (** Three-level super-call mirror of MyModelDesc.Internalize. *)
    PROCEDURE (v: MyViewDesc) Internalize* (VAR rd: Stores.Reader);
    BEGIN
        v.Internalize^(rd);
        v.internalizeRuns := v.internalizeRuns + 1
    END Internalize;


    (* -- Driver ---------------------------------------------------------- *)

    PROCEDURE Run* (): INTEGER;
        VAR v: MyView; m: MyModel;
            vbase: Views.View;
            rd: Stores.Reader;
            minW, maxW, minH, maxH: INTEGER;
    BEGIN
        NEW(v);
        NEW(m);
        v.internalizeRuns := 0;
        v.acceptableCalls := 0;
        m.internalizeRuns := 0;

        (* Stage 1: Containers.View.InitModel -> virtual dispatch
           into MyViewDesc.AcceptableModel.  Cross-module vtable
           lookup. *)
        v.InitModel(m);                       (* InitModel calls v.AcceptableModel via vtable *)

        (* Stage 2: three-level super-call through Models / Containers.
           Driven via the leaf-typed pointer. *)
        rd.handle := 0; rd.eof := TRUE;
        m.Internalize(rd);

        (* Stage 3: same chain on the View side, but through the
           widened base pointer so dispatch is purely virtual. *)
        vbase := v;
        vbase.Internalize(rd);

        (* Stage 4: pull GetEmbeddingLimits through the Model ABSTRACT
           override. *)
        m.GetEmbeddingLimits(minW, maxW, minH, maxH);

        (* Packed result:
             m.internalizeRuns      = 1  (Stage 2)
             v.internalizeRuns      = 1  (Stage 3)
             v.acceptableCalls      = 1  (Stage 1)
             minW=1, maxW=2, minH=3, maxH=4
           Encode as
             v.internalizeRuns * 100000 +
             m.internalizeRuns * 10000 +
             v.acceptableCalls * 1000 +
             minW * 100 + maxW * 10 + maxH
             = 100000 + 10000 + 1000 + 100 + 20 + 4
             = 111124 *)
        RETURN v.internalizeRuns * 100000
             + m.internalizeRuns * 10000
             + v.acceptableCalls * 1000
             + minW * 100
             + maxW * 10
             + maxH
    END Run;

END ContainerExtBase.
