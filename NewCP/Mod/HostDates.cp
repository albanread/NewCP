MODULE HostDates;
(*
   Concrete Dates.Hook implementation backed by HostDateSys.

   BlackBox Host/Mod/HostDates.odc calls Win32 GetLocalTime /
   GetSystemTime / GetTimeZoneInformation. NewCP routes those primitives
   through the Rust runtime (std::time) via the flat HostDateSys
   facade — same pattern as HostFiles wrapping HostFileSys.

   Module body installs `theHook` as the active Dates hook so any
   importer of `Dates` immediately gets a working clock.
*)

IMPORT Dates, HostDateSys;

TYPE
    StdHookDesc* = RECORD (Dates.HookDesc)
        marker-: INTEGER     (* placeholder so the record is non-empty *)
    END;
    StdHook* = POINTER TO StdHookDesc;

VAR
    theHook-: StdHook;


PROCEDURE (h: StdHookDesc) GetTime* (OUT d: Dates.Date; OUT t: Dates.Time);
BEGIN
    HostDateSys.GetLocalTime(d.year, d.month, d.day,
                             t.hour, t.minute, t.second)
END GetTime;

PROCEDURE (h: StdHookDesc) GetUTCTime* (OUT d: Dates.Date; OUT t: Dates.Time);
BEGIN
    HostDateSys.GetUTCTime(d.year, d.month, d.day,
                           t.hour, t.minute, t.second)
END GetUTCTime;

PROCEDURE (h: StdHookDesc) GetUTCBias* (OUT bias: INTEGER);
BEGIN
    bias := HostDateSys.GetUTCBias()
END GetUTCBias;

PROCEDURE (h: StdHookDesc) DateToString* (IN d: Dates.Date; format: INTEGER;
                                          OUT str: ARRAY OF CHAR);
BEGIN
    HostDateSys.DateToString(d.year, d.month, d.day, format, str)
END DateToString;

PROCEDURE (h: StdHookDesc) TimeToString* (IN t: Dates.Time;
                                          OUT str: ARRAY OF CHAR);
BEGIN
    HostDateSys.TimeToString(t.hour, t.minute, t.second, str)
END TimeToString;


BEGIN
    NEW(theHook);
    Dates.SetHook(theHook)
END HostDates.
