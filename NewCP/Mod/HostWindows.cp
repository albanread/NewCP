DEFINITION MODULE HostWindows;

(* Interface only. The actual implementations are Rust-hosted exports
   registered by native_module_artifact() in newcp-runtime. *)

PROCEDURE PublishUi*(json: ARRAY OF SHORTCHAR);

PROCEDURE RequestClose*;

PROCEDURE RequestPresent*;

PROCEDURE WaitNamedEvent*(VAR name: ARRAY OF SHORTCHAR;
                          VAR payload: ARRAY OF SHORTCHAR;
                          timeoutMs: INTEGER): INTSHORT;

END HostWindows.
