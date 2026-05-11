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

END Dialog.
