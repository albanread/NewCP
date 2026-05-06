# Loader Production Tracker

This tracker is for getting `newcp-loader` from bring-up status to a production-ready subsystem for the NewCP runtime.

## Status key

- `[x]` implemented and validated
- `[~]` partially implemented or structurally present
- `[ ]` not started

## Current status

- `[x]` recursive source graph discovery
- `[x]` dependency-ordered source-backed registration
- `[x]` active versus retired generation tracking
- `[x]` execution-scope based quiescence tracking
- `[x]` generation-aware CP export naming and loader-supplied CP import binding
- `[x]` executable residency for source-backed CP dependency chains
- `[x]` replacement safety and rollback semantics
- `[x]` stable public session API and structured diagnostics
- `[x]` driver/bootstrap integration as the primary orchestration path
- `[x]` explicit invalidation and recovery reporting
- `[x]` mixed native/source-backed command invocation API
- `[x]` broader reload and failure-path coverage
- `[ ]` optional persistent cache for warm-start scans

## Milestones

### M1. Safe replacement semantics

- `[x]` keep old generations resident until quiescent GC
- `[x]` pin retired generations through execution scopes
- `[x]` preserve the last known good active generation if a replacement build fails
- `[x]` preserve the last known good active generation if dependency graph rebuild fails midway
- `[x]` report replacement failure without corrupting active runtime state

Exit criterion:

- a failed edit or failed rebuild never leaves the loader without a usable previously-active generation

### M2. Public loader session API

- `[x]` add a stable status/report type for session state
- `[x]` add `ensure_command_loaded`
- `[x]` add loader-owned invoke path for source-backed commands
- `[x]` expose retired/active generation summaries as structured data instead of only strings

Exit criterion:

- the driver can use `LoaderSession` directly without custom orchestration logic

Status: functionally complete for the current CLI surface. Additional API refinement can happen later without reopening the basic design.

### M3. Failure and recovery behavior

- `[x]` distinguish parse/sema/codegen/materialization/runtime-registration failures
- `[x]` keep old active generations live on failure
- `[x]` keep session caches consistent after a failed rebuild
- `[x]` add focused tests for mixed success/failure reload graphs

Exit criterion:

- loader failures are recoverable and do not require session restart for ordinary edit-fix-retry loops

Status: complete for the current loader scope. Recovery and invalidation state are explicit in session status, and edit-fix-retry behavior is covered by focused regression tests.

### M4. Driver/bootstrap integration

- `[x]` route `load-module` through a loader session
- `[x]` route command invocation through loader session scope tracking when the target resolves to source-backed modules
- `[x]` use loader session during bootstrap/module bring-up instead of ad hoc flows

Exit criterion:

- the primary runtime bring-up path uses the loader as the canonical orchestration layer

### M5. Performance and persistence

- `[ ]` add persistent cache only if filesystem scans become a measurable bottleneck
- `[ ]` persist derived graph metadata only
- `[ ]` persist invalidation inputs, not authoritative semantics

Exit criterion:

- persistence improves startup time without changing semantic correctness

## Current work item

Current focus: loader work parked after M3 completion. Revisit only if a concrete runtime integration or performance bottleneck requires it.