MODULE ScanProbe;
(* Exercises TextMappers.Scanner.Scan against a real
   TextModels.Doc-backed buffer.  This covers:
     - skip-whitespace into the next token
     - positive integer scan
     - negative integer scan (sign + digits)
     - lone-sign-returns-as-char (no digit after)
     - double-quoted string scan
     - single-quoted string scan
     - single-char fallthrough for punctuation
     - EOT detection at the buffer's end

   Returns a packed integer encoding the verified counts:
     ints * 10000 + strings * 100 + chars
   Expected = 2 * 10000 + 2 * 100 + 2 = 20202 on success.
   Returns a negative error code for the first failed check. *)

    IMPORT TextModels, TextMappers;

    PROCEDURE Run* (): INTEGER;
        VAR doc: TextModels.Doc;
            f:   TextMappers.Formatter;
            s:   TextMappers.Scanner;
            ints, strings, chars: INTEGER;
    BEGIN
        NEW(doc);
        f.ConnectTo(doc);

        (* Build the source: `42 -17 "hello" 'world' + *`
           Tokens we expect to scan back:
             int(42), int(-17), string("hello"), string("world"),
             char("+"), char("*")
           Quote chars go through their hex codes (22X and 27X) so the
           one-char string literal heuristic doesn't promote them to
           Shortstring. *)
        f.WriteString("42 -17 ");
        f.WriteChar(22X); f.WriteString("hello"); f.WriteChar(22X);
        f.WriteChar(" ");
        f.WriteChar(27X); f.WriteString("world"); f.WriteChar(27X);
        f.WriteString(" + *");

        s.ConnectTo(doc);

        ints := 0; strings := 0; chars := 0;

        (* Token 1: int 42.  Diagnostics in this slice return the
           actual s.type so we can tell which path Scan took. *)
        s.Scan();
        IF s.type # TextMappers.int THEN RETURN -100 - s.type END;
        IF s.int # 42 THEN RETURN -1000 - s.int END;
        INC(ints);

        (* Token 2: int -17 *)
        s.Scan;
        IF s.type # TextMappers.int THEN RETURN -201 END;
        IF s.int # -17 THEN RETURN -202 END;
        INC(ints);

        (* Token 3: string "hello" *)
        s.Scan;
        IF s.type # TextMappers.string THEN RETURN -301 END;
        IF s.len # 5 THEN RETURN -302 END;
        IF (s.string[0] # "h") OR (s.string[4] # "o") OR (s.string[5] # 0X) THEN
            RETURN -303
        END;
        INC(strings);

        (* Token 4: string 'world' *)
        s.Scan;
        IF s.type # TextMappers.string THEN RETURN -401 END;
        IF s.len # 5 THEN RETURN -402 END;
        IF (s.string[0] # "w") OR (s.string[4] # "d") OR (s.string[5] # 0X) THEN
            RETURN -403
        END;
        INC(strings);

        (* Token 5: char "+" — lone sign, no digit follows *)
        s.Scan;
        IF s.type # TextMappers.char THEN RETURN -501 END;
        IF s.char # "+" THEN RETURN -502 END;
        INC(chars);

        (* Token 6: char "*" — punctuation fallthrough *)
        s.Scan;
        IF s.type # TextMappers.char THEN RETURN -601 END;
        IF s.char # "*" THEN RETURN -602 END;
        INC(chars);

        (* EOT: one more scan past the last token should set type=eot. *)
        s.Scan;
        IF s.type # TextMappers.eot THEN RETURN -700 END;

        RETURN ints * 10000 + strings * 100 + chars
    END Run;

END ScanProbe.
