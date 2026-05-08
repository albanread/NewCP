DEFINITION MODULE WinPayload;

(* Interface only. The actual implementations are Rust-hosted exports
   registered by winpayload_module_artifact() in newcp-runtime. *)

PROCEDURE GetStr*(payload, key: ARRAY OF SHORTCHAR;
                  VAR out: ARRAY OF SHORTCHAR): INTSHORT;

PROCEDURE GetInt*(payload, key: ARRAY OF SHORTCHAR;
                  VAR out: INTEGER): INTSHORT;

PROCEDURE GetBool*(payload, key: ARRAY OF SHORTCHAR;
                   VAR out: INTSHORT): INTSHORT;

END WinPayload.
