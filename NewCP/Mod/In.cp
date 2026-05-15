MODULE In;
(* BB-faithful In — the standard textual-input module that reads
   from the focus text's selection (or from the start of the text
   if there is no selection).

   The full BB module is six public procedures plus a `Done` flag
   that the caller polls between calls: any failed scan flips
   `Done` to FALSE and the rest of the chain becomes a sequence
   of no-ops, exactly the way BB's `In` works.

   Deferred — `In.Name` requires `TextMappers.ScanQualIdent`, which
   is not in our TextMappers slice yet (the full BB scanner state
   machine for qualified identifiers lands in a follow-up).  The
   procedure is omitted here rather than shipping a half-faithful
   version; callers that need name reading can hand-tokenize via
   the Scanner directly. *)

    IMPORT TextMappers, TextControllers;

    VAR
        Done*: BOOLEAN;
        s: TextMappers.Scanner;

    (** Open the focus text for reading.  If there is a selection,
        reads from the selection's start; otherwise from position 0.
        Sets `Done` accordingly. *)
    PROCEDURE Open*;
        VAR c: TextControllers.Controller; beg, end: INTEGER;
    BEGIN
        c := TextControllers.Focus();
        IF c # NIL THEN
            IF c.HasSelection() THEN
                c.GetSelection(beg, end)
            ELSE
                beg := 0
            END;
            s.ConnectTo(c.text);
            s.SetPos(beg);
            s.rider.ReadChar;
            Done := TRUE
        ELSE
            s.ConnectTo(NIL);
            Done := FALSE
        END
    END Open;

    (** Read one CHAR.  Stops on end-of-text (flips `Done` to FALSE). *)
    PROCEDURE Char* (OUT ch: CHAR);
    BEGIN
        IF Done THEN
            IF s.rider.eot THEN
                Done := FALSE
            ELSE
                ch := s.rider.char;
                s.rider.ReadChar
            END
        END
    END Char;

    (** Read one INTEGER. *)
    PROCEDURE Int* (OUT i: INTEGER);
    BEGIN
        IF Done THEN
            s.Scan;
            IF s.type = TextMappers.int THEN
                i := s.int
            ELSE
                Done := FALSE
            END
        END
    END Int;

    (** Read one LONGINT.  Accepts the int case too since BB's
        scanner promotes small literals to `int` (not `lint`). *)
    PROCEDURE LongInt* (OUT l: LONGINT);
    BEGIN
        IF Done THEN
            s.Scan;
            IF (s.type = TextMappers.lint) OR (s.type = TextMappers.int) THEN
                l := s.lint
            ELSE
                Done := FALSE
            END
        END
    END LongInt;

    (** Read one REAL.  Accepts an integer literal as REAL too. *)
    PROCEDURE Real* (OUT x: REAL);
    BEGIN
        IF Done THEN
            s.Scan;
            IF s.type = TextMappers.real THEN
                x := s.real
            ELSIF s.type = TextMappers.int THEN
                x := s.int
            ELSE
                Done := FALSE
            END
        END
    END Real;

    (** Read one string literal (quoted in the source).  Copies up
        to the destination's capacity; longer strings are truncated.
        BB writes `str := s.string$` here — the `$` operator trims at
        the first NUL.  Our IR layer doesn't lower `$`-assignment as
        a fixed-to-open-array copy yet, and a plain array `:=` would
        try to bulk-cast `[256]CHAR` to a `CHAR*` slot.  Do the
        char-by-char copy explicitly for now; this also lets us bound
        by the actual destination capacity. *)
    PROCEDURE String* (OUT str: ARRAY OF CHAR);
        VAR i, cap: INTEGER; ch: CHAR;
    BEGIN
        IF Done THEN
            s.Scan;
            IF s.type = TextMappers.string THEN
                cap := LEN(str) - 1;
                i := 0;
                WHILE (i < cap) & (s.string[i] # 0X) DO
                    ch := s.string[i];
                    str[i] := ch;
                    INC(i)
                END;
                str[i] := 0X
            ELSE
                Done := FALSE
            END
        END
    END String;

END In.
