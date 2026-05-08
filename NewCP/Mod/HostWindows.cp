DEFINITION MODULE HostWindows;

(* Interface only. The actual implementations are Rust-hosted exports
   registered by native_module_artifact() in newcp-runtime. *)

PROCEDURE PublishUi*(json: ARRAY OF SHORTCHAR);

PROCEDURE RequestClose*;

PROCEDURE RequestPresent*;

PROCEDURE WaitNamedEvent*(VAR name: ARRAY OF SHORTCHAR;
                          VAR payload: ARRAY OF SHORTCHAR;
                          timeoutMs: INTEGER): INTSHORT;

(* Open an MDI child window under `parentWindowId`, publishing `specJson`
   into the child's embedded native UI session. The new child's WindowId
   is returned via `childWindowId`. Returns 1 on success, 0 on failure.
   `title` may be empty (defaults to "Document"); `specJson` may be empty
   (creates a bare child the caller can publish into later). *)
PROCEDURE OpenChildWindow*(parentWindowId: INTEGER;
                           title: ARRAY OF SHORTCHAR;
                           specJson: ARRAY OF SHORTCHAR;
                           VAR childWindowId: INTEGER): INTSHORT;

(* Close an MDI child by its WindowId (cascades through the same path as
   the X-button). Returns 1 on success, 0 on failure. *)
PROCEDURE CloseChildWindow*(childWindowId: INTEGER): INTSHORT;

END HostWindows.
