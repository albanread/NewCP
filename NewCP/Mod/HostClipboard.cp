MODULE HostClipboard;
(*
   First slice of the BlackBox `HostClipboard` port.

   BB's HostClipboard is a small (~230-line) Win32 wrapper over
   the OS clipboard — `OpenClipboard`, `SetClipboardData`,
   `GetClipboardData`.  Used by Std/Cmds' Copy / Cut / Paste
   commands and by OLE data interchange.

   This slice ships the public surface as no-op stubs.
   Welcome-page open doesn't touch the clipboard, so the framework
   compiles and runs without any clipboard backend.

   Deferred: every body — port alongside the OS clipboard plumbing.
*)

    IMPORT Stores, Views;

    (** Place `v` on the clipboard, advertised at (w, h) DIP. *)
    PROCEDURE Register* (v: Views.View; w, h: INTEGER; isSingle: BOOLEAN);
    BEGIN
    END Register;

    (** Pull the current clipboard view (or NIL if empty). *)
    PROCEDURE GetClipView* (OUT v: Views.View; OUT w, h: INTEGER);
    BEGIN
        v := NIL;
        w := 0; h := 0
    END GetClipView;

    (** Test whether the clipboard payload can be converted to
        the requested store type. *)
    PROCEDURE ConvertibleTo* (type: Stores.TypeName): BOOLEAN;
    BEGIN
        RETURN FALSE
    END ConvertibleTo;

    (** Drop the clipboard contents. *)
    PROCEDURE Flush*;
    BEGIN
    END Flush;

    (** Diagnostic dump.  Stub. *)
    PROCEDURE Dump*;
    BEGIN
    END Dump;

END HostClipboard.
