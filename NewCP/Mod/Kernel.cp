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

    (* Resolution of `Kernel.Time()` — ticks per second.
       BlackBox uses 1000 (millisecond resolution); we mirror
       that.  Services rescales user-facing tick units against
       this value. *)
    timeResolution* = 1000;

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

    (** Abstract hook base — every framework module that
        installs a runtime callback (`Services.ActionHook`,
        `Views.GetSpecHook`, `Dialog.GetHook`, …) extends this.
        Identity-only; the hook type is the public interface
        between a module's published `Set<X>Hook` setter and
        whatever subclass the host installs. *)
    HookDesc* = ABSTRACT RECORD END;
    Hook*     = POINTER TO HookDesc;

    (** Trap-cleaner base. Subclasses override `Cleanup` to roll
        back transactional state when the runtime traps. The
        runtime invokes registered cleaners LIFO before unwinding,
        so a cleaner's `Cleanup` MUST be tolerant of partial state
        (it may be called at any moment between any two CP
        statements that ran while it was active). *)
    TrapCleanerDesc* = ABSTRACT RECORD END;
    TrapCleaner*     = POINTER TO TrapCleanerDesc;

    (** Generic event record for the language-thread event loop.
        Field semantics are kind-dependent; see the EvKey / EvChar /
        EvMouse / ... constants below for the per-kind layout. The
        same wire shape iGui.NextEvent uses, so handlers can pass
        Event values straight through to lower-level dispatch
        without re-marshaling. *)
    Event* = RECORD
        kind*:    INTEGER;   (* one of EvNone, EvKey, EvChar, ... *)
        childId*: INTEGER;   (* originating child window, or 0  *)
        timeMs*:  INTEGER;   (* event timestamp, or 0           *)
        p1*:      INTEGER;
        p2*:      INTEGER;
        p3*:      INTEGER;
        p4*:      INTEGER
    END;

    (** Event-loop callback signature. The handler receives one event
        per loop iteration; setting `quit # 0` causes Loop to exit
        cleanly after the current call returns. The runtime synthesises
        an EvFrameClose event when the UI thread's frame closes, so
        handlers that want to honour OS-driven shutdown can just check
        `ev.kind = EvFrameClose` and set quit. *)
    EventHandler* = PROCEDURE (VAR ev: Event; VAR quit: INTEGER);

CONST
    (* Event kinds. Match iGui.Ev* constants byte-for-byte. *)
    EvNone*        = 0;
    EvKey*         = 1;
    EvChar*        = 2;
    EvMouse*       = 3;
    EvFocus*       = 4;
    EvResize*      = 5;
    EvPaint*       = 6;
    EvClose*       = 7;
    EvFrameClose*  = 8;
    EvMenu*        = 9;
    EvThemeChange* = 10;
    EvDpiChange*   = 11;
    EvTick*        = 13;

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

PROCEDURE GetTypeName* (t: Type; OUT name: ARRAY OF CHAR);
    (** Bare type name (e.g. "StoreDesc"). For the qualified form,
        callers concatenate `GetModName(ModOf(t))` + "." +
        `GetTypeName(t)`, OR use `GetQualifiedTypeName` directly.
        `name` is an open-array OUT parameter — the ABI passes the
        buffer's element count alongside the payload pointer, so
        callers can use any sized `ARRAY OF CHAR` (typically
        `Kernel.Name`). The shim caps writes at length-1 and zero-
        terminates. *)

PROCEDURE GetQualifiedTypeName* (t: Type; OUT name: ARRAY OF CHAR);
    (** Full qualified type name (e.g. "Stores.StoreDesc"). The
        runtime always knows the qualified form; callers that need
        it directly avoid composing it from parts. *)

PROCEDURE GetModName* (m: Module; OUT name: ARRAY OF CHAR);
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

PROCEDURE Collect* ();
    (** Explicit garbage-collection cycle.  Cooperatively pauses every
        registered mutator thread, marks live objects, sweeps dead
        ones, and runs any pending finalizers.  Returns once the
        world is fully resumed.  Cheap to call; the runtime
        short-circuits if nothing has been allocated since the last
        cycle. *)

PROCEDURE TrapCount* (): INTEGER;
    (** Snapshot of the process-wide trap counter.  Increments each
        time a recoverable trap fires (today: never, since traps
        abort).  Used by reentrancy guards in the model / sequencer
        layers — a procedure records `TrapCount() + 1` on entry,
        and on re-entry asserts the snapshot still matches.  When
        a trap fires the counter advances and the assertion catches
        the orphaned guard.  See `Models.Broadcast` / `Models.Domaincast`. *)

(* -- Event loop --------------------------------------------------------- *)

PROCEDURE Loop* (handler: EventHandler);
    (** Run the language-thread event loop. Blocks until the handler
        sets `quit # 0`, until an `EvFrameClose` arrives, or until
        another thread calls `Kernel.Quit`. On every real event the
        handler is invoked once with that event and a `quit` slot;
        between events (when no GUI input is queued) the loop runs
        an internal idle hook (GC pressure check, finalizer drain,
        retired-generation collection) without calling the handler.

        Re-entrancy: not supported. A second `Kernel.Loop` call from
        inside a handler will trap. *)

PROCEDURE Quit* (code: INTEGER);
    (** Signal the running event loop to exit at the next iteration.
        Safe to call from any thread. `code` is recorded for the
        bootstrap shell to retrieve via the future `ExitCode`
        accessor; not yet inspected. The OS-side frame is *not*
        closed by this call — pair with `iGui.Quit` if a clean
        process shutdown is required. *)

END Kernel.
