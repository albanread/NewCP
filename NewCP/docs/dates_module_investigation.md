# Dates module — port status

## Current state — fully landed, end-to-end working

Three CP modules and one Rust runtime module ported. The full
`Dates.GetDate(d)` / `Dates.DayOfWeek(d)` / `Dates.GetEasterDate(year, d)` /
`Dates.TimeToString(t, str)` chain runs end-to-end and is verified by
the test suite.

| Layer | File | Status |
|---|---|---|
| Rust runtime | [src/newcp-runtime/src/host_date_sys.rs](../src/newcp-runtime/src/host_date_sys.rs) | ✅ 5 C-ABI shims: `GetUTCTime`, `GetLocalTime`, `GetUTCBias`, `DateToString`, `TimeToString`. Uses `std::time::SystemTime` + a portable Hinnant civil-from-days decomposition. |
| CP definition | [Mod/HostDateSys.cp](../Mod/HostDateSys.cp) | ✅ Flat-API definition module exposing the runtime symbols. |
| CP abstract | [Mod/Dates.cp](../Mod/Dates.cp) | ✅ Faithful port of BlackBox `System/Mod/Dates.odc`: `Date`, `Time`, `Hook` + 5 abstract methods + `Day`/`DayToDate`/`DayOfWeek`/`GetEasterDate`/`ValidDate`/`ValidTime`/`SetHook`. |
| CP concrete | [Mod/HostDates.cp](../Mod/HostDates.cp) | ✅ `StdHook` subclass forwarding to HostDateSys; module body installs `theHook` as the active `Dates.hook`. |
| Tests | `tests::dates_*` in [tests/newcp-tests/src/lib.rs](../tests/newcp-tests/src/lib.rs) | ✅ 16 tests: 10 pure-arithmetic, 5 clock+formatting, all pass. |

## What's covered by the test suite

| Test | What it asserts |
|---|---|
| `dates_day_ordinal_for_2026_05_09` | `Day` formula returns a sane ordinal |
| `dates_day_round_trip` | `DayToDate(Day(d))` = `d` for 2000-02-29 (leap day) |
| `dates_weekday_may9_2026_is_saturday` | `DayOfWeek` returns 5 for Sat 2026-05-09 |
| `dates_weekday_2024_jan_1_is_monday` | `DayOfWeek` returns 0 for Mon 2024-01-01 |
| `dates_easter_2024` | Gauss algorithm: Easter 2024 = March 31 |
| `dates_easter_2025` | Gauss algorithm: Easter 2025 = April 20 |
| `dates_feb29_in_leap_year_is_valid` | `ValidDate` accepts 2024-02-29 |
| `dates_feb29_in_nonleap_is_invalid` | `ValidDate` rejects 2023-02-29 |
| `dates_valid_time_midnight` | `ValidTime` accepts 00:00:00 |
| `dates_valid_time_24h_rejected` | `ValidTime` rejects hour=24 |
| `dates_get_date_returns_recent_year` | Hooked clock returns a year in 2020..2100 |
| `dates_get_utc_date_returns_recent_year` | Same for UTC |
| `dates_get_time_fields_in_range` | `GetTime` returns a time satisfying `ValidTime` |
| `dates_date_to_string_non_empty` | `DateToString` writes at least one character |
| `dates_time_to_string_zero_pads` | `TimeToString(7,5,3)` formats as `"07:05:03"` |

## Compiler fixes that fell out of doing this

Driving Dates surfaced two cross-module IR-layer issues that needed
real fixes — not workarounds:

1. **Records pass by reference at the call ABI.** Previously the IR
   layer only special-cased open arrays and fixed arrays; record-typed
   value/IN args fell through to `lower_expr`, which loaded the
   record value and tried to GEP into it at the callee. LLVM rejects
   GEP on a struct value (`GEP base pointer is not a vector or a
   vector of pointers`).

   Fix: `lower_args_with_signature` now treats any record-typed arg
   the same way it treats a fixed array — pass `designator_addr`
   (the address) rather than `lower_expr` (the value). New helper
   `semantic_resolves_to_record` walks Named aliases (local + cross-
   module) to detect record types.

   Required source-side change: abstract method declarations that
   take a record by value (`(t: Time; ...)`) need to use `IN t: Time`
   to make the by-reference contract explicit. Without `IN`, the
   abstract sig declares value-mode but every caller passes by
   reference — ABI mismatch shows up as a misaligned-pointer crash
   in the receiver. Both `Dates.HookDesc.DateToString` and
   `Dates.HookDesc.TimeToString` were updated to use `IN`.

2. **Cross-module imported-procedure signatures need module-name
   qualification.** `imported_callee_procedure_type` returns the
   imported procedure's signature with `Named { module: None,
   name: T }` for any T defined locally inside that module. From
   the caller's perspective those refs lose their source-module
   identity, which broke `semantic_resolves_to_record` (it
   couldn't find "Date" in the importing module's symbols and
   defaulted to "not a record").

   Fix: `imported_callee_procedure_type` now applies
   `qualify_local_named_refs_in_sem_type` to every parameter and
   the result type, mirroring what we already do for imported
   record symbols. Same trick: rewrite `Named { module: None }` to
   `Named { module: Some(<that module>), kind: Imported }` if the
   name is one of the source module's top-level types.

## Architectural pattern (repeats from Files)

Dates uses the same four-layer split that Files / HostFiles / HostFileSys
established:

1. **Flat C-ABI runtime** in Rust (`host_date_sys.rs`) — pure
   primitives, no CP awareness, just `std::time` calls + a
   decomposition algorithm.
2. **CP definition module** (`HostDateSys.cp`) — declares the runtime
   symbols' signatures so the type-checker / loader can resolve them.
3. **Abstract OOP interface** (`Dates.cp`) — BlackBox-faithful surface:
   record types, `Hook` abstract record with abstract methods, the
   pure-value helpers that don't need a hook (`Day`, `DayOfWeek`,
   `ValidDate`, `GetEasterDate`).
4. **Concrete subclass** (`HostDates.cp`) — `StdHook` subclass whose
   methods forward to the flat runtime; module body installs the
   active hook so importers get a working clock for free.

This is now a proven template for any "abstract OOP interface with a
host-platform backend" port. Next candidates that fit the same shape:
`Services` (timers + async), `Log` (text-buffer-backed) once
TextModels is ported.

## What's intentionally simplified

- `GetUTCBias` returns 0 — local time is treated as UTC. Real
  timezone handling would need a `chrono` dep or a `libc`/Win32
  call; deferred until something actually needs it.
- `DateToString` formats are American-defaulted (`"M/D/Y"` for
  short, etc.) rather than locale-aware.
