DEFINITION MODULE HostDateSys;
(**
   Flat C-ABI clock + date-formatting facade backed by Rust's std::time
   (see src/newcp-runtime/src/host_date_sys.rs). HostDates.cp wraps these
   primitives as a concrete Dates.Hook subclass; direct CP clients
   should normally use Dates instead.

   Conventions:
   - All time queries decompose into six OUT INTEGER values:
     (year, month, day, hour, minute, second). Local-time queries are
     identical to UTC until UTC bias is non-zero.
   - GetUTCBias returns minutes; UTC = local + bias.
   - DateToString format codes match BlackBox Dates:
       0 = short            "M/D/Y"
       1 = long             "Month D, Y"
       2 = abbreviated      "Mon D, Y"
       3 = plainLong        "D Month Y"
       4 = plainAbbreviated "D Mon Y"
   - String OUT params follow the CP `OUT s: ARRAY OF CHAR` open-array
     ABI (the runtime accepts the hidden length but uses the explicit
     null terminator).
*)

PROCEDURE GetUTCTime*   (OUT year, month, day, hour, minute, second: INTEGER);
PROCEDURE GetLocalTime* (OUT year, month, day, hour, minute, second: INTEGER);
PROCEDURE GetUTCBias*   (): INTEGER;

PROCEDURE DateToString* (year, month, day, format: INTEGER; OUT str: ARRAY OF CHAR);
PROCEDURE TimeToString* (hour, minute, second: INTEGER; OUT str: ARRAY OF CHAR);

END HostDateSys.
