MODULE HostRegistry;
(*
   First slice of the BlackBox `HostRegistry` port.

   BB's HostRegistry is a small (~300-line) Win32 wrapper over
   `RegOpenKey` / `RegQueryValueEx` / `RegSetValueEx` — the
   per-installation preference store.  Used by `Dialog.GetIntPref`
   / `Dialog.GetStringPref` / `Properties.RegisterUnitOptions`
   and a handful of Std/* form-binding widgets.

   This slice ships the public surface as no-op stubs that
   report "key not found" (`res # 0`).  Framework callers (Dialog,
   StdMenuTool, StdDialog) already tolerate missing keys — they
   fall back to compiled-in defaults.  Real persistence ports
   alongside the OS-side preference plumbing we don't have yet.

   Deferred: every body.  Bring the registry online once we
   pick a persistence backend (Windows registry, plist, dotfile).
*)

    (** Read a string-valued preference.  Stub: res # 0. *)
    PROCEDURE ReadString* (IN key: ARRAY OF CHAR; OUT x: ARRAY OF CHAR; OUT res: INTEGER);
    BEGIN
        x[0] := 0X;
        res  := -1
    END ReadString;

    (** Read an integer-valued preference.  Stub: res # 0. *)
    PROCEDURE ReadInt* (IN key: ARRAY OF CHAR; OUT x: INTEGER; OUT res: INTEGER);
    BEGIN
        x   := 0;
        res := -1
    END ReadInt;

    (** Read a boolean-valued preference.  Stub: res # 0. *)
    PROCEDURE ReadBool* (IN key: ARRAY OF CHAR; OUT x: BOOLEAN; OUT res: INTEGER);
    BEGIN
        x   := FALSE;
        res := -1
    END ReadBool;

    (** Read an INTEGER list preference.  Stub: res # 0. *)
    PROCEDURE ReadIntList* (IN key: ARRAY OF CHAR; OUT x: ARRAY OF INTEGER; OUT res: INTEGER);
    BEGIN
        res := -1
    END ReadIntList;


    (** Write a string-valued preference.  Stub: drop on the floor. *)
    PROCEDURE WriteString* (IN key, str: ARRAY OF CHAR);
    BEGIN
    END WriteString;

    (** Write an integer-valued preference.  Stub. *)
    PROCEDURE WriteInt* (IN key: ARRAY OF CHAR; x: INTEGER);
    BEGIN
    END WriteInt;

    (** Write a boolean-valued preference.  Stub. *)
    PROCEDURE WriteBool* (IN key: ARRAY OF CHAR; x: BOOLEAN);
    BEGIN
    END WriteBool;

    (** Write an INTEGER list preference.  Stub. *)
    PROCEDURE WriteIntList* (IN key: ARRAY OF CHAR; IN x: ARRAY OF INTEGER);
    BEGIN
    END WriteIntList;

END HostRegistry.
