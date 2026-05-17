DEFINITION MODULE HostClipboardSys;
(*
   System-level clipboard access — DEFINITION only.

   Backed by host_clipboard_sys.rs (Win32 OpenClipboard /
   GetClipboardData / SetClipboardData).

   Both procedures work with ARRAY OF CHAR (UTF-32 wide strings).
   The Rust implementation converts between UTF-32 and the platform
   clipboard format (CF_UNICODETEXT / UTF-16 on Windows).

   Return value convention: 1 = success, 0 = failure.
*)

PROCEDURE GetText* (OUT text: ARRAY OF CHAR): INTSHORT;
(** Copy the clipboard's current text content (if any) into `text`.
    Returns 1 on success, 0 if the clipboard is empty, has no text,
    or an OS error occurs.  `text` is NUL-terminated on success. *)

PROCEDURE SetText* (IN text: ARRAY OF CHAR): INTSHORT;
(** Replace the clipboard's text content with `text`.
    Returns 1 on success, 0 on OS error. *)

END HostClipboardSys.
