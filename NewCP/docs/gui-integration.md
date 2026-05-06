# Wingui Integration Design & Tracker

This document outlines the architecture and implementation plan for integrating the declarative `multiwingui` (SuperTerminal) framework with the NewCP Component Pascal environment.

## 1. Architecture

The legacy BlackBox Component Builder tightly coupled its UI and compilation environment to a single Win32 message loop thread. NewCP breaks this coupling using the `multiwingui` SuperTerminal model:

*   **UI Thread (Foreground):** Managed entirely by `multiwingui`. It creates the OS windows, runs the Win32 message pump, maintains the Direct3D 11 rendering contexts, and manages the declarative native UI state (JSON).
*   **CP Thread (Background):** Managed by `newcp-driver`. The NewCP runtime (JIT, compiler, garbage collector, and the running CP app) operates on a background client thread. It NEVER touches Win32 `HWNDs` or Direct3D APIs directly.

## 2. Cross-Thread Communication

Communication between the OS UI and the CP application occurs exclusively through typed lock-free queues (channels) exposed via the C API:

*   **Event Queue (UI → CP):** Captures keyboard, mouse, and resize events. The CP standard library (`HostControllers` / `Controllers`) polls this queue to route interactions to the focused CP View.
*   **Command Queue (CP → UI):** When CP code needs to draw, create a window, or update a form, it pushes commands to this queue. Operations include patching the JSON declarative layout, injecting text into text-grid panes, or writing pixels to RGBA panes.

## 3. Implementation Tracker

### Phase 1: Rust FFI Scaffolding
- [x] Read and map the `multiwingui/include/wingui/terminal.h` C API.
- [x] Create `wingui_ffi.rs` in `newcp-runtime`.
- [x] Define Rust structs for `SuperTerminalClientContext`, `SuperTerminalWindowDesc`, and IDs.
- [x] Define `unsafe extern "system"` bindings for creation, layout publishing, and queue polling.
- [x] Add `build.rs` to link against `wingui.lib` and required Win32 system libraries.

### Phase 2: Safe Rust Wrappers
- [x] `wingui_host.rs`: `OnceLock<usize>` stores the context pointer set in the SuperTerminal startup callback.
- [x] `run_host()` entry point: fills `SuperTerminalAppDesc`, calls `super_terminal_run`, blocks the main thread.
- [x] Shim functions registered as `extern "C"` with `#[unsafe(export_name)]`:
  - `HostWindows.RequestPresent`
  - `HostWindows.RequestClose`
  - `HostWindows.PublishUi` (JSON layout, default window)
  - `HostWindows.PatchUi` (JSON patch, default window)
  - `HostWindows.WaitEvent` (blocking event poll, returns event-type integer)
- [x] `wingui_host::native_module_artifact()` registered in `Runtime::new()`.

### Phase 3: Component Pascal Host Integration
- [x] `Mod/HostWindows.cp`: CP stub module with procedure bodies matching shim signatures.
- [ ] Update `newcp-driver` to call `wingui_host::run_host()` and spawn the CP worker.
- [ ] Map legacy `System.Views` drawing commands to `multiwingui` commands.

### Phase 4: Driver Integration
- [x] `run-gui [Module.Command]` command added to `newcp-driver`.
- [x] `run_gui()` calls `wingui_host::run_host()` — blocks the main thread on the Win32 message loop.
- [x] `cp_worker_startup` callback bootstraps the resident kernel on the background CP thread.
- [ ] Wire `_command_path` into `cp_worker_startup` to invoke a specific CP command on launch.
