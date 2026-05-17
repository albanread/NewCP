MODULE HostClipboard;
(*
   High-level clipboard operations for wide (CHAR) text.

   Wraps HostClipboardSys with a clean Boolean API so callers
   never have to deal with the INTSHORT return convention.

   The clipboard text is assumed to be short enough for the callers'
   stack buffers (ClipCapacity = 65536 CHARs).  A future slice can
   heap-allocate for very large pastes.
*)

    IMPORT HostClipboardSys;

    CONST
        ClipCapacity* = 65536;

    (** Copy `text` to the system clipboard.
        Returns TRUE on success, FALSE on error. *)
    PROCEDURE SetText* (IN text: ARRAY OF CHAR): BOOLEAN;
    BEGIN
        RETURN HostClipboardSys.SetText(text) # 0
    END SetText;

    (** Read text from the system clipboard into `text`.
        Returns TRUE on success, FALSE if clipboard is empty / not text. *)
    PROCEDURE GetText* (OUT text: ARRAY OF CHAR): BOOLEAN;
    BEGIN
        RETURN HostClipboardSys.GetText(text) # 0
    END GetText;


END HostClipboard.
