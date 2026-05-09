MODULE DatesArith;
(* Pure-value arithmetic tests for Dates — no clock dependency.
   Validates the BlackBox Day / DayToDate / DayOfWeek / GetEasterDate
   formulas survive the port. *)

IMPORT Dates;

(* Day(2026, 5, 9) — known good ordinal for that date. *)
PROCEDURE DayOfMay9_2026* (): INTEGER;
    VAR d: Dates.Date;
BEGIN
    d.year := 2026; d.month := 5; d.day := 9;
    RETURN Dates.Day(d)
END DayOfMay9_2026;

(* Round-trip: Day(d) then DayToDate back must reproduce d. *)
PROCEDURE RoundTrip* (): INTEGER;
    VAR d, d2: Dates.Date; n: INTEGER;
BEGIN
    d.year := 2000; d.month := 2; d.day := 29;
    n := Dates.Day(d);
    Dates.DayToDate(n, d2);
    IF (d2.year # 2000) OR (d2.month # 2) OR (d2.day # 29) THEN
        RETURN -1
    END;
    RETURN 1
END RoundTrip;

(* DayOfWeek of 2026-05-09 — May 9, 2026 is a Saturday → 5 *)
PROCEDURE WeekdayOfMay9_2026* (): INTEGER;
    VAR d: Dates.Date;
BEGIN
    d.year := 2026; d.month := 5; d.day := 9;
    RETURN Dates.DayOfWeek(d)
END WeekdayOfMay9_2026;

(* DayOfWeek of 2024-01-01 (Monday) → 0 *)
PROCEDURE Weekday2024Jan1* (): INTEGER;
    VAR d: Dates.Date;
BEGIN
    d.year := 2024; d.month := 1; d.day := 1;
    RETURN Dates.DayOfWeek(d)
END Weekday2024Jan1;

(* Easter 2024 = 2024-03-31 → return month*100 + day = 331 *)
PROCEDURE Easter2024* (): INTEGER;
    VAR d: Dates.Date;
BEGIN
    Dates.GetEasterDate(2024, d);
    RETURN d.month * 100 + d.day
END Easter2024;

(* Easter 2025 = 2025-04-20 → return month*100 + day = 420 *)
PROCEDURE Easter2025* (): INTEGER;
    VAR d: Dates.Date;
BEGIN
    Dates.GetEasterDate(2025, d);
    RETURN d.month * 100 + d.day
END Easter2025;

(* ValidDate tests *)
PROCEDURE FebInLeapYearIsValid* (): INTEGER;
    VAR d: Dates.Date;
BEGIN
    d.year := 2024; d.month := 2; d.day := 29;
    IF Dates.ValidDate(d) THEN RETURN 1 ELSE RETURN 0 END
END FebInLeapYearIsValid;

PROCEDURE FebInNonLeapIsInvalid* (): INTEGER;
    VAR d: Dates.Date;
BEGIN
    d.year := 2023; d.month := 2; d.day := 29;
    IF Dates.ValidDate(d) THEN RETURN 1 ELSE RETURN 0 END
END FebInNonLeapIsInvalid;

PROCEDURE ValidTimeMidnight* (): INTEGER;
    VAR t: Dates.Time;
BEGIN
    t.hour := 0; t.minute := 0; t.second := 0;
    IF Dates.ValidTime(t) THEN RETURN 1 ELSE RETURN 0 END
END ValidTimeMidnight;

PROCEDURE ValidTimeOutOfRange* (): INTEGER;
    VAR t: Dates.Time;
BEGIN
    t.hour := 24; t.minute := 0; t.second := 0;
    IF Dates.ValidTime(t) THEN RETURN 1 ELSE RETURN 0 END
END ValidTimeOutOfRange;

END DatesArith.
