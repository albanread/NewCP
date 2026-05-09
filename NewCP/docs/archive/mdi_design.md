# MDI Frame Design

## Purpose

This document defines how a wingui top-level window can act as an MDI frame
that hosts multiple child document windows, instead of those children being
free-floating top-level windows. The motivation is the BlackBox Component
Builder UX: one main application window with a menu / command bar / status
bar, and N document windows living inside the frame's client area. Children
are owned by the frame, share its menu, and close when the frame closes.

This is a chrome-and-window-management concern, not a rendering one. Surface,
text-grid, rgba-pane, and indexed-graphics panes work the same inside an MDI
child as they do in a top-level window — the only thing that changes is who
the child window's HWND parent is.

## Non-goals

- Modern dockable/floating panel layouts (VS Code / Visual Studio style).
  Real Win32 MDI is the first slice. Custom-rendered docking with split
  views and tear-off floating windows is a separate, larger workstream and
  is explicitly out of scope here.
- Cross-platform MDI. The first slice targets Windows MDI only. Other hosts
  remain free-floating top-level windows; the same wingui spec stays valid.
- Replacing the existing top-level window model. Top-level windows continue
  to exist — MDI is an opt-in mode for windows that explicitly want it.
- Visual styles. Win32 MDI without a Common Controls v6 manifest renders in
  classic NT4/9x chrome. That matches the BlackBox look. Modernising the
  chrome is a separate decision.

## Feasibility summary

Doable, with bounded work. The most expensive piece — multi-window plumbing
end-to-end — is already in place. What is missing is the parent/child
relationship and the MDI-frame chrome.

What already exists:

- `super_terminal_create_window` creates additional top-level windows.
- `SuperTerminalWindowId` is threaded through every command and event API.
- `wingui_spec_bind_runtime_resolve_pane_id_for_window` already namespaces
  pane IDs per window. MDI children become more windows, so their panes
  inherit per-window ID scoping for free.
- `SuperTerminalWindowDesc.flags` is an unused `uint32_t` slot, so adding
  an `mdi_frame` capability bit is ABI-safe.
- Each surface / rgba / text-grid pane already creates its own
  `WinguiContext` keyed off the pane's HWND. MDI children can host them
  unchanged.

What is missing:

- A way to mark a window as an MDI frame (gives the window an MDI client
  area inside its body region).
- A way to open a child window with `parent_window_id` set, which causes
  the host to create it as an MDI child of that frame's MDI client.
- Event routing that includes the source window ID, so CP can dispatch
  per-document.
- A small CP surface (`HostWindows.OpenChildWindow` / `CloseChildWindow`)
  on top of the FFI changes.

Risk assessment: low–medium. Win32 MDI is well-trodden territory. The main
implementation risk is making sure surface-pane Direct2D contexts behave
correctly when their HWND parent is an MDI child rather than a top-level
window, but the existing per-pane `wingui_create_context` path already
treats each pane HWND independently, so this should be mechanical.

## Architectural decision

Reuse the existing `WindowId` model. An MDI child is just a `Window` whose
parent is another `Window`, not a separate `Document` concept. This keeps
the FFI surface narrow and lets CP-side code that already operates on
`WindowId` continue to work.

A frame is created by setting one bit in `WindowDesc.flags`:

```text
SUPERTERMINAL_WINDOW_FLAG_MDI_FRAME = 1u << 0
```

A child is created by passing the frame's `WindowId` as
`parent_window_id` to a new `create_window` overload. The host arranges
the child as a real Win32 MDI child of the frame's MDI client.

## Wire-level design

### C ABI changes (multiwingui)

Add one bit to `WindowDesc.flags`:

```c
#define SUPERTERMINAL_WINDOW_FLAG_MDI_FRAME 0x00000001u
```

When this bit is set on a window's create-window call, the host wraps the
window's body in an MDI client window (`MDICLIENT` class) instead of placing
the body content directly. The body's content tree becomes the MDI frame's
"non-client" content (menu / command bar / status bar / fallback area
behind the MDI client).

Add a child-create entry point — additive, does not touch existing
`super_terminal_create_window`:

```c
WINGUI_API int32_t WINGUI_CALL super_terminal_create_child_window(
    SuperTerminalClientContext* ctx,
    SuperTerminalWindowId parent_window_id,
    const SuperTerminalWindowDesc* desc,
    SuperTerminalWindowId* out_window_id);
```

Behavior:

- `parent_window_id` must refer to a window created with
  `SUPERTERMINAL_WINDOW_FLAG_MDI_FRAME`. Else the call returns 0 with
  `wingui_last_error_utf8` set.
- The returned child `WindowId` participates in every existing per-window
  API (publish UI, resolve pane id for window, post pane msg, …).
- Closing the parent closes all its children first.

The spec_bind layer mirrors with:

```c
WINGUI_API int32_t WINGUI_CALL wingui_spec_bind_runtime_create_child_window(
    WinguiSpecBindRuntime* runtime,
    SuperTerminalWindowId parent_window_id,
    const SuperTerminalWindowDesc* desc,
    SuperTerminalWindowId* out_window_id);
```

### Event payload changes

Today's `dispatchUiEventJson` ([native_ui.cpp:3619](../../multiwingui/src/native_ui.cpp))
emits `{"type":"ui-event","event":"...","source":"native-win32",...}` with
no source window field. Add `"window_id"`:

```json
{
  "type": "ui-event",
  "event": "clear_log",
  "source": "native-win32",
  "window_id": 12,
  "id": "clear_log"
}
```

The event still flows through the existing event queue; CP's
`WinPayload.GetInt` can read the `window_id` field. Existing single-window
apps that ignore `window_id` continue to work.

### Rust runtime (newcp-runtime)

`HostWindows` gains two new exports:

```rust
#[unsafe(export_name = "HostWindows.OpenChildWindow")]
pub extern "C" fn host_open_child_window(
    parent_window_id: i64,
    json_ptr: *const u8,
    out_child_window_id: *mut i64,
) -> i32;

#[unsafe(export_name = "HostWindows.CloseChildWindow")]
pub extern "C" fn host_close_child_window(child_window_id: i64) -> i32;
```

Both forward to the new `create_child_window` / `close_window` entry points
on `WinguiSpecBindRuntime` and propagate the result.

### CP-side surface (HostWindows / WinView)

`HostWindows.cp` (the existing DEFINITION module) adds two heading-only
declarations:

```cp
PROCEDURE OpenChildWindow*(parentWindowId: INTEGER;
                           spec: ARRAY OF SHORTCHAR;
                           VAR childWindowId: INTEGER): INTSHORT;

PROCEDURE CloseChildWindow*(childWindowId: INTEGER): INTSHORT;
```

`WinView.cp` (or a thin wrapper) grows two helpers that build a child
window's spec the same way `Render` builds a top-level spec:

```cp
PROCEDURE RenderChild* (parentWindowId: INTEGER;
                        renderer: RenderProc;
                        VAR childWindowId: INTEGER): INTSHORT;

PROCEDURE CloseChild* (childWindowId: INTEGER);
```

A real CP demo would look like:

```cp
PROCEDURE OnNewDocument*;
  VAR docId: INTEGER;
      ok:    INTSHORT;
BEGIN
  ok := WinView.RenderChild(App.frameId, BuildDocSpec, docId);
  IF ok # 0 THEN
    Documents.RegisterDoc(docId)
  END
END OnNewDocument;
```

## What stays unchanged

- Spec JSON for non-frame windows.
- All pane types (surface, text-grid, rgba-pane, indexed-graphics) and
  their batch protocols.
- Per-pane FFI calls — they keep working because they already key off
  pane HWND.
- The `EVENT_QUEUE` consumer path. Events arrive with an extra
  `window_id` field that single-window apps can ignore.
- The `[wingui-trace]` and `native_patch_trace.log` instrumentation.

## Implementation phases

### Phase 1 — host frame + MDI client area

`multiwingui/src/native_ui.cpp` and the matching native window code:

1. Honor `SUPERTERMINAL_WINDOW_FLAG_MDI_FRAME` at window-create time.
   Insert an `MDICLIENT` window inside the frame's content area.
2. Resize the MDI client when the frame body's reserved area resizes.
3. Verify the existing menu / command bar / status bar still render
   above the MDI client area.

Acceptance: a frame window opens with a visibly empty MDI client area
inside it, no children yet.

### Phase 2 — MDI child create / destroy

1. Add `super_terminal_create_child_window` and the spec_bind alias.
2. Route `WM_MDICREATE` / `WM_MDIDESTROY` for the new children.
3. Each child gets its own `WindowId` and is registered in the host
   window table the same way top-level windows are.
4. Closing the parent cascades: enumerate children, send `WM_MDIDESTROY`,
   release host state.

Acceptance: open a frame, open two child windows under it, close one,
close the parent — no leaks, no orphan HWNDs.

### Phase 3 — per-window event routing

1. Add `"window_id"` to the JSON payload of every `dispatchUiEventJson`
   call site.
2. Plumb the field into `parseNativeUiEventFields` so the `EventView`
   carries it (or extend the payload to include it for downstream Rust
   parsing).
3. Update `[wingui-trace] kind=submit` and friends to include the
   originating window where relevant.

Acceptance: a button click in a child fires an event that includes the
child's `WindowId`. Two child windows with same-named buttons can be
distinguished.

### Phase 4 — Rust + CP surface

1. Add `HostWindows.OpenChildWindow` / `CloseChildWindow` Rust exports
   and register them in `native_module_artifact`.
2. Add the matching `HostWindows.cp` heading-only declarations.
3. Add `WinView.RenderChild` / `CloseChild`.
4. Update `App.cp` (or a new demo) to open one MDI child with a
   text-grid and one with a surface, prove both paint correctly.

Acceptance: an MDI demo with two child documents, each with its own
panes, both painted simultaneously, both reachable via per-window event
routing.

### Phase 5 — Window menu and standard MDI commands

1. Auto-populate the `Window` menu (or equivalent) with the list of
   open MDI children, with the active child checked.
2. Wire `Cascade`, `Tile Horizontally`, `Tile Vertically`, `Arrange Icons`
   commands. These are one-line `WM_MDICASCADE` / `WM_MDITILE` /
   `WM_MDIICONARRANGE` SendMessage calls; no logic to write.

Acceptance: the standard MDI Window menu appears and works.

### Phase 6 — polish (optional)

- Active-child title in the frame title bar when the child is maximised.
- Per-child dirty-mark behaviour (close prompt for unsaved changes).
- Drag-out to top-level (advanced; not BlackBox-equivalent).

## Open questions

- **Where should the frame's menu come from?** BlackBox uses one
  application menu shared by all children. For NewCP this is naturally
  provided by the frame window's existing menu spec. CP-side: the same
  `WinView.SetRenderer` covers it. No new mechanism needed.

- **Should child specs share or duplicate widget IDs?** Per-window event
  routing makes shared IDs unambiguous, but it might still be cleaner to
  require unique IDs across siblings. Recommend: enforce uniqueness in
  v1, relax later if needed.

- **Pane scoping.** Surface/text-grid/etc. pane IDs are resolved
  per-window today, so a child with a `node_id="editor_main"` pane gets
  a distinct pane from any other window's `editor_main`. This is what
  we want; no work needed.

- **Modal dialogs.** Modal dialogs that block the frame should disable
  all child windows for input. Win32 MDI handles this naturally via
  `EnableWindow` on the frame; the existing dialog code path may need
  to be checked once Phase 5 lands.

- **Cross-platform fallback.** On non-Windows hosts (when those exist),
  MDI children should fall back to top-level windows so the same CP
  code keeps working. The CP code only knows `WindowId`s, which works
  the same in either model — only the chrome differs.

## Anti-goals (preventing future drift)

1. Do not add a separate `Document` concept. Children are `Window`s with
   a parent. Every existing per-`WindowId` API works on them.
2. Do not invent a custom MDI renderer. Win32 MDI is what the host gives
   us; we use it. Custom-rendered docking is a different design.
3. Do not let the frame's menu spec and the children's content tree
   share state implicitly. The frame is the menu's owner; children
   handle their own content.
4. Do not extend the spec JSON to embed children inline. Children are
   created imperatively via the new `OpenChildWindow` API. Mixing
   imperative document open/close with declarative widget layout would
   muddy the model.
