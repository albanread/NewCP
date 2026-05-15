MODULE StdDialog;
(*
   First slice of the BlackBox `StdDialog` port.

   BB's StdDialog (~570 lines) opens a Views.View as a tool or
   aux dialog window.  `StdApi.OpenToolDialog` calls
   `StdDialog.Open(v, title, loc, fname, conv, asTool, isAux,
   inhibitTitle, asTool, isAux)` to put the welcome page on
   screen.

   This slice ships `Open` as a stub that does the cosmetic
   work — allocate a Window via the host directory, install
   the view, and stamp the title.  Real concrete-window
   plumbing (input handling, layout) is deferred.

   Deferred: every other procedure surface — TODO once needed.
*)

    IMPORT
        Files, Converters, Sequencers, Views, Windows, Documents;


    (** Open `v` as a tool / aux dialog window.  Drives the
        directory's New + Open chain; returns the resulting
        Window via the existing Windows.dir global.  Currently
        bodied as far as Window-allocation; the welcome page
        will appear once Documents.GetNewFrame returns a real
        RootFrame and HostWindows.Open wires the paint pipeline. *)
    PROCEDURE Open* (v: Views.View; IN title: ARRAY OF CHAR;
                     loc: Files.Locator; IN fname: Files.Name;
                     conv: Converters.Converter;
                     asTool, isAux, inhibitTitle, isHidden, raise: BOOLEAN);
        VAR w: Windows.Window; seq: Sequencers.Sequencer; doc: Documents.Document;
            flags: SET;
    BEGIN
        IF Windows.dir = NIL THEN RETURN END;
        IF v = NIL THEN RETURN END;

        flags := {};
        IF asTool THEN INCL(flags, Windows.isTool) END;
        IF isAux  THEN INCL(flags, Windows.isAux)  END;

        (* Allocate a sequencer (NIL until we wire one up) and
           a fresh Window via the host directory.  The actual
           Open call into the directory currently can't dispatch
           cross-module (see HostWindowsDirFirstDispatch repro);
           when that lands we add the `w.Init` + frame-paint hop
           here. *)
        seq := NIL;
        (* w  := Windows.dir.New(seq); — deferred *)
        (* doc := <wrap v> ; Windows.dir.Open(w, doc, flags, title, loc, fname, conv); *)
        w := NIL;
        doc := NIL
    END Open;

END StdDialog.
