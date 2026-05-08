DEFINITION MODULE WinSpec;

(* Interface only. The actual implementations are Rust-hosted exports
   registered by winspec_module_artifact() in newcp-runtime. *)

PROCEDURE Begin*(title: ARRAY OF SHORTCHAR);

PROCEDURE OpenStack*(gap: INTSHORT);

PROCEDURE OpenRow*(gap: INTSHORT);

PROCEDURE CloseContainer*;

PROCEDURE AddButton*(id, label, event: ARRAY OF SHORTCHAR);

PROCEDURE AddText*(text: ARRAY OF SHORTCHAR);

PROCEDURE AddTextarea*(id, label, value: ARRAY OF SHORTCHAR; readonly: INTSHORT);

PROCEDURE AddTextGrid*(id, event: ARRAY OF SHORTCHAR; cols, rows: INTSHORT);

PROCEDURE AddSurface*(id, event: ARRAY OF SHORTCHAR; width, height: INTSHORT);

PROCEDURE AddRgbaPane*(id, event: ARRAY OF SHORTCHAR; width, height: INTSHORT);

PROCEDURE GetSpec*(VAR buf: ARRAY OF SHORTCHAR): INTSHORT;

END WinSpec.
