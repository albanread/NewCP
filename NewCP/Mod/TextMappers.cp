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

    (** Skip whitespace advancing to the next significant character.
        When `returnCtrlChars` is in opts, TAB / LINE / PARA are NOT
        skipped — they are returned as token types by Scan.
        Sets `s.start` to the cursor of the next meaningful token,
        or leaves `s.type = eot` if the reader hit end-of-text. *)
    PROCEDURE (VAR s: Scanner) Skip* (OUT ch: CHAR), NEW;
    BEGIN
        IF s.rider = NIL THEN
            ch := 0X;
            s.type := eot;
            RETURN
        END;
        ch := s.rider.char;
        LOOP
            IF ~((ch <= " ") & ~s.rider.eot) THEN EXIT END;
            (* Stop at control chars when the caller wants them as tokens. *)
            IF returnCtrlChars IN s.opts THEN
                IF (ch = TAB) OR (ch = LINE) OR (ch = PARA) THEN EXIT END
            END;
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

    (** Tokenize a number literal at the current cursor.
        `neg` is TRUE if a preceding '-' was consumed.

        Handles three forms:
          decimal integer  — plain digit sequence (base 10)
          hex integer      — hex-digit sequence followed by 'H' (base 16)
          real             — digit sequence, '.', optional fraction,
                             optional 'E'/'e' exponent

        After return `s.rider.char` is positioned at the first
        character past the literal. *)
    PROCEDURE ScanNumber (VAR s: Scanner; neg: BOOLEAN);
        VAR ch: CHAR;
            buf: ARRAY 32 OF CHAR;
            i, v, hv, exp, esign: INTEGER;
            isHex: BOOLEAN;
            frac, scale, r: REAL;
    BEGIN
        ch := s.rider.char;
        i := 0;
        isHex := FALSE;

        (* Collect all hex-compatible leading digit chars. *)
        LOOP
            IF s.rider.eot OR (i >= 31) THEN EXIT END;
            IF (ch >= '0') & (ch <= '9') THEN
                buf[i] := ch; INC(i);
                s.rider.ReadChar(); ch := s.rider.char
            ELSIF ((ch >= 'A') & (ch <= 'F')) OR ((ch >= 'a') & (ch <= 'f')) THEN
                buf[i] := ch; INC(i);
                isHex := TRUE;
                s.rider.ReadChar(); ch := s.rider.char
            ELSE
                EXIT
            END
        END;
        buf[i] := 0X;

        IF ch = 'H' THEN
            (* Hex integer literal: e.g. 0FFH, 1AH *)
            s.rider.ReadChar();   (* consume 'H' *)
            v := 0; i := 0;
            WHILE buf[i] # 0X DO
                hv := ORD(buf[i]) - ORD('0');
                IF hv > 9 THEN
                    IF buf[i] >= 'a' THEN
                        hv := ORD(buf[i]) - ORD('a') + 10
                    ELSE
                        hv := ORD(buf[i]) - ORD('A') + 10
                    END
                END;
                v := v * 16 + hv; INC(i)
            END;
            IF neg THEN v := -v END;
            s.type := int; s.int := v; s.lint := v; s.base := 16
        ELSIF ~isHex & (ch = '.') THEN
            (* Real literal: integer part already in buf, now read fraction *)
            v := 0; i := 0;
            WHILE buf[i] # 0X DO
                v := v * 10 + (ORD(buf[i]) - ORD('0')); INC(i)
            END;
            frac := 0.0; scale := 1.0;
            s.rider.ReadChar(); ch := s.rider.char;   (* skip '.' *)
            WHILE (ch >= '0') & (ch <= '9') & ~s.rider.eot DO
                frac := frac * 10.0 + (ORD(ch) - ORD('0'));
                scale := scale * 10.0;
                s.rider.ReadChar(); ch := s.rider.char
            END;
            r := v + frac / scale;
            (* Optional exponent: E or e followed by optional sign and digits *)
            IF (ch = 'E') OR (ch = 'e') THEN
                exp := 0; esign := 1;
                s.rider.ReadChar(); ch := s.rider.char;
                IF ch = '-' THEN
                    esign := -1; s.rider.ReadChar(); ch := s.rider.char
                ELSIF ch = '+' THEN
                    s.rider.ReadChar(); ch := s.rider.char
                END;
                WHILE (ch >= '0') & (ch <= '9') & ~s.rider.eot DO
                    exp := exp * 10 + (ORD(ch) - ORD('0'));
                    s.rider.ReadChar(); ch := s.rider.char
                END;
                IF esign > 0 THEN
                    WHILE exp > 0 DO r := r * 10.0; DEC(exp) END
                ELSE
                    WHILE exp > 0 DO r := r / 10.0; DEC(exp) END
                END
            END;
            IF neg THEN r := -r END;
            s.type := real; s.real := r;
            s.int := SHORT(ENTIER(r)); s.lint := s.int
        ELSE
            (* Decimal integer — or invalid if hex letters appeared without 'H' *)
            IF isHex THEN
                (* Malformed: hex digits without 'H' suffix; return invalid *)
                s.type := invalid; s.int := 0; s.lint := 0; s.base := 10
            ELSE
                v := 0; i := 0;
                WHILE buf[i] # 0X DO
                    v := v * 10 + (ORD(buf[i]) - ORD('0')); INC(i)
                END;
                IF neg THEN v := -v END;
                s.type := int; s.int := v; s.lint := v; s.base := 10
            END
        END
    END ScanNumber;

    (** Scan an identifier at the current cursor position.
        `ch` holds the first character (already confirmed as IsIdentStart).
        Sets s.type = string (or bool when interpretBools is set).
        When returnQualIdents is in opts a Module.Name form is collected
        as a single "Module.Name" string token.
        After return s.rider.char is positioned past the identifier. *)
    PROCEDURE ScanIdent (VAR s: Scanner; ch: CHAR);
        VAR i, cap: INTEGER;
    BEGIN
        cap := LEN(s.string) - 1;
        i := 0;
        (* Collect the first identifier *)
        WHILE Strings.IsIdent(ch) & ~s.rider.eot DO
            IF i < cap THEN s.string[i] := ch; INC(i) END;
            s.rider.ReadChar();
            ch := s.rider.char
        END;
        (* Qualified ident extension: Mod.Name — consume '.' + second ident *)
        IF (returnQualIdents IN s.opts) & (ch = '.') & ~s.rider.eot THEN
            s.rider.ReadChar();
            ch := s.rider.char;
            IF Strings.IsIdentStart(ch) THEN
                IF i < cap THEN s.string[i] := '.'; INC(i) END;
                WHILE Strings.IsIdent(ch) & ~s.rider.eot DO
                    IF i < cap THEN s.string[i] := ch; INC(i) END;
                    s.rider.ReadChar();
                    ch := s.rider.char
                END
            END
            (* Note: if '.' was not followed by a letter we've consumed it;
               this is acceptable for well-formed input and degenerate cases. *)
        END;
        s.string[i] := 0X;
        s.len := i;
        s.char := 0X;
        (* Boolean keyword check (case-sensitive, matches BB convention) *)
        IF interpretBools IN s.opts THEN
            IF (s.string[0] = 'T') & (s.string[1] = 'R') & (s.string[2] = 'U')
               & (s.string[3] = 'E') & (s.string[4] = 0X) THEN
                s.type := bool; s.bool := TRUE; RETURN
            ELSIF (s.string[0] = 'F') & (s.string[1] = 'A') & (s.string[2] = 'L')
                  & (s.string[3] = 'S') & (s.string[4] = 'E') & (s.string[5] = 0X) THEN
                s.type := bool; s.bool := FALSE; RETURN
            END
        END;
        s.type := string
    END ScanIdent;

    (** Tokenize a quoted string at the current cursor.  `quote`
        is the opening quote char (already at s.rider.char).
        Sets s.type = string and copies the inner chars into
        s.string up to the closing quote or buffer capacity. *)
    PROCEDURE ScanQuotedString (VAR s: Scanner; quote: CHAR);
        VAR ch: CHAR; i, cap: INTEGER;
    BEGIN
        cap := LEN(s.string) - 1;     (* keep room for NUL *)
        s.rider.ReadChar();           (* skip opening quote *)
        ch := s.rider.char;
        i := 0;
        WHILE (ch # quote) & ~s.rider.eot & (i < cap) DO
            s.string[i] := ch;
            INC(i);
            s.rider.ReadChar();
            ch := s.rider.char
        END;
        s.string[i] := 0X;
        s.len  := i;
        s.type := string;
        IF ch = quote THEN
            s.rider.ReadChar()        (* skip closing quote *)
        END
    END ScanQuotedString;

    (** Parse a SET literal `{ n [.. m] [, ...] }` with the
        opening brace already at s.rider.char.
        Sets s.type = set, s.set = result on success;
        s.type = invalid on any syntax error.
        Called only when interpretSets is in s.opts. *)
    PROCEDURE ScanSet (VAR s: Scanner);
        VAR ch: CHAR; lo, hi: INTEGER; result: SET;
    BEGIN
        result := {};
        s.rider.ReadChar();              (* consume '{' *)
        ch := s.rider.char;
        (* Skip whitespace *)
        WHILE (ch <= " ") & ~s.rider.eot DO s.rider.ReadChar(); ch := s.rider.char END;
        IF ch = "}" THEN
            s.rider.ReadChar();          (* empty set: {} *)
            s.type := set; s.set := {}; RETURN
        END;
        LOOP
            WHILE (ch <= " ") & ~s.rider.eot DO s.rider.ReadChar(); ch := s.rider.char END;
            (* Lower bound must be a non-negative decimal integer *)
            IF (ch < "0") OR (ch > "9") THEN s.type := invalid; RETURN END;
            lo := 0;
            WHILE (ch >= "0") & (ch <= "9") & ~s.rider.eot DO
                lo := lo * 10 + (ORD(ch) - ORD("0"));
                s.rider.ReadChar(); ch := s.rider.char
            END;
            hi := lo;
            WHILE (ch <= " ") & ~s.rider.eot DO s.rider.ReadChar(); ch := s.rider.char END;
            (* Optional range: lo .. hi *)
            IF ch = "." THEN
                s.rider.ReadChar(); ch := s.rider.char;
                IF ch # "." THEN s.type := invalid; RETURN END;
                s.rider.ReadChar(); ch := s.rider.char;
                WHILE (ch <= " ") & ~s.rider.eot DO s.rider.ReadChar(); ch := s.rider.char END;
                IF (ch < "0") OR (ch > "9") THEN s.type := invalid; RETURN END;
                hi := 0;
                WHILE (ch >= "0") & (ch <= "9") & ~s.rider.eot DO
                    hi := hi * 10 + (ORD(ch) - ORD("0"));
                    s.rider.ReadChar(); ch := s.rider.char
                END;
                WHILE (ch <= " ") & ~s.rider.eot DO s.rider.ReadChar(); ch := s.rider.char END
            END;
            (* INCL range into result — guard bounds *)
            IF (lo >= 0) & (lo <= hi) & (hi <= MAX(SET)) THEN
                WHILE lo <= hi DO INCL(result, lo); INC(lo) END
            ELSE
                s.type := invalid; RETURN
            END;
            WHILE (ch <= " ") & ~s.rider.eot DO s.rider.ReadChar(); ch := s.rider.char END;
            IF ch = "," THEN
                s.rider.ReadChar(); ch := s.rider.char   (* next element *)
            ELSIF ch = "}" THEN
                s.rider.ReadChar();                       (* consume closing brace *)
                EXIT
            ELSE
                s.type := invalid; RETURN
            END
        END;
        s.type := set; s.set := result
    END ScanSet;

    (** Scan the next CP-style token starting at the current
        reading cursor.  Sets:
          s.type   ← one of {int, real, string, char, bool,
                             tab, line, para, eot, invalid}
          s.int    ← integer value (if type = int or hex int)
          s.real   ← real value (if type = real)
          s.bool   ← boolean value (if type = bool)
          s.string ← token text with NUL terminator
                     (type = string → identifier text;
                      type = char   → single char)
          s.char   ← single-char value (if type = char)
          s.base   ← 10 (decimal) or 16 (hex)

        Token forms recognised in this slice:
          identifier       — letter + (letter | digit | '_')*
          qualified ident  — Mod.Name (when returnQualIdents in opts)
          boolean keyword  — TRUE | FALSE (when interpretBools in opts)
          decimal integer  — digit sequence (optional leading sign)
          hex integer      — hex-digit sequence + 'H' suffix
          real             — digit '.' digit* ('E' ['+'|'-'] digit+)?
          quoted string    — "..." or '...'
          TAB / LINE / PARA pseudo-tokens (when returnCtrlChars in opts)
          char             — any other single character *)
    PROCEDURE (VAR s: Scanner) Scan*, NEW;
        VAR ch: CHAR; neg: BOOLEAN;
    BEGIN
        s.Skip(ch);
        IF s.type = eot THEN RETURN END;

        (* Control char pseudo-tokens when returnCtrlChars is set. *)
        IF ch = TAB THEN
            IF returnCtrlChars IN s.opts THEN
                s.type := tab; s.char := TAB;
                s.string[0] := TAB; s.string[1] := 0X; s.len := 1;
                s.rider.ReadChar; RETURN
            END
        ELSIF ch = LINE THEN
            IF returnCtrlChars IN s.opts THEN
                s.type := line; s.char := LINE;
                s.string[0] := LINE; s.string[1] := 0X; s.len := 1;
                s.rider.ReadChar; RETURN
            END
        ELSIF ch = PARA THEN
            IF returnCtrlChars IN s.opts THEN
                s.type := para; s.char := PARA;
                s.string[0] := PARA; s.string[1] := 0X; s.len := 1;
                s.rider.ReadChar; RETURN
            END
        END;

        (* Identifier or boolean keyword. *)
        IF Strings.IsIdentStart(ch) THEN
            ScanIdent(s, ch);
            RETURN
        END;

        (* Signed number or lone sign character. *)
        neg := FALSE;
        IF (ch = "-") OR (ch = "+") THEN
            IF ch = "-" THEN neg := TRUE END;
            s.rider.ReadChar();
            ch := s.rider.char;
            IF (ch < "0") OR (ch > "9") OR s.rider.eot THEN
                s.type := char;
                IF neg THEN s.char := "-" ELSE s.char := "+" END;
                s.string[0] := s.char;
                s.string[1] := 0X;
                s.len := 1;
                RETURN
            END
        END;

        IF (ch >= "0") & (ch <= "9") THEN
            ScanNumber(s, neg);
            RETURN
        END;

        IF (ch = '"') OR (ch = "'") THEN
            ScanQuotedString(s, ch);
            RETURN
        END;

        (* SET literal when interpretSets is active. *)
        IF (ch = "{") & (interpretSets IN s.opts) THEN
            ScanSet(s);
            RETURN
        END;

        (* Anything else: one-char token. *)
        s.type := char;
        s.char := ch;
        s.string[0] := ch;
        s.string[1] := 0X;
        s.len  := 1;
        s.rider.ReadChar()
    END Scan;


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

    (** Append an integer right-aligned in a field of at least `minW`
        characters, padded on the left with spaces.  If `minW <= 0`
        no padding is added.  Uses decimal (base-10) formatting. *)
    PROCEDURE (VAR f: Formatter) WriteInt* (x: INTEGER; minW: INTEGER), NEW;
        VAR buf: ARRAY 24 OF CHAR; neg: BOOLEAN; i, j, len: INTEGER; v: INTEGER;
    BEGIN
        (* Format the absolute value into buf (reversed). *)
        neg := x < 0;
        IF neg THEN v := -x ELSE v := x END;
        len := 0;
        IF v = 0 THEN
            buf[0] := '0'; len := 1
        ELSE
            WHILE v > 0 DO
                buf[len] := CHR(v MOD 10 + ORD('0')); INC(len); v := v DIV 10
            END;
            IF neg THEN buf[len] := '-'; INC(len) END
        END;
        (* Left-pad with spaces *)
        j := len;
        WHILE j < minW DO f.rider.WriteChar(' '); INC(j) END;
        (* Write digits (reverse order) *)
        i := len;
        WHILE i > 0 DO DEC(i); f.rider.WriteChar(buf[i]) END
    END WriteInt;

    (** Append a formatted integer.  `form` selects the base:
          `decimal`     (= 10)  — decimal notation
          `hexadecimal` (= -2)  — hexadecimal, 'H' appended if showBase
          `charCode`    (= -1)  — decimal, 'X' appended if showBase
        `minW` / `fillCh` control minimum field width and fill character.
        `showBase` appends a base indicator ('H' for hex, 'X' for char). *)
    PROCEDURE (VAR f: Formatter) WriteIntForm*
        (x: INTEGER; form, minW: INTEGER; fillCh: CHAR; showBase: BOOLEAN), NEW;
        VAR s: ARRAY 48 OF CHAR; i: INTEGER;
    BEGIN
        Strings.IntToStringForm(x, form, minW, fillCh, showBase, s);
        i := 0;
        WHILE s[i] # 0X DO f.rider.WriteChar(s[i]); INC(i) END
    END WriteIntForm;

    (** Append a boolean value as "TRUE" or "FALSE". *)
    PROCEDURE (VAR f: Formatter) WriteBool* (x: BOOLEAN), NEW;
    BEGIN
        IF x THEN f.rider.WriteString("TRUE")
        ELSE       f.rider.WriteString("FALSE")
        END
    END WriteBool;

    (** Append a SET in BB notation: {0, 3..5, 7}. *)
    PROCEDURE (VAR f: Formatter) WriteSet* (x: SET), NEW;
        VAR s: ARRAY 128 OF CHAR; i: INTEGER;
    BEGIN
        Strings.SetToString(x, s);
        i := 0;
        WHILE s[i] # 0X DO f.rider.WriteChar(s[i]); INC(i) END
    END WriteSet;

    (** Append a real number in BlackBox default format:
        16 digits of precision, auto-exponent, digitspace (CHR(08FH))
        fill.  `minW` is the minimum total field width. *)
    PROCEDURE (VAR f: Formatter) WriteReal* (x: REAL; minW: INTEGER), NEW;
        VAR s: ARRAY 80 OF CHAR; i: INTEGER;
    BEGIN
        Strings.RealToStringForm(x, 16, minW, 0, CHR(08FH), s);
        i := 0;
        WHILE s[i] # 0X DO f.rider.WriteChar(s[i]); INC(i) END
    END WriteReal;

    (** Append a real number with full format control.
        Maps directly to Strings.RealToStringForm.
          precision  significant digits (1..16)
          minW       minimum field width (>= 0)
          expW       0 = auto; <0 = fixed with |expW| frac digits;
                     >0 = scientific with expW exponent digits
          fillCh     padding character *)
    PROCEDURE (VAR f: Formatter) WriteRealForm*
        (x: REAL; precision, minW, expW: INTEGER; fillCh: CHAR), NEW;
        VAR s: ARRAY 80 OF CHAR; i: INTEGER;
    BEGIN
        Strings.RealToStringForm(x, precision, minW, expW, fillCh, s);
        i := 0;
        WHILE s[i] # 0X DO f.rider.WriteChar(s[i]); INC(i) END
    END WriteRealForm;


END TextMappers.
