# PaneDemo hang in run-igui mode

## Symptom

`newcp-driver run-igui PaneDemo.Run` opens the iGui frame but hangs
silently with no child window painted.  The CP worker thread never
returns from `LoaderSession::invoke_command`.

The hang reproduces consistently on:

- Windows 11 / 64-bit
- Inkwell-13 / LLVM-18 (whatever the workspace's `Cargo.toml` pins)
- Both with and without the `LLVM_GLOBAL_LOCK` (`src/newcp-llvm/src/lib.rs`)

## What does NOT hang

The same `LoaderSession::invoke_command` path works fine in:

| Mode | Notes |
|---|---|
| `cargo test -p newcp-tests` | 469 tests pass including ones that load TextSetters via the same import graph. |
| `newcp-driver load-module PaneDemo PaneDemo.Run` | No iGui frame thread.  Worker runs on main thread.  Demo completes; OpenChild correctly returns 0 because FRAME_HWND is unset. |
| `newcp-driver run-igui HelloPixels.Run` | Imports only `iGui, Console` — no framework chain.  Paints "Hello, pixels!" successfully. |
| `newcp-driver run-igui` with any demo importing < TextSetters | TinyDemo (iGui + Console), TinyDemo2 (+ HostFonts), T6 (+ Stores) — all open a pane. |

So the hang is specific to:
1. **`run-igui` mode** (main thread in iGui's message loop, CP code on worker thread), AND
2. **Importing TextSetters** (or anything that transitively pulls it in).

## Where the hang lives

Per-module compile tracing (instrumentation since removed) shows:

```
[loader] compile begin: Fonts                ✓
[loader]  -> compile_executable_image done   ✓
[loader] compile begin: Ports                ✓
...
[loader] compile begin: TextRulers           ✓
[loader]  -> compile_executable_image done   ✓
[loader] compile begin: TextSetters          ← hang
[loader]  -> calling compile_executable_image
                                              (never returns)
```

The freeze is INSIDE `newcp_llvm::compile_from_path(TextSetters.cp)` — i.e.
LLVM compilation of TextSetters under inkwell.  No panic, no abort —
the worker thread just stops making forward progress.  Eventually
the user closes the frame, the main thread exits, the worker is killed.

## What I ruled out

- **The LLVM_GLOBAL_LOCK** added in `a814439`.  Disabled it temporarily;
  hang still occurs.  So it's not lock contention with another thread.
- **Compile correctness**.  TextSetters compiles fine in the unit test
  suite (469 tests pass).  The IR / sema / LLVM passes are correct in
  isolation.
- **The two compiler fixes from this session**:
  - `lower.rs` array-vs-single-char string-compare (fixes a different
    bug surfaced by HostFonts.Default()).
  - `lower.rs` `resolve_named_anywhere` for array-named-type assignment.
  These fixes are NOT what's hanging; reverting them does not change
  TextSetters' hang behaviour.

## What I did NOT rule out

- Some Windows or DirectWrite/COM global the iGui frame initialises
  conflicting with what inkwell's MCJIT does for TextSetters
  specifically.  TextSetters has a large `LineBox` record with an
  inline `ARRAY 32 OF INTEGER` field plus deep cross-module inheritance
  — possibly tripping a path inkwell doesn't exercise for simpler
  modules.
- A specific LLVM optimisation pass entering an infinite loop on
  TextSetters' particular IR under run-igui's thread environment but
  not under cargo test's.  Could be deterministic but obscure.

## Next steps when picking this up

1. **Attach a debugger** (`windbg`, `cdb`, or Visual Studio) to the
   hung process, dump all threads, find the worker thread, identify
   where it's stuck.  This is the only credible way to confirm if it's
   in an LLVM pass, a Windows API call, or a mutex wait.
2. **Bisect with `cargo test` lib running concurrently** with `run-igui`
   to see if the conflict is in any concurrent loader scenario, not
   just iGui's.
3. **Try `--opt none`** in the loader.  If `OptLevel::None` doesn't
   hang, the freeze is in a specific opt pass — narrow further with
   `OptLevel::Less` etc.

## Until then

Use `HelloPixels.cp` as the "see pixels" demo.  It paints the same
content the framework would, just calling `iGui.*` directly without
the TextViews/Pane/HostPorts chain.  All other framework work continues
in unit tests where the entire chain runs cleanly.
