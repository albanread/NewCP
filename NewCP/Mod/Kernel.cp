DEFINITION MODULE Kernel;
(*
   NewCP `Kernel` definition module — the typed public surface CP
   framework modules import. Backed directly by the runtime
   (newcp-runtime/src/lib.rs and gc.rs) via the same DEFINITION-
   MODULE-to-Rust-shim pattern as `HostFileSys` / `HostDateSys`.

   Compatibility scope: this is the subset of the BlackBox
   `System/Mod/Kernel.odc` surface that the framework actually
   depends on (`Stores`, `Files`, `Models`, `Views`, …). Out of
   scope:
   - the `Item` / `ItemExt` reflection hierarchy (delegated to a
     future `Meta` port);
   - `Identifier` / `Reducer` (delegated to `Sequencers`);
   - inline-assembly intrinsics, `ExcpFrame` plumbing, `Address`
     space — replaced by LLVM-emitted code + Rust runtime;
   - cluster / heap-management primitives — covered by NewCP's
     unified GC.

   Divergences from BlackBox worth flagging:
   - `Module` and `Type` are *opaque* pointer aliases. BlackBox
     exposes `t.mod` and `m.name` as direct field reads; in NewCP
     all such access goes through `GetTypeName` / `GetModName` /
     `ModOf` etc. so the runtime is free to evolve `TypeDesc` /
     `ModuleDesc` layouts without breaking source compatibility.
     Stores.cp and friends will use the procedure form.
   - `TrapCleaner` is `ABSTRACT` rather than the legacy `EMPTY` —
     subclasses MUST override `Cleanup`. Stores already does.

   The implementation is provided by the runtime: every procedure
   declared here links at JIT time to a `__newcp_kernel_*` Rust
   function. The CP source declares signatures only — there is no
   CP-side body for any of them.
*)

CONST
    (* Filename suffixes — what the loader and the IDE write to
       disk. Stores reads these for type-tag pattern matching. *)
    objType* = "ocf";
    symType* = "osf";
    docType* = "odc";

    (* Endianness — NewCP targets x86_64 only in this slice. *)
    littleEndian* = TRUE;

    (* Loader result codes. Match the discriminants in
       `KernelSys.loader*` and `newcp-loader::LoaderFailurePhase`. *)
    none*           = 0;
    fileNotFound*   = 1;
    syntaxError*    = 2;
    objNotFound*    = 3;
    illegalFPrint*  = 4;
    cyclicImport*   = 5;
    invalidModule*  = 6;

TYPE
    Name* = ARRAY 256 OF CHAR;

    (** A loaded module. Opaque — use the accessor procedures
        (`GetModName`, etc.) to read its identity. *)
    ModuleDesc* = ABSTRACT RECORD END;
    Module*     = POINTER TO ModuleDesc;

    (** A runtime type descriptor. Opaque pointer; comparisons by
        identity. Use `GetTypeName`, `ModOf`, `BaseOf`, `LevelOf`,
        `SizeOf` to read its fields. *)
    TypeDesc*  = ABSTRACT RECORD END;
    Type*      = POINTER TO TypeDesc;

    (** Trap-cleaner base. Subclasses override `Cleanup` to roll
        back transactional state when the runtime traps. The
        runtime invokes registered cleaners LIFO before unwinding,
        so a cleaner's `Cleanup` MUST be tolerant of partial state
        (it may be called at any moment between any two CP
        statements that ran while it was active). *)
    TrapCleanerDesc* = ABSTRACT RECORD END;
    TrapCleaner*     = POINTER TO TrapCleanerDesc;

(* -- TrapCleaner contract ------------------------------------------------ *)

PROCEDURE (c: TrapCleanerDesc) Cleanup*, NEW, ABSTRACT;

(* -- Reflection ---------------------------------------------------------- *)

PROCEDURE ThisMod* (IN name: ARRAY OF CHAR): Module;
    (** Look up a loaded module by name. Returns NIL if not found.
        On failure, sets the loader-result state observable via
        `GetLoaderResult`. *)

PROCEDURE ThisType* (m: Module; IN typeName: ARRAY OF CHAR): Type;
    (** Look up a type by name within a module. `typeName` is the
        bare record name (e.g. "StoreDesc"), not qualified. NIL
        if not found. *)

PROCEDURE TypeOf* (obj: ANYPTR): Type;
    (** Runtime type of a heap pointer, read from its block
        header. NIL only if `obj = NIL`. *)

PROCEDURE GetTypeName* (t: Type; OUT name: Name);
    (** Bare type name (e.g. "StoreDesc"). For the qualified form,
        callers concatenate `GetModName(ModOf(t))` + "." +
        `GetTypeName(t)`. *)

PROCEDURE GetModName* (m: Module; OUT name: Name);
    (** Module name (e.g. "Stores"). Empty string if `m = NIL`. *)

PROCEDURE ModOf* (t: Type): Module;
    (** Owning module of a type. NIL if unknown. *)

PROCEDURE BaseOf* (t: Type): Type;
    (** Direct base type, or NIL if `t` is a root type. *)

PROCEDURE LevelOf* (t: Type): INTEGER;
    (** Inheritance depth: 0 for a root, 1 for a direct extension,
        and so on. Mirrors BlackBox `Kernel.LevelOf`. *)

PROCEDURE SizeOf* (t: Type): INTEGER;
    (** Payload size in bytes (excludes GC block header). *)

(* -- Allocation --------------------------------------------------------- *)

PROCEDURE NewObj* (VAR p: ANYPTR; t: Type);
    (** Heap-allocate a record of runtime type `t`. `p` receives
        the new zeroed payload pointer. Traps if `t = NIL`. This
        is the typed entry point that mirrors `Kernel.NewObj` in
        BlackBox; static `NEW(...)` calls go through the same GC
        but bypass the runtime-typed dispatch. *)

(* -- Trap-cleaner stack -------------------------------------------------- *)

PROCEDURE PushTrapCleaner* (c: TrapCleaner);
    (** Register a cleaner. The runtime invokes `c.Cleanup()` if a
        trap fires before a matching `PopTrapCleaner`. *)

PROCEDURE PopTrapCleaner* (c: TrapCleaner);
    (** Pop the matching cleaner. `c` MUST equal the value most
        recently pushed; a mismatch indicates unbalanced
        push/pop and the runtime traps. *)

(* -- Loader feedback ---------------------------------------------------- *)

PROCEDURE GetLoaderResult* (OUT res: INTEGER;
                             OUT m1, m2, m3: ARRAY OF CHAR);
    (** Read the last loader operation's result code (one of the
        constants above) plus up to three diagnostic strings.
        `res = none` means the last load succeeded. Used by
        `Stores.ThisType` to dispatch on "module-not-found" vs
        "type-not-found" etc. *)

(* -- Misc --------------------------------------------------------------- *)

PROCEDURE Time* (): LONGINT;
    (** Monotonic timestamp in nanoseconds since the Unix epoch.
        Used by `Stores.Writer.era` and IDE timestamping. *)

PROCEDURE Beep* ();
    (** System bell — best effort. Never traps. *)

END Kernel.
