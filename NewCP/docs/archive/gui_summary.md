# GUI Design Summary

## Overview

The current design is a split-thread architecture with a deliberate boundary.

The Component Pascal side owns the full MVC system. Models, controllers, and
logical views all live on the language thread, just as they effectively did in
BlackBox. That means the language runtime is free to use fine-grained
synchronous model/view/controller messages, nested micro-views, targeted
invalidation, and whatever document semantics it needs without paying
cross-thread coordination costs for every small operation.

The UI side is a pane-based SuperTerminal. It owns windows, panes, Direct3D,
shaders, native controls, layout reconciliation, and hardware-accelerated
drawing. It does not own document semantics. It receives pane-scoped render
instructions and executes them efficiently.

That does not mean all panes are interchangeable.

- `surface` is the richer, purpose-built target for ordinary MVC rendering.
- `text-grid`, `rgba-pane`, `indexed-graphics`, and similar fast panes remain
  specialized high-speed paths for workloads such as monospaced editing, pixel
  graphics, sprite systems, and games.
- MVC may later wrap those fast panes as special-purpose embedded views, but they
  are not the basis for general document rendering.
- we should not drift into fake `surface` implementations that simply reuse the
  existing fast panes and thereby throw away the richer primitive contract needed
  for BlackBox-style MVC.

## Core Design

There are really two systems.

1. The CP thread is the semantic system.
   It runs the application, document model, controller logic, view
   composition, micro-view traversal, and display-list generation.

2. The UI thread is the rendering system.
   It runs Win32, D3D11, spec binding, pane management, frame draining, and GPU
   execution.

The important consequence is that we no longer try to split the view across
threads. The view is logically on the CP thread. What crosses to the UI thread
is not view logic, but render output.

That leads to a strict rule:

- Only panes get fast channels.
- Individual embedded views do not get their own channels.
- Micro-views render into their parent pane's batch.
- Only flattened rendering commands, invalidation hints, and UI-side responses
  cross the thread boundary.

And there is a second strict rule:

- General-purpose MVC rendering targets `surface`.
- Fast panes keep their own specialized contracts.
- If MVC uses a fast pane later, it does so through an intentional wrapper view,
  not by collapsing the whole MVC surface design onto that pane.

## Features This Enables

This design is meant to preserve BlackBox-style richness while avoiding the
earlier freezes and architectural confusion.

It supports:

- Fine-grained MVC messaging on the language side, including targeted updates
  and embedded views.
- Multiple views observing the same model, with each root view targeting its
  own pane.
- A clean split between the general MVC surface and specialized high-speed panes,
  so performance-oriented features do not force the ordinary document surface into
  the wrong abstraction.
- Fast-path rendering for editors, graphs, animation, caret and selection
  overlays, and other high-frequency visuals.
- Slow-path declarative UI updates for native widgets and window structure
  through the spec/JSON path.
- Hardware acceleration on the UI side through shaders and GPU-backed pane
  rendering.
- Future extension of the render vocabulary without moving application
  semantics onto the UI thread.

In short, the CP side stays expressive, and the UI side stays fast.

## Goals

The current goals are straightforward.

- Recreate BlackBox-level document and text-model behavior without
  reintroducing single-thread coupling.
- Keep all CP/JIT execution off the UI thread.
- Let the language side remain arbitrarily granular in its internal MVC
  protocol.
- Reduce cross-thread traffic to pane-scoped rendering commands and explicit
  UI responses.
- Make the UI terminal programmable enough that if performance pressure
  appears, we solve it with better pane commands or shaders, not by leaking
  model/view logic into the UI thread.
- Build a foundation that can handle text editing, embedded objects, graphics
  panes, and mixed document content under one coherent architecture.

The short version is: MVC and document intelligence live on the language
thread; panes and accelerated drawing live on the UI thread; channels connect
them; only rendering crosses fast.