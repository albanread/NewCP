MODULE DatesClock;
(* Smoke test: HostDates supplies a working hook, so Dates.GetDate
   returns a valid date and Dates.DateToString formats it. We don't
   pin to a specific clock value (would be racy); we just check that
   the year is in a sane range and the formatted string is non-empty. *)

IMPORT Dates, HostDates;

(* Year fetched from the hooked clock should be within a reasonable
   contemporary band. Returns 1 on success, the actual year on failure
   for diagnostics. *)
PROCEDURE GetDateReturnsRecentYear* (): INTEGER;
    VAR d: Dates.Date;
BEGIN
    Dates.GetDate(d);
    IF (d.year < 2020) OR (d.year > 2100) THEN RETURN d.year END;
    RETURN 1
END GetDateReturnsRecentYear;

(* Year from UTC clock — same sanity check. *)
PROCEDURE GetUTCDateReturnsRecentYear* (): INTEGER;
    VAR d: Dates.Date;
BEGIN
    Dates.GetUTCDate(d);
    IF (d.year < 2020) OR (d.year > 2100) THEN RETURN d.year END;
    RETURN 1
END GetUTCDateReturnsRecentYear;

(* GetTime fields are within valid clock range. *)
PROCEDURE GetTimeFieldsInRange* (): INTEGER;
    VAR t: Dates.Time;
BEGIN
    Dates.GetTime(t);
    IF Dates.ValidTime(t) THEN RETURN 1 ELSE RETURN -1 END
END GetTimeFieldsInRange;

(* DateToString returns at least one character for a known date. *)
PROCEDURE DateToStringNonEmpty* (): INTEGER;
    VAR d: Dates.Date; s: ARRAY 64 OF CHAR; i: INTEGER;
BEGIN
    d.year := 2026; d.month := 5; d.day := 9;
    Dates.DateToString(d, Dates.short, s);
    i := 0;
    WHILE (i < LEN(s)) & (s[i] # 0X) DO INC(i) END;
    RETURN i
END DateToStringNonEmpty;

(* TimeToString formats HH:MM:SS — for 7:5:3 we expect "07:05:03"
   so character at index 0 is '0', index 1 is '7', index 2 is ':'. *)
PROCEDURE TimeToStringFirstThree* (): INTEGER;
    VAR t: Dates.Time; s: ARRAY 64 OF CHAR;
BEGIN
    t.hour := 7; t.minute := 5; t.second := 3;
    Dates.TimeToString(t, s);
    (* Pack three CHARs into one INTEGER for easy assertion. *)
    RETURN ORD(s[0]) * 256 * 256 + ORD(s[1]) * 256 + ORD(s[2])
END TimeToStringFirstThree;

END DatesClock.
