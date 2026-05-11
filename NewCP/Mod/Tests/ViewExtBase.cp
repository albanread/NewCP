MODULE ViewExtBase;
(*
   Cross-module workout for `Views.View` extension.

   This module defines a concrete subclass of `Views.ViewDesc` and
   exercises three vtable-sensitive paths:

   - super-call up the Internalize chain (subclass → Views.View →
     Stores.Store);
   - override of an ABSTRACT method on the base (Restore);
   - override of an EXTENSIBLE method on the base (ThisModel)
     that the subclass wraps with its own logic.

   The `Run` procedure constructs a CountingView, drives a no-op
   Restore via the View-typed pointer (forcing vtable dispatch),
   and returns a packed result. Used by integration tests to
   confirm cross-module record-descriptor vtables resolve.
*)

    IMPORT Views, Models, Stores;

    CONST
        RestoreTag* = 7;
        ThisModelTag* = 13;

    TYPE
        CountingViewDesc* = RECORD (Views.ViewDesc)
            paintCount*: INTEGER;
            lastL*, lastT*, lastR*, lastB*: INTEGER
        END;
        CountingView* = POINTER TO CountingViewDesc;


    (** Concrete override of the ABSTRACT Restore method.
        Records the paint rectangle and bumps the counter. *)
    PROCEDURE (v: CountingViewDesc) Restore* (f: Views.Frame; l, t, r, b: INTEGER);
    BEGIN
        v.paintCount := v.paintCount + RestoreTag;
        v.lastL := l;
        v.lastT := t;
        v.lastR := r;
        v.lastB := b
    END Restore;

    (** Override of the EXTENSIBLE ThisModel method.
        Default returns NIL; we wrap that. *)
    PROCEDURE (v: CountingViewDesc) ThisModel* (): Models.Model;
        VAR base: Models.Model;
    BEGIN
        base := v.ThisModel^();           (* super-call into Views.ViewDesc.ThisModel *)
        ASSERT(base = NIL, 30);            (* Views default is NIL *)
        v.paintCount := v.paintCount + ThisModelTag;
        RETURN NIL
    END ThisModel;


    (** Drive both methods through the View base-typed pointer
        so dispatch goes through the vtable, not the static
        bound type. *)
    PROCEDURE Run* (): INTEGER;
        VAR cv: CountingView; v: Views.View; m: Models.Model;
    BEGIN
        NEW(cv);
        cv.paintCount := 0;
        v := cv;                              (* widen to base pointer *)
        v.Restore(NIL, 1, 2, 3, 4);           (* virtual dispatch -> CountingView.Restore *)
        m := v.ThisModel();                    (* virtual dispatch -> CountingView.ThisModel *)
        ASSERT(m = NIL, 40);
        (* Packed result encodes which methods fired AND the
           Restore rectangle was recorded.  Expected:
             paintCount = 7 + 13 = 20
             paintCount * 1000 + lastL*1000 + lastT*100 + lastR*10 + lastB
             = 20000 + 1000 + 200 + 30 + 4
             = 21234 *)
        RETURN cv.paintCount * 1000
              + cv.lastL * 1000
              + cv.lastT * 100
              + cv.lastR * 10
              + cv.lastB
    END Run;

END ViewExtBase.
