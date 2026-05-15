MODULE StdScrollers;
(*
   First slice of the BlackBox `StdScrollers` port.

   BB's StdScrollers wraps a child view in a scrollable
   container with a scroll-bar pair.  ~1700 lines.

   This slice ships type surface only — scrolling is not on
   the welcome-page critical path (the About page fits in the
   default window size).
*)

    IMPORT Views;


    TYPE
        (** Abstract scroller view.  Concrete subclass lives
            in the host-side renderer; surface here is just so
            other modules can reference the type. *)
        ViewDesc* = ABSTRACT RECORD (Views.ViewDesc) END;
        View*     = POINTER TO ViewDesc;

END StdScrollers.
