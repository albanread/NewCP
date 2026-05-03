**In**

DEFINITION In;

    VAR Done-: BOOLEAN;

    PROCEDURE Open;

    PROCEDURE Char (OUT ch: CHAR);

    PROCEDURE Int (OUT i: INTEGER);

    PROCEDURE LongInt (OUT l: LONGINT);

    PROCEDURE Real (OUT x: REAL);

    PROCEDURE Name (OUT name: ARRAY OF CHAR);

    PROCEDURE String (OUT str: ARRAY OF CHAR);

END In.

This module is provided for compatibility with the book "Programming in Oberon" by Reiser/Wirth. It is useful when learning the language. It is not recommended for use in production programs.

VAR **Done**

This variable indicates whether the most recent input operation has succeeded. It is set to *TRUE* by a successful *Open*, and set to *FALSE* by the first unsuccessful input operation. Once set to *FALSE*, it remains *FALSE* until the next *Open*.

PROCEDURE **Open**

This procedure opens the input stream. In BlackBox, the input stream is opened onto the target focus' text. If there is no target focus, or if it doesn't contain text, *Done* is set to FALSE. If there is a target focus containing text, the input stream is connected to the beginning of the text if there is no selection, otherwise to the beginning of the selection.

Post

Done

    input stream was opened successfully

~Done

    input stream couldn't be opened

PROCEDURE **Char** (OUT ch: CHAR)

If *Done* holds, this procedure attempts to read a character, otherwise it does nothing.

Post

Done

    ch has been read

~Done

    no character could be read

PROCEDURE **Int** (OUT i: INTEGER)

If *Done* holds, this procedure attempts to read an integer, otherwise it does nothing.

Post

Done

    i has been read

~Done

    no integer could be read

PROCEDURE **LongInt** (OUT l: LONGINT)

If *Done* holds, this procedure attempts to read a long integer, otherwise it does nothing.

Post

Done

    l has been read

~Done

    no long integer could be read

PROCEDURE **Real** (OUT x: REAL)

If *Done* holds, this procedure attempts to read a real number, otherwise it does nothing.

Post

Done

    x has been read

~Done

    no real number could be read

PROCEDURE **Name** (OUT name: ARRAY OF CHAR)

If *Done* holds, this procedure attempts to read a name, otherwise it does nothing. A name is a sequence of legal Component Pascal identifiers concatenated by periods, e.g., "Dialog.Beep".

Post

Done

    x has been read

~Done

    no name could be read

PROCEDURE **String** (OUT str: ARRAY OF CHAR)

If *Done* holds, this procedure attempts to read a string, otherwise it does nothing. A string is a sequence of characters delimited by white space (i.e., blanks, carriage returns, tabulators) or by double quotes (").

Post

Done

    str has been read

~Done

    no string could be read

