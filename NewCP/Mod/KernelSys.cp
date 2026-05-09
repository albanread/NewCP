DEFINITION MODULE KernelSys;
(**
   Flat C-ABI Kernel primitives backed by the NewCP runtime
   (newcp-runtime/src/lib.rs and gc.rs). Exposes the runtime's
   module registry, type-descriptor walks, GC allocation, the
   trap-cleaner stack, and last-loader-result diagnostic state
   under an INTEGER-handle ABI.

   This is the host-side primitive layer that `Kernel.cp` wraps
   in CP-friendly types (Module, Type, TrapCleaner pointers).
   Direct CP clients should normally use `Kernel` instead;
   `KernelSys` is the low-level layer.

   Conventions:
   - `Module` / `Type` / `TrapCleaner` are opaque INTEGER handles.
     `0` always means "invalid / not found".
   - `Type` handles are the bytewise address of the runtime
     `TypeDesc` (`__newcp_new_rec` already takes a `TypeDesc*`);
     callers must not perform arithmetic on them.
   - `Module` handles are 1-based indices into the runtime's
     module registry. The legacy BlackBox `Module` was a pointer
     to a self-described struct; NewCP uses an indirect handle so
     the registry can move underneath without invalidating CP refs.
   - String OUT params follow the standard CP `OUT s: ARRAY OF CHAR`
     open-array ABI (UTF-32, null-terminated; the runtime accepts
     the hidden length and writes the explicit terminator).
   - `Time` returns nanoseconds since the Unix epoch (LONGINT).
*)

CONST
    (* Endianness — NewCP targets x86_64-only in this slice;
       little-endian is hard-coded so wire-codec branches can
       collapse at compile time. *)
    littleEndian* = TRUE;

    (* Loader result codes — surfaced via `LastLoaderResult` and
       mirror `newcp-loader::LoaderFailurePhase` discriminants.
       `loaderOk` indicates "no failure recorded since the last
       successful load". *)
    loaderOk*            = 0;
    loaderFileNotFound*  = 1;
    loaderSyntaxError*   = 2;
    loaderObjNotFound*   = 3;
    loaderIllegalFPrint* = 4;
    loaderCyclicImport*  = 5;
    loaderInvalidModule* = 6;

(* -- Reflection ---------------------------------------------------------- *)

PROCEDURE ThisMod*  (IN name: ARRAY OF CHAR): INTEGER;
    (** Look up a registered module by name. Returns the module
        handle (>0) or 0 if no module of that name is loaded. *)

PROCEDURE ThisType* (modHandle: INTEGER; IN typeName: ARRAY OF CHAR): INTEGER;
    (** Look up a type by name within a specific module. Returns
        the `TypeDesc` address as an opaque INTEGER, or 0 if not
        found. The `typeName` is the bare record name (e.g.
        "StoreDesc"), not the qualified form. *)

PROCEDURE TypeOf* (obj: INTEGER): INTEGER;
    (** Given a heap-pointer payload address, returns the address
        of its `TypeDesc` by reading `BlockHeader.tag` (with the
        GC mark bit masked out). 0 if `obj` is not a managed
        pointer or is NIL. *)

PROCEDURE GetTypeName* (typeHandle: INTEGER; OUT name: ARRAY OF CHAR);
    (** Writes the bare type name (e.g. "StoreDesc") into `name`.
        On failure or if the runtime hasn't yet got names for
        TypeDescs (the codegen-side type-name emission is a
        deferred prerequisite), writes "<Type@0xADDR>". *)

PROCEDURE GetModName* (modHandle: INTEGER; OUT name: ARRAY OF CHAR);
    (** Writes the module's registered name (e.g. "Stores") into
        `name`. Empty string if `modHandle = 0`. *)

PROCEDURE TypeMod* (typeHandle: INTEGER): INTEGER;
    (** Owning-module handle for a TypeDesc. 0 if unknown. *)

PROCEDURE TypeBase* (typeHandle: INTEGER): INTEGER;
    (** Direct base TypeDesc handle, or 0 if this type is a root. *)

PROCEDURE TypeSize* (typeHandle: INTEGER): INTEGER;
    (** Payload size in bytes (excludes GC `BlockHeader`). *)

PROCEDURE LevelOf* (typeHandle: INTEGER): INTEGER;
    (** Inheritance depth: 0 for a root type, 1 for a direct
        extension, etc. Mirrors BlackBox `Kernel.LevelOf`. *)

(* -- Allocation --------------------------------------------------------- *)

PROCEDURE NewObj* (VAR p: ANYPTR; typeHandle: INTEGER);
    (** Heap-allocate a record of the given runtime type via the
        GC's `__newcp_new_rec`. `p` is set to the new payload
        pointer (zeroed). Aborts via the runtime trap handler if
        `typeHandle = 0` or if allocation fails. *)

(* -- Trap-cleaner stack ------------------------------------------------- *)

(** The trap-cleaner stack is invoked LIFO when the runtime traps
    (HALT, ASSERT failure, panic). Each cleaner is a parameterless
    procedure value that the runtime calls before unwinding.

    The legacy BlackBox surface used a typed `TrapCleaner` record
    with a `Cleanup` method; the typed wrapping happens in
    `Kernel.cp`. KernelSys exposes the underlying procedure-pointer
    contract because it doesn't depend on CP method-dispatch. *)

PROCEDURE PushTrapCleaner* (cleanup: PROCEDURE; cookie: INTEGER);
    (** Register a cleanup procedure to be invoked on the next
        runtime trap. `cookie` is a caller-supplied value passed
        opaquely back through to the cleanup proc — typically the
        address of the typed record holding state to roll back. *)

PROCEDURE PopTrapCleaner* (cleanup: PROCEDURE; cookie: INTEGER);
    (** Pop and discard a cleaner registered by `PushTrapCleaner`.
        The (cleanup, cookie) pair MUST match the top of the stack;
        a mismatch indicates an unbalanced push/pop and the runtime
        traps. *)

(* -- Loader feedback ---------------------------------------------------- *)

PROCEDURE LastLoaderResult* (OUT res: INTEGER;
                              OUT m1, m2, m3: ARRAY OF CHAR);
    (** Reports the last loader operation's result code (one of the
        `loader*` constants above) and up to three diagnostic
        strings. Used by `Stores.ThisType` to disambiguate "type
        not found" from "module not loadable" when CP code reads
        a stored type descriptor referencing an unknown class. *)

(* -- Misc --------------------------------------------------------------- *)

PROCEDURE Time* (): LONGINT;
    (** Current monotonic timestamp, nanoseconds since the Unix
        epoch. Used for `Stores.Writer.era`-style stamps and for
        IDE timestamping. *)

PROCEDURE Beep* ();
    (** System bell. Best-effort no-op on hosts without an audible
        bell; never fails. *)

END KernelSys.
