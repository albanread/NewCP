**Dates**

DEFINITION Dates;

    CONST

        monday = 0; tuesday = 1; wednesday = 2; thursday = 3; friday = 4; saturday = 5; sunday = 6;

        short = 0; long = 1; abbreviated = 2; plainLong = 3; plainAbbreviated = 4;

    TYPE

        Date = RECORD

            year, month, day: INTEGER

        END;

        Time = RECORD

            hour, minute, second: INTEGER

        END;

    PROCEDURE ValidDate (IN d: Date): BOOLEAN;

    PROCEDURE ValidTime (IN t: Time): BOOLEAN;

    PROCEDURE GetDate (OUT d: Date);

    PROCEDURE GetTime (OUT t: Time);

    PROCEDURE GetEasterDate (year: INTEGER; OUT d: Date);

    PROCEDURE DayOfWeek (IN d: Date): INTEGER;

    PROCEDURE Day (IN d: Date): INTEGER;

    PROCEDURE DayToDate (n: INTEGER; OUT d: Date);

    PROCEDURE DateToString (IN d: Date; format: INTEGER; OUT str: ARRAY OF CHAR);

    PROCEDURE TimeToString (IN t: Time; OUT str: ARRAY OF CHAR);

END Dates.

Module *Dates* provides basic procedures to work with dates. It covers the Julian calendar up to 10/4/1582 and the Gregorian calendar starting at 10/15/1582. Module *Dates* can deal with dates from 1/1/1 up to 12/31/9999. The types *Date* and *Time* are known to the framework and can be displayed by suitable controls.

CONST **monday, tuesday, wednesday, thursday, friday, saturday, sunday**

Possible return value of procedure *DayOfWeek.*

CONST **short,  long, abbreviated, longPlain, abbreviatedPlain**

Possible value for parameter *format* of *DateToString* to specify the format.

TYPE **Date**

Date information.

**year**: INTEGER    0001 <= year <= 9999

**month**: INTEGER    1 <= month <= 12

**day**: INTEGER    1 <= day <= 31

TYPE **Time**

Time information.

**hour**: INTEGER    0 <= hour <= 23

**minute**: INTEGER    0 <= minute <= 59

**second**: INTEGER    0 <= second <= 59

PROCEDURE  **ValidDate** (IN d: Date): BOOLEAN

Test whether *d *is a valid date according to the Julian (before 1582) or Gregorian (after 1582) calendar. Dates between 10/5/1582 and 10/14/1582 did not exist and are not valid.

PROCEDURE  **ValidTime** (IN t: Time): BOOLEAN

Test whether time *t* is valid.

PROCEUDRE **GetDate **(OUT d: Date)

Get the current date.

PROCEDURE **GetTime** (OUT t: Time)

Get the current time.

PROCEDURE **GetEasterDate** (year: INTEGER; OUT d: Date)

Get the Easter date of *year.*

Pre

(year > 1582)  &  (year < 2300)    20

PROCEDURE **DayOfWeek** (IN d: Date): INTEGER

Return the weekday of date *d*.

Pre

ValidDate(d)    (not explicitly checked)

Post

result IN {monday .. sunday}

PROCEDURE  **Day** (d: Date): INTEGER;

For date *d*, return the number of days since 1/1/1. Day(1/1/1) = 1.

The difference between two dates in days can be computed with *Day(d2) - Day(d1).*

Pre

ValidDate(d)    (not explicitly checked)

Post

(result > 0)  &  (result < 3652062)

PROCEDURE  **DayToDate** (n: INTEGER; OUT d: Date);

Convert the number of days since 1/1/1 into a date.

*DayToDate(Day(d1), d2) => d1=d2*

Pre

(n > 0)  &  (n < 3652062)    (not explicitly checked)

Post

ValidDate(d) & Day(d) = n

PROCEDURE  **DateToString **(IN d: Date; format: INTEGER; OUT s: ARRAY OF CHAR);

Convert the date *d *into string *s. *The format of the conversion is specified through the operation system, usually depending on country and language.

**format    example**

short    01/02/92

abbreviated    Thu, Jan 2, 1992

long    Thursday, January 2, 1992

plainAbbreviated    Jan 2, 1992

plainLong    January 2, 1992

Pre

ValidDate(d)    (not explicitly checked)

format IN {short, abbreviated, long, plainAbbreviated, longAbbreviated}    20

PROCEDURE **TimeToString** (IN t: Time; OUT s: ARRAY OF CHAR);

Convert the time *t* into string *s*. The format of the conversion is specified through the operating system.

Pre

ValidTime(t)    (not explicitly checked)

