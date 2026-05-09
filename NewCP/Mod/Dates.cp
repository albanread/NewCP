MODULE Dates;
(*
   Faithful port of BlackBox System/Mod/Dates.odc.

   Like Files, this is mostly an abstract interface module — `Hook`
   declares the platform-clock contract and the procedure-level
   helpers (`Day`, `DayToDate`, `ValidDate`, `ValidTime`, `DayOfWeek`,
   `GetEasterDate`) are all pure-value computations. `HostDates.cp`
   provides the concrete `StdHook` backed by HostDateSys; module
   consumers call `Dates.GetDate` / `Dates.GetTime` / etc. and the
   active hook handles the platform query.

   Differences from BlackBox:
   - The original imports Kernel for `Kernel.Hook` (the universal hook
     base type used by the IDE's plug-in registry). NewCP doesn't have
     a Kernel module; `Dates.Hook` is declared standalone here. The
     abstract-method surface is otherwise identical.

   Date encoding: a `Date` record is `(year, month, day)`; the `Day`
   procedure projects it to an absolute ordinal so date arithmetic
   reduces to integer subtraction. The Easter formula uses the M/N
   tables initialised in the module body.
*)

CONST
    (* Day-of-week codes returned by `DayOfWeek` *)
    monday*    = 0;
    tuesday*   = 1;
    wednesday* = 2;
    thursday*  = 3;
    friday*    = 4;
    saturday*  = 5;
    sunday*    = 6;

    (* Format codes accepted by `DateToString` *)
    short*            = 0;     (* "5/9/2026"        *)
    long*             = 1;     (* "May 9, 2026"     *)
    abbreviated*      = 2;     (* "May 9, 2026"     *)
    plainLong*        = 3;     (* "9 May 2026"      *)
    plainAbbreviated* = 4;     (* "9 May 2026"      *)

TYPE
    Date* = RECORD
        year*, month*, day*: INTEGER
    END;

    Time* = RECORD
        hour*, minute*, second*: INTEGER
    END;

    HookDesc* = ABSTRACT RECORD END;
    Hook*     = POINTER TO HookDesc;

VAR
    M, N: ARRAY 8 OF INTEGER;     (* Easter-formula tables *)
    hook-: Hook;


(* -- Abstract methods --------------------------------------------------- *)

PROCEDURE (h: HookDesc) GetTime*    (OUT d: Date; OUT t: Time), NEW, ABSTRACT;
PROCEDURE (h: HookDesc) GetUTCTime* (OUT d: Date; OUT t: Time), NEW, ABSTRACT;
PROCEDURE (h: HookDesc) GetUTCBias* (OUT bias: INTEGER), NEW, ABSTRACT;
PROCEDURE (h: HookDesc) DateToString* (IN d: Date; format: INTEGER;
                                       OUT str: ARRAY OF CHAR), NEW, ABSTRACT;
PROCEDURE (h: HookDesc) TimeToString* (IN t: Time;
                                       OUT str: ARRAY OF CHAR), NEW, ABSTRACT;


(* -- Hook registration -------------------------------------------------- *)

PROCEDURE SetHook* (h: Hook);
BEGIN
    hook := h
END SetHook;


(* -- Pure-value validators --------------------------------------------- *)

PROCEDURE ValidTime* (IN t: Time): BOOLEAN;
BEGIN
    RETURN
        (t.hour >= 0) & (t.hour <= 23)
        & (t.minute >= 0) & (t.minute <= 59)
        & (t.second >= 0) & (t.second <= 59)
END ValidTime;

PROCEDURE ValidDate* (IN d: Date): BOOLEAN;
    VAR y, m, d1: INTEGER;
BEGIN
    IF (d.year < 1) OR (d.year > 9999) OR (d.month < 1) OR (d.month > 12) OR (d.day < 1) THEN
        RETURN FALSE
    ELSE
        y := d.year; m := d.month;
        IF m = 2 THEN
            IF (y < 1583) & (y MOD 4 = 0)
            OR (y MOD 4 = 0) & ((y MOD 100 # 0) OR (y MOD 400 = 0)) THEN
                d1 := 29
            ELSE d1 := 28
            END
        ELSIF m IN {1, 3, 5, 7, 8, 10, 12} THEN d1 := 31
        ELSE d1 := 30
        END;
        (* Skip the 10 days dropped in October 1582 (Gregorian cutover). *)
        IF (y = 1582) & (m = 10) & (d.day > 4) & (d.day < 15) THEN RETURN FALSE END;
        RETURN d.day <= d1
    END
END ValidDate;


(* -- Day arithmetic ---------------------------------------------------- *)

(* Project (year, month, day) to an absolute ordinal day number.
   Implementation is the BlackBox formula verbatim — handles both the
   Julian (n <= 577737) and Gregorian (n > 577737) eras. *)
PROCEDURE Day* (IN d: Date): INTEGER;
    VAR y, m, n: INTEGER;
BEGIN
    y := d.year; m := d.month - 3;
    IF m < 0 THEN INC(m, 12); DEC(y) END;
    n := y * 1461 DIV 4 + (m * 153 + 2) DIV 5 + d.day - 306;
    IF n > 577737 THEN n := n - (y DIV 100 * 3 - 5) DIV 4 END;
    RETURN n
END Day;

(* Inverse of Day: ordinal n -> (year, month, day). *)
PROCEDURE DayToDate* (n: INTEGER; OUT d: Date);
    VAR c, y, m: INTEGER;
BEGIN
    IF n > 577737 THEN
        n := n * 4 + 1215; c := n DIV 146097; n := n MOD 146097 DIV 4
    ELSE
        n := n + 305; c := 0
    END;
    n := n * 4 + 3; y := n DIV 1461; n := n MOD 1461 DIV 4;
    n := n * 5 + 2; m := n DIV 153; n := n MOD 153 DIV 5;
    IF m > 9 THEN m := m - 12; INC(y) END;
    (* BlackBox uses SHORT() here because its INTEGER is 32-bit and
       Date fields are SHORTINT (16-bit). NewCP makes Date fields
       INTEGER (i64), so the assignment is direct. *)
    d.year  := 100 * c + y;
    d.month := m + 3;
    d.day   := n + 1
END DayToDate;


(* -- Hook-dispatched primitives --------------------------------------- *)

PROCEDURE GetDate* (OUT d: Date);
    VAR t: Time;
BEGIN
    ASSERT(hook # NIL, 100);
    hook.GetTime(d, t)
END GetDate;

PROCEDURE GetTime* (OUT t: Time);
    VAR d: Date;
BEGIN
    ASSERT(hook # NIL, 100);
    hook.GetTime(d, t)
END GetTime;

(* UTC = Coordinated Universal Time, also known as Greenwich Mean
   Time (GMT). UTC = local time + bias. *)

PROCEDURE GetUTCDate* (OUT d: Date);
    VAR t: Time;
BEGIN
    ASSERT(hook # NIL, 100);
    hook.GetUTCTime(d, t)
END GetUTCDate;

PROCEDURE GetUTCTime* (OUT t: Time);
    VAR d: Date;
BEGIN
    ASSERT(hook # NIL, 100);
    hook.GetUTCTime(d, t)
END GetUTCTime;

PROCEDURE GetUTCBias* (OUT bias: INTEGER);
BEGIN
    ASSERT(hook # NIL, 100);
    hook.GetUTCBias(bias)
END GetUTCBias;


(* -- Easter date (Gauss algorithm) ------------------------------------ *)

PROCEDURE GetEasterDate* (year: INTEGER; OUT d: Date);
    VAR k, m, n, a, b, c, d0, e, o: INTEGER; month, day: INTEGER;
BEGIN
    ASSERT((year >= 1583) & (year <= 2299), 20);
    k := year DIV 100 - 15;
    m := M[k]; n := N[k];
    a := year MOD 19; b := year MOD 4; c := year MOD 7;
    d0 := (19 * a + m) MOD 30; e := (2 * b + 4 * c + 6 * d0 + n) MOD 7;
    o := 21 + d0 + e; month := 3 + o DIV 31; day := o MOD 31 + 1;
    IF month = 4 THEN
        IF day = 26 THEN day := 19
        ELSIF (day = 25) & (d0 = 28) & (e = 6) & (a > 10) THEN day := 18
        END
    END;
    d.year := year;
    d.month := month;
    d.day := day
END GetEasterDate;


(* -- Day of week ------------------------------------------------------- *)

(* res = 0: Monday .. res = 6: Sunday  (BlackBox convention) *)
PROCEDURE DayOfWeek* (IN d: Date): INTEGER;
BEGIN
    RETURN (4 + Day(d)) MOD 7
END DayOfWeek;


(* -- String formatters (delegate to active hook) ----------------------- *)

PROCEDURE DateToString* (IN d: Date; format: INTEGER; OUT str: ARRAY OF CHAR);
BEGIN
    ASSERT(hook # NIL, 100);
    hook.DateToString(d, format, str)
END DateToString;

PROCEDURE TimeToString* (IN t: Time; OUT str: ARRAY OF CHAR);
BEGIN
    ASSERT(hook # NIL, 100);
    hook.TimeToString(t, str)
END TimeToString;


BEGIN
    (* Easter-table seed values (Gauss). *)
    M[0] := 22; N[0] := 2;
    M[1] := 22; N[1] := 2;
    M[2] := 23; N[2] := 3;
    M[3] := 23; N[3] := 4;
    M[4] := 24; N[4] := 5;
    M[5] := 24; N[5] := 5;
    M[6] := 24; N[6] := 6;
    M[7] := 25; N[7] := 0
END Dates.
