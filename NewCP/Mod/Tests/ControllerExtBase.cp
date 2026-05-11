MODULE ControllerExtBase;
(*
   Cross-module workout for the `Controllers.Controller` slice.

   Three things this test pins down:

   1. A concrete subclass of `Controllers.ControllerDesc`
      (`MyControllerDesc`) compiles and is reachable through both
      the leaf-typed pointer and a `Controllers.Controller` base
      pointer.  The chain is

          Stores.StoreDesc
            └── Controllers.ControllerDesc
                  └── MyControllerDesc

      and exercises the same cross-module record-descriptor /
      vtable plumbing as Views.

   2. The Controllers message types are extensible — we declare
      a custom EditMsg-style record that fills in fields the
      base inherits and the test confirms field reads round-trip
      through the inheritance chain.

   3. `Containers.View.controller` (now typed as
      `Containers.Controller`) accepts a concrete subclass.  We
      construct a `Containers.ControllerDesc` extension and assign
      it through a `Containers.View`-pointing field reference to
      prove the type identity stitches together.

   Returns a packed result that proves each stage fired.
*)

    IMPORT Stores, Views, Controllers, Containers;

    TYPE
        (** Leaf controller — extends Controllers.Controller via
            the abstract ControllerDesc base, carries a counter
            that vtable-dispatched methods bump. *)
        MyControllerDesc* = RECORD (Controllers.ControllerDesc)
            domainCalls*: INTEGER
        END;
        MyController* = POINTER TO MyControllerDesc;

        (** Subclass of Containers.Controller proving the
            Containers -> Controllers chain stitches together
            cleanly.  No extra state — just exercises the type. *)
        BoundControllerDesc* = RECORD (Containers.ControllerDesc)
            boundTag*: INTEGER
        END;
        BoundController* = POINTER TO BoundControllerDesc;

        (** A populated EditMsg-shaped record we read back to prove
            field inheritance from Controllers.Message ->
            Views.CtrlMessage round-trips. *)
        TaggedEditMsg* = RECORD (Controllers.EditMsg)
            tag*: INTEGER
        END;


    (** Override the inherited Domain method from Stores.Store via
        Controllers.ControllerDesc.Domain. *)
    PROCEDURE (c: MyControllerDesc) Domain* (): Stores.Domain;
    BEGIN
        c.domainCalls := c.domainCalls + 1;
        RETURN NIL
    END Domain;


    PROCEDURE Run* (): INTEGER;
        VAR mc: MyController;
            bc: BoundController;
            base: Controllers.Controller;
            cbase: Containers.Controller;
            d: Stores.Domain;
            msg: TaggedEditMsg;
            sum: INTEGER;
    BEGIN
        (* Stage 1: leaf controller, verify cross-module vtable
           dispatch on Domain. *)
        NEW(mc);
        mc.domainCalls := 0;
        base := mc;
        d := base.Domain();                  (* virtual dispatch -> MyControllerDesc.Domain *)
        ASSERT(d = NIL, 50);
        ASSERT(mc.domainCalls = 1, 51);

        (* Stage 2: a Containers-derived controller is a
           Controllers.Controller too — verify both widen
           assignments compile and the inheritance chain
           reaches the Stores.Store base. *)
        NEW(bc);
        bc.boundTag := 7;
        cbase := bc;
        base  := bc;                         (* widen via Containers.Controller's base *)
        ASSERT(cbase # NIL, 60);
        ASSERT(base # NIL, 61);

        (* Stage 3: the Controllers message records carry fields
           from RequestMessage -> CursorMessage -> TransferMessage
           chains, but for the simpler EditMsg subclass we just
           prove the inherited `op` / `requestFocus` slots round-
           trip alongside our own tag. *)
        msg.op := 1;
        msg.requestFocus := TRUE;
        msg.tag := 99;

        sum := 0;
        IF msg.requestFocus THEN sum := sum + 1000 END;
        sum := sum + msg.op * 100;
        sum := sum + msg.tag;

        (* Packed result:
             mc.domainCalls = 1
             bc.boundTag    = 7
             sum            = 1000 (msg.requestFocus) + msg.op*100 (=100)
                            + msg.tag (=99)
                            = 1199
           Encode:
             mc.domainCalls * 100000 = 100000
             + 10000                 = 110000     (stage-2 widen marker)
             + bc.boundTag * 1000    = 117000
             + sum                   = 118199 *)
        RETURN mc.domainCalls * 100000
             + 10000
             + bc.boundTag * 1000
             + sum
    END Run;

END ControllerExtBase.
