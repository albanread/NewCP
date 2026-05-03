**Out**

DEFINITION Out;

    PROCEDURE Open;

    PROCEDURE Char (ch: CHAR);

    PROCEDURE Ln;

    PROCEDURE Int (i: LONGINT; n: INTEGER);

    PROCEDURE Real (x: REAL; n: INTEGER);

    PROCEDURE String (str: ARRAY OF CHAR);

END Out.

This module is provided for compatibility with the book "Programming in Oberon" by Reiser/Wirth. It is useful when learning the language. It is not recommended for use in production programs.

PROCEDURE **Open**

Brings open log window to the top. If no log window is open, a new one is opened.

PROCEDURE **Char** (ch: CHAR)

Writes a character into the log.

PROCEDURE **Ln**

Writes a carriage return into the log.

PROCEDURE **Int** (i: LONGINT; n: INTEGER)

Writes an integer number into the log, with *n* digits. If *n* is too small (e.g., 0) to represent the number correctly, the necessary minimal number of digits is used.

PROCEDURE **Real** (x: REAL; n: INTEGER)

Writes a real number into the log, with *n* digits. If *n* is too small (e.g., 0) to represent the number correctly, the necessary minimal number of digits is used.

PROCEDURE **String** (str: ARRAY OF CHAR)

Writes a string into the log.

