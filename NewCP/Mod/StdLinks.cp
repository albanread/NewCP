MODULE StdLinks;
(*
   First slice of the BlackBox `StdLinks` port.

   BB's StdLinks owns the hyperlink + target machinery inside
   `.odc` documents — the welcome page's "click this link" UI
   plumbs through `StdLinks.ShowTarget`.  The full module is
   ~1700 lines including a Stamps-style attribute writer and
   the target-name registry.

   This slice ships only the public-procedure surface; bodies
   are no-ops.  The welcome page's link targets won't navigate
   until the body lands.
*)


    PROCEDURE ShowTarget* (IN ident: ARRAY OF CHAR);
    BEGIN
    END ShowTarget;

    PROCEDURE CreateLink*;
    BEGIN
    END CreateLink;

    PROCEDURE CreateTarget*;
    BEGIN
    END CreateTarget;

END StdLinks.
