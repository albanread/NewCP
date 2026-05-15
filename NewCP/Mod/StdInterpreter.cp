MODULE StdInterpreter;
(*
   First slice of the BlackBox `StdInterpreter` port.

   BB's StdInterpreter parses `"Module.Proc('arg', 99)"` strings
   into a reflection-driven procedure call through Meta — it's
   what backs `Dialog.Call(cmd, args, res)`.  Most of the work is
   private (a small Scan / Concat / CallProc machine); the only
   exported surface is a `CallHook` type that StdInterpreter
   itself installs into Dialog's hook slot at module init.

   This slice ships only the surface — module body deferred.
   `Dialog.Call` in our Dialog slice is already a no-op stub
   reporting "command not found", so callers (Init, Config)
   already tolerate the missing dispatch.

   Deferred: the entire body — every command-parse / argument-
   marshal / reflection-invoke path lands once Meta.LookupPath
   returns real items and we wire StdInterpreter as Dialog's
   CallHook.
*)


    (** Module body is a no-op for now; installing the
        StdInterpreter as Dialog's CallHook is deferred. *)
BEGIN
END StdInterpreter.
