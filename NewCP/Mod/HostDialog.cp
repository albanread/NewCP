MODULE HostDialog;
(*
   Host file-chooser dialogs — wraps `HostDialogSys` to give
   callers a BB-faithful Boolean result and a CHAR path string.

   Pattern matches the HostXxxSys layer:
     HostDialog.cp     — CP-friendly surface (this module)
     HostDialogSys.cp  — DEFINITION module; backed by Rust Win32 shim

   Usage example (in StdCmds or a tool module):

       VAR path: ARRAY 1024 OF CHAR;
       IF HostDialog.GetOpenFileName("", path) THEN
           (* path contains the chosen file path *)
       END;
*)

    IMPORT HostDialogSys;

    CONST
        (** Maximum path length, matching Win32's MAX_PATH convention. *)
        MaxPath = 1024;

        (** A catch-all filter string (All Files).  Each field is
            separated by a literal NUL character (not representable as
            a string literal, so we rely on the array initialiser — the
            host_dialog_sys shim treats an empty filter as "All Files"). *)
        defaultFilter = "";


    (** Show an Open-File dialog.
        `filter`  — optional NUL-separated filter pairs; use "" for
                    "All Files (*.*)" only.
        `path`    — receives the chosen path on return.
        Returns TRUE iff the user chose a file (not cancelled). *)
    PROCEDURE GetOpenFileName* (IN filter: ARRAY OF SHORTCHAR;
                                OUT path: ARRAY OF CHAR): BOOLEAN;
        VAR ok: INTSHORT;
    BEGIN
        ok := HostDialogSys.ShowOpenFile(filter, path);
        RETURN ok = 1
    END GetOpenFileName;


    (** Show a Save-File dialog pre-populated with `initialName`.
        `filter`      — same format as GetOpenFileName.
        `initialName` — suggested filename / path (may be empty).
        `path`        — receives the confirmed path on return.
        Returns TRUE iff the user confirmed a save path. *)
    PROCEDURE GetSaveFileName* (IN filter: ARRAY OF SHORTCHAR;
                                IN initialName: ARRAY OF CHAR;
                                OUT path: ARRAY OF CHAR): BOOLEAN;
        VAR ok: INTSHORT;
    BEGIN
        ok := HostDialogSys.ShowSaveFile(filter, initialName, path);
        RETURN ok = 1
    END GetSaveFileName;


END HostDialog.
