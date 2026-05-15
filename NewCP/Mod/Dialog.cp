MODULE Dialog;
(*
   Stub slice of the BlackBox `Dialog` module.

   `Dialog` is the runtime that backs the cross-module property-
   sheet / form-binding plumbing.  The full module (~2200 lines)
   wires CP-side bound variables to UI controls via a set of
   abstract hook types (`Beep`, `GetHook`, `SetHook`, …),
   provides commands for showing form dialogs, and hosts the
   tree-control state used by the side panel.

   This slice ships only the types other framework modules
   reference at the type level — most importantly `Color`, which
   `Properties.StdProp.color` carries.  The full Dialog surface
   (the bound-variable protocol, hooks, tree controls, command
   procedures) is deferred.
*)

    CONST
        (** Stable named-color sentinels for the property bag.
            BlackBox values match BB's `Dialog.Color` constants. *)
        background* = 0FF000000H;


    TYPE
        (** Logical color.  Carries an RGB-ish 32-bit value;
            concrete `Properties.StdProp.color` round-trips this
            through the property channels.  BlackBox makes this a
            single-field record so future evolution can add a
            colour-space tag without breaking ABI. *)
        Color* = RECORD
            val*: INTEGER
        END;

        (** BB-faithful dialog string buffer — 256 chars.  Used
            as a name buffer across the framework (Converters'
            `imp`/`exp` proc-name slots, command names supplied
            to `Dialog.Call`, etc.). *)
        String* = ARRAY 256 OF CHAR;


    (** BB-faithful Dialog.Beep — emits the system attention
        sound.  Console echo only in this slice (no GUI yet);
        the host-bridge replacement plugs in when HostDialog ports. *)
    PROCEDURE Beep*;
    BEGIN
        (* no-op: deferred until HostDialog provides the system bell *)
    END Beep;

    (** BB-faithful Dialog.ShowMsg — pops up a modal alert.
        BB's body locates the message resource by name (`"#mod:key"`)
        and invokes the registered display hook.  This slice has no
        hook and no resource catalog; the call is a no-op so
        Converters' "no converter found" path doesn't trap. *)
    PROCEDURE ShowMsg* (IN msg: ARRAY OF CHAR);
    BEGIN
        (* no-op: deferred until the message-resource hook lands *)
    END ShowMsg;

    (** BB-faithful Dialog.ShowParamMsg — substituting variant of
        ShowMsg.  Same deferral applies. *)
    PROCEDURE ShowParamMsg* (IN msg, p0, p1, p2: ARRAY OF CHAR);
    BEGIN
        (* no-op *)
    END ShowParamMsg;

    (** BB-faithful Dialog.Call — dispatches a reflection-style
        command like `"StdMenuTool.UpdateAllMenus"`.  BB resolves
        the command name through `Meta.LookupPath` and invokes
        the resulting procObj.  Until `Meta.LookupPath` returns
        real results (this slice's stub returns undef), this body
        sets `res # 0` to flag "command not found"; callers in
        Init / Config tolerate that. *)
    PROCEDURE Call* (IN cmd, args: ARRAY OF CHAR; OUT res: INTEGER);
    BEGIN
        res := -1
    END Call;

END Dialog.
