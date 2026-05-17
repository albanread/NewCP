DEFINITION MODULE HostDialogSys;
(**
   Flat C-ABI file-dialog facade backed by the Rust runtime
   (src/newcp-runtime/src/host_dialog_sys.rs).

   Both procedures show a native Win32 file-chooser dialog and
   return the user-selected path in `path` (UTF-32, null-terminated).

   Return value:
     1  — a file was chosen; `path` is set
     0  — the user cancelled or no file system is available;
          `path[0]` is set to 0X (empty string)

   filter format (matches `OPENFILENAMEW.lpstrFilter`):
     SHORTCHAR pairs separated by NUL, terminated by double-NUL.
     Example: "Text Files^0*.txt^0All Files^0*.*^0^0"
     (where ^0 represents the 0X character).
     Use an empty string for "All Files *.*".
*)

(** Show a file-open dialog.
    `filter` — NUL-separated pairs (description + pattern) + double-NUL.
    `path`   — receives the chosen path; 0X on cancel. *)
PROCEDURE ShowOpenFile* (IN filter: ARRAY OF SHORTCHAR;
                         OUT path: ARRAY OF CHAR): INTSHORT;

(** Show a file-save dialog pre-populated with `initialName`.
    `filter`      — same format as ShowOpenFile.
    `initialName` — suggested filename / path (may be empty).
    `path`        — receives the confirmed path; 0X on cancel. *)
PROCEDURE ShowSaveFile* (IN filter: ARRAY OF SHORTCHAR;
                         IN initialName: ARRAY OF CHAR;
                         OUT path: ARRAY OF CHAR): INTSHORT;

END HostDialogSys.
