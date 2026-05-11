MODULE ViewExtSuper;
(*
   Three-level vtable workout that crosses two modules:

       Stores.StoreDesc                 (Stores)
         └── Views.ViewDesc             (Views)
               └── TaggedViewDesc       (ViewExtSuper)

   The Internalize chain super-calls all the way up. We synthesize
   a fake Stores.Reader (handle = 0, eof = TRUE) so the chain runs
   to completion without trying to consume real bytes; we then
   verify that each layer's Internalize body executed by counting
   side-effects on the receiver. The base layer (Views.ViewDesc.
   Internalize) sets nothing observable, but the chain still
   resolves; the subclass writes its own counter.

   Run returns the count of subclass-level Internalize invocations
   (we drive it twice through `cv.Internalize(rd)` and `v.Internalize(rd)`
   so virtual + static dispatch both increment).
*)

    IMPORT Views, Stores;

    TYPE
        TaggedViewDesc* = RECORD (Views.ViewDesc)
            internalizeRuns*: INTEGER
        END;
        TaggedView* = POINTER TO TaggedViewDesc;


    (** Concrete (non-EXTENSIBLE) override that still super-calls
        through Views.ViewDesc up to Stores.StoreDesc.Internalize.
        TaggedViewDesc itself is a leaf RECORD so the override
        cannot be EXTENSIBLE — sema correctly rejects that
        combination. The super-call works irrespective of leaf
        extensibility. *)
    PROCEDURE (v: TaggedViewDesc) Internalize* (VAR rd: Stores.Reader);
    BEGIN
        v.Internalize^(rd);                 (* super-call into Views.ViewDesc.Internalize *)
        v.internalizeRuns := v.internalizeRuns + 1
    END Internalize;

    (** Required ABSTRACT override on Views.ViewDesc. EMPTY for us. *)
    PROCEDURE (v: TaggedViewDesc) Restore* (f: Views.Frame; l, t, r, b: INTEGER);
    BEGIN
    END Restore;


    PROCEDURE Run* (): INTEGER;
        VAR cv: TaggedView; v: Views.View; rd: Stores.Reader;
    BEGIN
        NEW(cv);
        cv.internalizeRuns := 0;

        (* Reader with handle 0 / eof = TRUE — the runtime's
           StoresSys treats handle 0 as a dead reader, every
           subsequent Read* is a no-op. *)
        rd.handle := 0;
        rd.eof := TRUE;

        cv.Internalize(rd);                  (* static dispatch on the subclass *)

        v := cv;
        v.Internalize(rd);                   (* virtual dispatch through Views.View *)

        RETURN cv.internalizeRuns            (* expect 2 *)
    END Run;

END ViewExtSuper.
