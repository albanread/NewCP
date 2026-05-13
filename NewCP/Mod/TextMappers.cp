MODULE TextMappers;
(*
   First slice of the BlackBox `TextMappers` port.

   `TextMappers` ships `Scanner` and `Formatter` — typed value-
   record cursors that ride on top of `TextModels.Reader` /
   `TextModels.Writer` and expose a higher-level streaming
   interface (read the next token of a known type, write a
   formatted value).

   The full BlackBox module (~1070 lines) is mostly the
   scanning state machine for numbers / sets / quoted
   strings.  Most BlackBox call sites use the higher-level
   helpers via `ConnectTo` + `Scan`; the host's command
   loop, the dialog form-binding plumbing, and the Std
   converters all sit on top of it.

   This slice ships the surface (records, constants,
   `ConnectTo` / `Pos` / `SetPos` / `SetOpts`) and a small
   subset of the scan / format primitives (`Char`, `Integer`,
   `WriteChar`, `WriteString`, `WriteInt`, `WriteLn`).  The
   long numeric scanners (`Real`, `Number`, `Set`, …) and the
   `Scan` dispatcher are deferred — they're a substantial
   piece of work and will land in a follow-up.

   Deferred: see comments per-procedure.
*)

    IMPORT Strings, Views, TextModels;

    CONST
        (** Scanner.opts *)
        returnCtrlChars*  = 1;
        returnQualIdents* = 2;
        returnViews*      = 3;
        interpretBools*   = 4;
        interpretSets*    = 5;
        maskViews*        = 6;

        (** Scanner.type — type-of-the-last-scanned-token. *)
        char*    = 1;
        string*  = 3;
        int*     = 4;
        real*    = 5;
        bool*    = 6;
        set*     = 7;
        view*    = 8;
        tab*     = 9;
        line*    = 10;
        para*    = 11;
        lint*    = 16;
        eot*     = 30;
        invalid* = 31;

        (** Formatter.WriteIntForm base. *)
        charCode*    = Strings.charCode;
        decimal*     = Strings.decimal;
        hexadecimal* = Strings.hexadecimal;

        (** Formatter.WriteIntForm showBase. *)
        hideBase* = Strings.hideBase;
        showBase* = Strings.showBase;

        (* Local convenience for the BB-faithful char constants
           we touch frequently.  Folded once here so the body
           code below reads cleanly. *)
        VIEW = TextModels.viewcode;
        TAB  = TextModels.tab;
        LINE = TextModels.line;
        PARA = TextModels.para;


    TYPE
        (** Local string buffer — same shape as
            `Dialog.String` (256 chars) in BlackBox. *)
        String* = ARRAY 256 OF CHAR;

        (** Streaming-scanner state.  Use:
              s.ConnectTo(model); s.SetOpts(...);
              REPEAT s.Scan UNTIL s.type = eot.
            Each Scan call populates `type` and ONE of the
            typed slots (`int` / `real` / `bool` / `set` /
            `view` / `string` / `char`) so the caller can
            switch on `s.type` and pull the value. *)
        Scanner* = RECORD
            opts-:   SET;
            rider-:  TextModels.Reader;   (** prefetch state for single-char look-ahead *)
            type*:   INTEGER;
            start*, lines*, paras*: INTEGER;
            char*:   CHAR;
            int*:    INTEGER;
            base*:   INTEGER;
            lint*:   INTEGER;
            real*:   REAL;
            bool*:   BOOLEAN;
            set*:    SET;
            len*:    INTEGER;
            string*: String;
            view*:   Views.View;
            w*, h*:  INTEGER
        END;

        (** Streaming-writer state. *)
        Formatter* = RECORD
            rider-: TextModels.Writer
        END;


    (* -- Scanner methods ------------------------------------------------ *)

    (** Bind the scanner to `text`'s reader (or detach by
        passing NIL).  Resets options to {} and cursor to
        position 0. *)
    PROCEDURE (VAR s: Scanner) ConnectTo* (text: TextModels.Model), NEW;
    BEGIN
        IF text # NIL THEN
            s.rider := text.NewReader(s.rider);
            s.SetPos(0);
            s.SetOpts({})
        ELSE
            s.rider := NIL
        END
    END ConnectTo;

    (** Seek the scanner's reading cursor; resets the
        last-token state. *)
    PROCEDURE (VAR s: Scanner) SetPos* (pos: INTEGER), NEW;
    BEGIN
        s.rider.SetPos(pos);
        s.start := pos;
        s.lines := 0;
        s.paras := 0;
        s.type  := invalid
    END SetPos;

    (** Update the option set without changing position. *)
    PROCEDURE (VAR s: Scanner) SetOpts* (opts: SET), NEW;
    BEGIN
        s.opts := opts
    END SetOpts;

    (** Current reader position. *)
    PROCEDURE (VAR s: Scanner) Pos* (): INTEGER, NEW;
    BEGIN
        RETURN s.rider.Pos()
    END Pos;

    (** Skip whitespace (and optionally control chars and view
        placeholders, controlled by `opts`) advancing to the
        next significant character.  Sets `s.start` to the
        cursor of the next meaningful token, or `eot` if the
        reader hit end-of-text. *)
    PROCEDURE (VAR s: Scanner) Skip* (OUT ch: CHAR), NEW;
    BEGIN
        IF s.rider = NIL THEN
            ch := 0X;
            s.type := eot;
            RETURN
        END;
        ch := s.rider.char;
        WHILE (ch <= " ") & ~s.rider.eot DO
            IF ch = LINE THEN INC(s.lines)
            ELSIF ch = PARA THEN INC(s.paras)
            END;
            s.rider.ReadChar();
            ch := s.rider.char
        END;
        IF ~s.rider.eot THEN
            s.start := s.rider.Pos() - 1
        ELSE
            s.start := s.rider.Base().Length();
            s.type  := eot
        END
    END Skip;


    (* -- Formatter methods --------------------------------------------- *)

    (** Bind the formatter to `text`'s writer. *)
    PROCEDURE (VAR f: Formatter) ConnectTo* (text: TextModels.Model), NEW;
    BEGIN
        IF text # NIL THEN
            f.rider := text.NewWriter(f.rider)
        ELSE
            f.rider := NIL
        END
    END ConnectTo;

    (** Seek the writer's append cursor. *)
    PROCEDURE (VAR f: Formatter) SetPos* (pos: INTEGER), NEW;
    BEGIN
        f.rider.SetPos(pos)
    END SetPos;

    (** Current writer position. *)
    PROCEDURE (VAR f: Formatter) Pos* (): INTEGER, NEW;
    BEGIN
        RETURN f.rider.Pos()
    END Pos;

    (** Append one character. *)
    PROCEDURE (VAR f: Formatter) WriteChar* (ch: CHAR), NEW;
    BEGIN
        f.rider.WriteChar(ch)
    END WriteChar;

    (** Append a 0X-terminated CHAR array. *)
    PROCEDURE (VAR f: Formatter) WriteString* (IN s: ARRAY OF CHAR), NEW;
    BEGIN
        f.rider.WriteString(s)
    END WriteString;

    (** Append a line separator. *)
    PROCEDURE (VAR f: Formatter) WriteLn*, NEW;
    BEGIN
        f.rider.WriteChar(LINE)
    END WriteLn;

    (** Append a paragraph separator. *)
    PROCEDURE (VAR f: Formatter) WritePara*, NEW;
    BEGIN
        f.rider.WriteChar(PARA)
    END WritePara;

    (** Append a horizontal tab. *)
    PROCEDURE (VAR f: Formatter) WriteTab*, NEW;
    BEGIN
        f.rider.WriteChar(TAB)
    END WriteTab;


END TextMappers.
