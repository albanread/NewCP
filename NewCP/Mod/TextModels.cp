MODULE TextModels;
(*
   First slice of the BlackBox `TextModels` port.

    `StdModel` extends the typed `Stores.Store` abstract base
   and reads the wire-format BlackBox writes for every persisted
   text model:

     6 bytes    super-class version chain
                  (Stores.Store + isElem byte, Models.Model,
                   Containers.Model, TextModels.Model,
                   TextModels.StdModel)
     4 bytes    run-list length (i32 LE)
     N bytes    piece run list — sequence of (ano, len) entries
                  terminated by ano = 0xFF
     M bytes    contiguous text characters

   This slice decodes the run list's *summary* — how many text
   pieces, how many view placeholders, how the attribute pool
   grew, and the total character count contributed by text runs —
   then walks the character buffer enough to surface the first
   text piece's content for verification.  The full piece-list
   reconstruction (with attribute resolution and recursive view
   stores) lands once the `TextViews` slice ships.

   See newcp-odc/src/text_model.rs for the canonical wire-format
   specification.
*)

IMPORT Stores, Models, Containers, Properties, Ports, Fonts, Sequencers, Views;

CONST
    (** Special character codes used by the text stream — these
        live in the same CHAR space as ordinary letters but are
        treated specially by Reader / Writer / TextMappers /
        TextSetters. *)
    viewcode*   = 02X;    (** placeholder for an embedded view *)
    tab*        = 09X;    (** horizontal tabulator *)
    line*       = 0DX;    (** line separator *)
    para*       = 0EX;    (** paragraph separator *)
    zwspace*    = 8BX;    (** zero-width space — word boundary, no glyph *)
    digitspace* = 8FX;    (** digit-width space *)
    hyphen*     = 90X;    (** real hyphen *)
    nbhyphen*   = 91X;    (** non-breaking hyphen *)
    nbspace*    = 0A0X;   (** non-breaking space *)
    softhyphen* = 0ADX;   (** soft hyphen — break-here hint, no glyph *)

    (** Pref.opts — options of text-aware views. *)
    maskChar* = 0;
    hideable* = 1;

    (** Prop.known / valid / readOnly bitmask positions. *)
    offset* = 0;
    code*   = 1;

    (** InfoMsg.op *)
    store* = 0;

    (** UpdateMsg.op *)
    replace* = 0;
    insert*  = 1;
    delete*  = 2;

    minAttrVersion = 0; maxAttrVersion = 0;
    minDocVersion = 0; maxDocVersion = 0;

    SuperVersionBytes* = 6;
    MaxPiecesTracked*  = 256;
    TextBufferChars*   = 65536;
    AnoTerminator      = 255;

    (** Piece-kind tags recorded in `pieceKind`. *)
    PieceKindText1* = 1;     (* 1-byte / Latin-1 text run *)
    PieceKindText2* = 2;     (* 2-byte / UCS-2 text run *)
    PieceKindView*  = 3;     (* embedded-view placeholder *)

    (** Reasons Internalize might bail before fully decoding. *)
    OkComplete*           = 0;
    OkSuperVersionsTrunc* = 1;
    OkRunListLenTrunc*    = 2;
    OkRunListTrunc*       = 3;
    OkCharsTrunc*         = 4;
    OkUnsupportedNewAttr* = 5;
    OkUnsupportedView*    = 6;
    OkLongCharOddBytes*   = 7;
    OkTooManyPieces*      = 8;

    (** Maximum chars in a Doc's text buffer.  Small enough to
        keep test allocations cheap; will grow when a paging /
        rope-based StdModel lands. *)
    DocCapacity* = 4096;

TYPE
    (** BB-faithful Attributes — text-run formatting state
        (color, font, sub/superscript offset). *)
    AttributesDesc* = EXTENSIBLE RECORD (Stores.StoreDesc)
        init-:   BOOLEAN;
        color-:  Ports.Color;
        font-:   Fonts.Font;
        offset-: INTEGER
    END;
    Attributes* = POINTER TO AttributesDesc;

    (** Abstract Reader — single-char streaming cursor. *)
    ReaderDesc* = ABSTRACT RECORD
        eot*:    BOOLEAN;
        attr*:   Attributes;
        char*:   CHAR;
        view*:   Views.View;
        w*, h*:  INTEGER
    END;
    Reader* = POINTER TO ReaderDesc;

    (** Abstract Writer — streaming cursor that appends. *)
    WriterDesc* = ABSTRACT RECORD
        attr-: Attributes
    END;
    Writer* = POINTER TO WriterDesc;

    (** Abstract base for every text model. *)
    ModelDesc* = ABSTRACT RECORD (Containers.ModelDesc) END;
    Model* = POINTER TO ModelDesc;

    (** Abstract embedding context for a text model. *)
    ContextDesc* = ABSTRACT RECORD (Models.ContextDesc) END;
    Context* = POINTER TO ContextDesc;

    PropDesc* = RECORD (Properties.PropertyDesc)
        offset*: INTEGER;
        code*:   CHAR
    END;
    Prop* = POINTER TO PropDesc;

    Pref* = RECORD (Properties.Preference)
        opts*: SET;
        mask*: CHAR
    END;

    UpdateMsg* = RECORD (Models.UpdateMsg)
        op*:                 INTEGER;
        beg*, end*, delta*:  INTEGER
    END;

    InfoMsg* = RECORD (Models.Message)
        op*: INTEGER
    END;

    DirectoryDesc* = ABSTRACT RECORD
        attr-: Attributes
    END;
    Directory* = POINTER TO DirectoryDesc;

    StdDirectoryDesc* = RECORD (DirectoryDesc) END;
    StdDirectory*     = POINTER TO StdDirectoryDesc;


    (* -- Stage-1 wire-format-only StdModel (kept for the
            existing TextViews probes that decode .odc bodies
            through Stores.StoreDesc).  Future slices will
          flip this to extend ModelDesc and use the BB
          Reader/Writer.  Two coexist for now. *)
        StdModelDesc* = RECORD (Stores.StoreDesc)
        (** Concatenation of the 6 super-class version bytes. *)
        superVersions*: ARRAY 6 OF BYTE;
        (** Length of the encoded run list in bytes (the i32 the
            store body writes immediately after the version chain).
            -1 when the read failed before reaching this field. *)
        runListLen*:    INTEGER;

        (* --- Run-list summary ---------------------------------- *)
        textPieceCount*: INTEGER;     (** number of text-run pieces *)
        viewPieceCount*: INTEGER;     (** number of inline-view pieces *)
        attrPoolGrowth*: INTEGER;     (** count of NEW-attribute slots seen *)
        totalChars*:     INTEGER;     (** sum of char counts across text pieces *)

        (* --- Per-piece metadata, in run-list order.  Capped at
              MaxPiecesTracked; once exceeded, Internalize sets
              `result = OkTooManyPieces` and returns.  These are
              stored as parallel arrays rather than a record array
              to side-step pointer-to-array-of-record allocation
              for now. *)
        pieceCount*:    INTEGER;
        pieceKind*:     ARRAY MaxPiecesTracked OF INTEGER;
        pieceAttrIdx*:  ARRAY MaxPiecesTracked OF INTEGER;
        pieceCharLen*:  ARRAY MaxPiecesTracked OF INTEGER;
        pieceBufBytes*: ARRAY MaxPiecesTracked OF INTEGER;

        (* --- Concatenated text content of every 1-byte text-run
              piece (widened to CHAR), in run-list order.
              View placeholders and 2-byte text runs are skipped
              here; their existence is recorded in `pieceKind`.
              `textLen` is capped at TextBufferChars - 1 so the
              buffer is always 0X-terminated. *)
        text*:    ARRAY TextBufferChars OF CHAR;
        textLen*: INTEGER;

        (** OkComplete on success; one of the OkXxx codes above
            otherwise.  Distinct codes let tests assert exactly
            which decoder branch surrendered. *)
        result*:        INTEGER
    END;
    StdModel* = POINTER TO StdModelDesc;


    (* ─── Concrete Doc / DocReader / DocWriter ───────────────
       First concrete TextModels.Model in the port.  Carries a
       fixed-capacity in-memory CHAR buffer.  BB-faithful prefix
       of what StdModel will eventually unify with (same naming-
       gap as TextViews.Pane vs StdView). *)
    DocDesc* = RECORD (ModelDesc)
        (** In-memory text buffer.  `len` chars are valid;
            `buf[len]` is always 0X (acts as a sentinel for
            cursor-style traversal). *)
        buf-:  ARRAY DocCapacity OF CHAR;
        len-:  INTEGER;
        (** Per-character attribute.  `attrs[i]` applies to
            `buf[i]`.  NIL means "inherit the view's default
            attribute" (normal typeface/weight/style).  Set
            by `DocWriter.WriteChar` or `SetAttrRange`. *)
        attrs: ARRAY DocCapacity OF Attributes
    END;
    Doc* = POINTER TO DocDesc;

    DocReaderDesc* = RECORD (ReaderDesc)
        doc-: Doc;
        pos-: INTEGER       (** next char to read; 0 <= pos <= doc.len *)
    END;
    DocReader* = POINTER TO DocReaderDesc;

    DocWriterDesc* = RECORD (WriterDesc)
        doc-:  Doc;
        wpos-: INTEGER      (** append cursor; 0 <= wpos <= doc.len *)
        (** curAttr shadows the base `attr-` field so the per-char
            store in WriteChar picks it up without a local variable. *)
    END;
    DocWriter* = POINTER TO DocWriterDesc;

    (** Undo operation for a single InsertChar.  `Done` toggles between
        forward (insert) and backward (delete) on successive calls. *)
    InsertOpDesc = RECORD (Stores.OperationDesc)
        doc:  Doc;
        pos:  INTEGER;
        ch:   CHAR;
        attr: Attributes;   (** attribute for the inserted char *)
        done: BOOLEAN
    END;
    InsertOp = POINTER TO InsertOpDesc;

    (** Undo operation for a DeleteRange.  Saves up to DocCapacity chars
        for reversal.  `nSaved = 0` marks a deletion too large to undo. *)
    DeleteOpDesc = RECORD (Stores.OperationDesc)
        doc:     Doc;
        beg:     INTEGER;
        n:       INTEGER;
        nSaved:  INTEGER;
        buf:     ARRAY DocCapacity OF CHAR;
        attrBuf: ARRAY DocCapacity OF Attributes;  (** saved per-char attrs *)
        done:    BOOLEAN
    END;
    DeleteOp = POINTER TO DeleteOpDesc;

VAR
    dir-,    stdDir-: Directory;
    std:              StdDirectory;

PROCEDURE (a: AttributesDesc) Internalize* (VAR rd: Stores.Reader);
    VAR ver, col, offset, size, weight: INTEGER;
        style: SET;
        hasFont: BOOLEAN;
        typeface: Fonts.Typeface;
        i: INTEGER;
        b: BYTE;
BEGIN
    a.Internalize^(rd);
    rd.ReadVersion(minAttrVersion, maxAttrVersion, ver);
    IF rd.cancelled THEN RETURN END;
    rd.ReadLong(col);
    IF rd.eof THEN RETURN END;
    a.color := col;
    (* font *)
    rd.ReadBool(hasFont);
    IF rd.eof THEN RETURN END;
    IF hasFont THEN
        rd.ReadXInt(size);
        IF rd.eof THEN RETURN END;
        rd.ReadSet(style);
        IF rd.eof THEN RETURN END;
        rd.ReadInt(weight);
        IF rd.eof THEN RETURN END;
        (* typeface: sequence of bytes terminated by 0 *)
        i := 0;
        rd.ReadByte(b);
        WHILE (b # 0) & ~rd.eof & (i < LEN(typeface) - 1) DO
            typeface[i] := CHR(b);
            INC(i);
            rd.ReadByte(b)
        END;
        typeface[i] := 0X;
        IF rd.eof THEN RETURN END;
        (* obtain font via factory if available, else leave NIL *)
        IF Fonts.dir # NIL THEN
            a.font := Fonts.dir.This(typeface, size, style, weight)
        ELSE
            a.font := NIL
        END
    ELSE
        a.font := NIL
    END;
    rd.ReadInt(offset);
    IF rd.eof THEN RETURN END;
    a.offset := offset;
    a.init := TRUE
END Internalize;

PROCEDURE (a: AttributesDesc) Externalize* (VAR wr: Stores.Writer);
    VAR i: INTEGER;
BEGIN
    a.Externalize^(wr);
    wr.WriteVersion(maxAttrVersion);
    wr.WriteLong(a.color);
    wr.WriteBool(a.font # NIL);
    IF a.font # NIL THEN
        wr.WriteXInt(a.font.size);
        wr.WriteSet(a.font.style);
        wr.WriteInt(a.font.weight);
        (* typeface: write each char as a byte, terminate with 0 *)
        i := 0;
        WHILE (i < LEN(a.font.typeface)) & (a.font.typeface[i] # 0X) DO
            wr.WriteByte(SHORT(SHORT(ORD(a.font.typeface[i]))));
            INC(i)
        END;
        wr.WriteByte(0)   (* NUL terminator *)
    END;
    wr.WriteInt(a.offset)
END Externalize;


PROCEDURE (m: StdModelDesc) Domain* (): Stores.Domain;
BEGIN
    RETURN NIL
END Domain;

PROCEDURE (m: StdModelDesc) Internalize* (VAR rd: Stores.Reader);
    VAR i, j, len, w, h, ascii, attrIdx: INTEGER;
        attrPoolSize, totalBufBytes: INTEGER;
        b, ano, lo, hi: BYTE;
BEGIN
    m.runListLen := -1;
    m.textPieceCount := 0;
    m.viewPieceCount := 0;
    m.attrPoolGrowth := 0;
    m.totalChars := 0;
    m.pieceCount := 0;
    m.textLen := 0;
    m.result := OkSuperVersionsTrunc;

    (* Super-class version chain. *)
    i := 0;
    WHILE i < SuperVersionBytes DO
        rd.ReadByte(b);
        IF rd.eof THEN RETURN END;
        m.superVersions[i] := b;
        INC(i)
    END;

    (* Run-list length prefix — 4-byte i32 in the BB wire format. *)
    m.result := OkRunListLenTrunc;
    rd.ReadLong(m.runListLen);
    IF rd.eof THEN RETURN END;

    (* Run-list pieces.  Each piece is an (ano, len[, w, h]) triple;
       ano = 0xFF terminates.  Both NEW-attribute and embedded-view
       cases consume an inline child store via SkipInlineStore — the
       typed `Attributes` and view records arrive in later slices. *)
    m.result := OkRunListTrunc;
    attrPoolSize := 0;
    LOOP
        rd.ReadByte(ano);
        IF rd.eof THEN RETURN END;
        IF ano = AnoTerminator THEN EXIT END;

        IF ano = attrPoolSize THEN
            rd.SkipStore();
            IF rd.cancelled THEN
                m.result := OkUnsupportedNewAttr;
                RETURN
            END;
            attrIdx := attrPoolSize;
            INC(attrPoolSize);
            INC(m.attrPoolGrowth)
        ELSE
            attrIdx := ano       (* reference into the existing pool *)
        END;

        rd.ReadInt(len);
        IF rd.eof THEN RETURN END;

        IF m.pieceCount >= MaxPiecesTracked THEN
            m.result := OkTooManyPieces;
            RETURN
        END;
        m.pieceAttrIdx[m.pieceCount] := attrIdx;

        IF len > 0 THEN
            m.pieceKind[m.pieceCount]     := PieceKindText1;
            m.pieceCharLen[m.pieceCount]  := len;
            m.pieceBufBytes[m.pieceCount] := len;
            INC(m.textPieceCount);
            INC(m.totalChars, len)
        ELSIF len < 0 THEN
            IF (-len) MOD 2 # 0 THEN
                m.result := OkLongCharOddBytes;
                RETURN
            END;
            m.pieceKind[m.pieceCount]     := PieceKindText2;
            m.pieceCharLen[m.pieceCount]  := (-len) DIV 2;
            m.pieceBufBytes[m.pieceCount] := -len;
            INC(m.textPieceCount);
            INC(m.totalChars, (-len) DIV 2)
        ELSE
            (* len = 0 → embedded view: w (i32), h (i32), inline store. *)
            rd.ReadLong(w); IF rd.eof THEN RETURN END;
            rd.ReadLong(h); IF rd.eof THEN RETURN END;
            rd.SkipStore();
            IF rd.cancelled THEN
                m.result := OkUnsupportedView;
                RETURN
            END;
            m.pieceKind[m.pieceCount]     := PieceKindView;
            m.pieceCharLen[m.pieceCount]  := 0;
            m.pieceBufBytes[m.pieceCount] := 1;
            INC(m.viewPieceCount)
        END;
        INC(m.pieceCount)
    END;

    (* Character buffer.  Walk piece-by-piece in run-list order;
       1-byte text runs widen into `text` (capped at
       TextBufferChars - 1), 2-byte text runs and view placeholders
       are consumed but not retained — picking those up is the
       work of a later slice once the typed Attributes / view
       layers exist. *)
    m.result := OkCharsTrunc;
    totalBufBytes := 0;
    i := 0;
    WHILE i < m.pieceCount DO
        IF m.pieceKind[i] = PieceKindText1 THEN
            j := 0;
            WHILE j < m.pieceCharLen[i] DO
                rd.ReadByte(b);
                IF rd.eof THEN RETURN END;
                ascii := b;
                IF m.textLen < TextBufferChars - 1 THEN
                    m.text[m.textLen] := CHR(ascii);
                    INC(m.textLen)
                END;
                INC(j)
            END;
            INC(totalBufBytes, m.pieceCharLen[i])
        ELSIF m.pieceKind[i] = PieceKindText2 THEN
            (* 2-byte UCS-2 LE: read lo then hi byte per char. *)
            j := 0;
            WHILE j < m.pieceCharLen[i] DO
                rd.ReadByte(lo);
                IF rd.eof THEN RETURN END;
                rd.ReadByte(hi);
                IF rd.eof THEN RETURN END;
                ascii := lo + hi * 256;
                IF m.textLen < TextBufferChars - 1 THEN
                    m.text[m.textLen] := CHR(ascii);
                    INC(m.textLen)
                END;
                INC(j)
            END;
            INC(totalBufBytes, m.pieceBufBytes[i])
        ELSE
            (* Skip the bytes this piece reserved in the chars
               buffer (view placeholder: 1 byte). *)
            j := 0;
            WHILE j < m.pieceBufBytes[i] DO
                rd.ReadByte(b);
                IF rd.eof THEN RETURN END;
                INC(j)
            END;
            INC(totalBufBytes, m.pieceBufBytes[i])
        END;
        INC(i)
    END;
    m.text[m.textLen] := 0X;
    m.result := OkComplete
END Internalize;


(* -- BB-faithful Reader / Writer / Model ABSTRACT method
      declarations.  Concrete subclasses (a future
      `StdReader` / `StdWriter` / `StdModel` ladder bridging
      to the wire-format StdModelDesc above) will implement.
      Surfaced here so framework callers (TextMappers,
      TextRulers, TextControllers) can dispatch against the
      abstract base without each having to repeat the
      vtable shape. *)

(** Read one character from the text under the cursor; sets
    `eot := TRUE` and leaves `char := 0X` on end-of-text.  The
    reader's `attr` / `view` / `w` / `h` fields are also
    updated as a side-effect when the cursor lands on a
    style-change or an embedded view. *)
PROCEDURE (rd: Reader) ReadChar* (), NEW, ABSTRACT;

(** Seek the cursor to absolute character position `pos`.
    Implementations must clear `eot` if `pos` is within the
    text, set `eot := TRUE` and `char := 0X` otherwise. *)
PROCEDURE (rd: Reader) SetPos* (pos: INTEGER), NEW, ABSTRACT;

(** Current character position. *)
PROCEDURE (rd: Reader) Pos* (): INTEGER, NEW, ABSTRACT;

(** The text model this reader was opened on. *)
PROCEDURE (rd: Reader) Base* (): Model, NEW, ABSTRACT;


(** Write one character via this writer, advancing its
    cursor.  The writer's current `attr` is applied to the
    character's run. *)
PROCEDURE (wr: Writer) WriteChar* (ch: CHAR), NEW, ABSTRACT;

(** Append an entire `ARRAY OF CHAR` (terminated by 0X). *)
PROCEDURE (wr: Writer) WriteString* (IN s: ARRAY OF CHAR), NEW, ABSTRACT;

(** Seek the writer's append cursor. *)
PROCEDURE (wr: Writer) SetPos* (pos: INTEGER), NEW, ABSTRACT;

(** Current write position. *)
PROCEDURE (wr: Writer) Pos* (): INTEGER, NEW, ABSTRACT;

(** Update the per-run attribute state. *)
PROCEDURE (wr: Writer) SetAttr* (attr: Attributes), NEW, ABSTRACT;

(** The text model this writer was opened on. *)
PROCEDURE (wr: Writer) Base* (): Model, NEW, ABSTRACT;


(** Open a streaming Reader on this model.  `old` is an
    existing Reader to recycle (or NIL for a fresh one). *)
PROCEDURE (m: Model) NewReader* (old: Reader): Reader, NEW, ABSTRACT;

(** Open a streaming Writer on this model. *)
PROCEDURE (m: Model) NewWriter* (old: Writer): Writer, NEW, ABSTRACT;

(** Character length of the text. *)
PROCEDURE (m: Model) Length* (): INTEGER, NEW, ABSTRACT;


(* ─── Concrete Doc / DocReader / DocWriter ──────────────────────
   First concrete TextModels.Model in the port.  `Doc` carries a
   fixed-capacity in-memory CHAR buffer; `DocReader` walks it
   one char at a time; `DocWriter` appends to it.  This is a
    BB-faithful prefix of what `StdModel` will eventually be — the
    wire-format reader and the in-memory model fuse once the
    persisted `StdModelDesc` and the in-memory `ModelDesc` ladder
    collapse into one chain (same constraint that's keeping
    `StdView` and `Pane` separate in TextViews).

   Until then, `Doc` is the way framework code (and probes)
   instantiate a real text model:

     NEW(d);                    (* d.len = 0, empty *)
     wr := d.NewWriter(NIL);
     wr.WriteString("hello");   (* d.buf and d.len update *)
     rd := d.NewReader(NIL);
     rd.ReadChar();             (* rd.char = 'h', rd.pos = 1, etc *)
*)


(* -- DocReader concrete overrides ------------------------------ *)

PROCEDURE (rd: DocReaderDesc) ReadChar* ();
BEGIN
    (* BB semantics: a successful ReadChar returns a real char and
       leaves eot = FALSE; eot only trips when there's nothing
       left to read.  The caller's loop pattern is:
         rd.ReadChar();
         WHILE ~rd.eot DO use(rd.char); rd.ReadChar() END
       — which reads chars *until the next* ReadChar fails.  An
       overly-eager "trip eot on the call that consumed the last
       char" implementation drops the last char on the floor. *)
    IF rd.pos >= rd.doc.len THEN
        rd.eot  := TRUE;
        rd.char := 0X;
        rd.attr := NIL
    ELSE
        rd.char := rd.doc.buf[rd.pos];
        rd.attr := rd.doc.attrs[rd.pos];   (* per-char attribute, NIL = default *)
        rd.pos  := rd.pos + 1;
        rd.eot  := FALSE
    END
END ReadChar;

PROCEDURE (rd: DocReaderDesc) SetPos* (pos: INTEGER);
BEGIN
    (* BB-faithful: SetPos only moves the cursor; `char` is reset
       to 0X and `eot` to FALSE.  The caller must invoke ReadChar
       to load the char at the new position.  Pre-loading here
       causes a double-read against TextMappers-style scanners
       whose ReadChar advances `pos` past the just-loaded char,
       so the first iteration of any tokenization loop would
       see the same char twice.  See StdReader.SetPos in BB. *)
    IF pos < 0 THEN pos := 0 END;
    IF pos > rd.doc.len THEN pos := rd.doc.len END;
    rd.pos  := pos;
    rd.eot  := FALSE;
    rd.char := 0X
END SetPos;

PROCEDURE (rd: DocReaderDesc) Pos* (): INTEGER;
BEGIN
    RETURN rd.pos
END Pos;

PROCEDURE (rd: DocReaderDesc) Base* (): Model;
BEGIN
    RETURN rd.doc
END Base;


(* -- DocWriter concrete overrides ----------------------------- *)

PROCEDURE (wr: DocWriterDesc) WriteChar* (ch: CHAR);
    VAR pos: INTEGER; msg: UpdateMsg;
BEGIN
    IF wr.wpos < DocCapacity - 1 THEN
        pos := wr.wpos;
        wr.doc.buf[wr.wpos]   := ch;
        wr.doc.attrs[wr.wpos] := wr.attr;   (* apply current attr to char *)
        wr.wpos := wr.wpos + 1;
        IF wr.wpos > wr.doc.len THEN
            wr.doc.len := wr.wpos;
            wr.doc.buf[wr.doc.len] := 0X    (* keep sentinel *)
        END;
        msg.op    := insert;
        msg.beg   := pos;
        msg.end   := pos + 1;
        msg.delta := 1;
        Models.Broadcast(wr.doc, msg)
    END
END WriteChar;

PROCEDURE (wr: DocWriterDesc) WriteString* (IN s: ARRAY OF CHAR);
    VAR i: INTEGER;
BEGIN
    i := 0;
    WHILE (i < LEN(s)) & (s[i] # 0X) DO
        wr.WriteChar(s[i]);
        INC(i)
    END
END WriteString;

PROCEDURE (wr: DocWriterDesc) SetPos* (pos: INTEGER);
BEGIN
    IF pos < 0 THEN pos := 0 END;
    IF pos > wr.doc.len THEN pos := wr.doc.len END;
    wr.wpos := pos
END SetPos;

PROCEDURE (wr: DocWriterDesc) Pos* (): INTEGER;
BEGIN
    RETURN wr.wpos
END Pos;

PROCEDURE (wr: DocWriterDesc) SetAttr* (attr: Attributes);
BEGIN
    (* Store the attribute on the base-class slot; WriteChar reads
       it from `wr.attr` and stamps it onto each char written. *)
    wr.attr := attr
END SetAttr;

PROCEDURE (wr: DocWriterDesc) Base* (): Model;
BEGIN
    RETURN wr.doc
END Base;


(* -- Doc concrete overrides ----------------------------------- *)

PROCEDURE (m: DocDesc) NewReader* (old: Reader): Reader;
    VAR rd: DocReader;
BEGIN
    (* BB-faithful: NewReader returns a rider positioned at 0
       with no char yet loaded — the caller (typically via
       Scanner.ConnectTo → SetPos → ReadChar, or an explicit
       ReadChar in In.Open's pattern) must read the first char.
       Pre-loading `rd.char := buf[0]` here would double-read
       against the BB convention where ReadChar reads buf[pos]
       then increments pos. *)
    NEW(rd);
    rd.doc  := m(Doc);
    rd.pos  := 0;
    rd.char := 0X;
    rd.eot  := FALSE;
    RETURN rd
END NewReader;

PROCEDURE (m: DocDesc) NewWriter* (old: Writer): Writer;
    VAR wr: DocWriter;
BEGIN
    NEW(wr);
    wr.doc  := m(Doc);
    wr.wpos := m.len;       (* append by default *)
    IF m.seq = NIL THEN
        Models.SetSequencer(m, Sequencers.dir.New())
    END;
    RETURN wr
END NewWriter;

PROCEDURE (m: DocDesc) Length* (): INTEGER;
BEGIN
    RETURN m.len
END Length;

PROCEDURE (m: DocDesc) Externalize* (VAR wr: Stores.Writer);
    VAR i: INTEGER;
BEGIN
    m.Externalize^(wr);
    wr.WriteVersion(maxDocVersion);
    wr.WriteLong(m.len);
    i := 0;
    WHILE i < m.len DO
        wr.WriteByte(SHORT(SHORT(ORD(m.buf[i]))));
        INC(i)
    END
END Externalize;

PROCEDURE (m: DocDesc) Internalize* (VAR rd: Stores.Reader);
    VAR ver, len, i: INTEGER; b: BYTE;
BEGIN
    m.Internalize^(rd);
    rd.ReadVersion(minDocVersion, maxDocVersion, ver);
    IF rd.cancelled THEN RETURN END;
    rd.ReadLong(len);
    IF rd.eof THEN RETURN END;
    IF len < 0 THEN len := 0
    ELSIF len > DocCapacity - 1 THEN len := DocCapacity - 1
    END;
    i := 0;
    WHILE i < len DO
        rd.ReadByte(b);
        IF rd.eof THEN RETURN END;
        m.buf[i] := CHR(b);
        INC(i)
    END;
    m.len := len;
    m.buf[m.len] := 0X
END Internalize;

(* Containers.Model abstracts.  Doc has no embedded views yet. *)

PROCEDURE (m: DocDesc) GetEmbeddingLimits*
    (OUT minW, maxW, minH, maxH: INTEGER);
BEGIN
    minW := 0; maxW := 0;
    minH := 0; maxH := 0
END GetEmbeddingLimits;

PROCEDURE (m: DocDesc) ReplaceView* (old, new: Views.View);
BEGIN
    (* No embedded views — nothing to replace. *)
END ReplaceView;


(* ─── Direct buffer operations (bypass sequencer / undo) ──────────────
   Called from InsertOp.Do / DeleteOp.Do to apply the actual edit
   without recursively creating more undo ops. *)

PROCEDURE InsertDirect (m: Doc; pos: INTEGER; ch: CHAR; attr: Attributes);
    VAR i: INTEGER; msg: UpdateMsg;
BEGIN
    i := m.len;
    WHILE i > pos DO
        m.buf[i]   := m.buf[i - 1];
        m.attrs[i] := m.attrs[i - 1];   (* shift per-char attrs along with chars *)
        DEC(i)
    END;
    m.buf[pos]   := ch;
    m.attrs[pos] := attr;               (* stamp the attribute onto the char *)
    INC(m.len);
    m.buf[m.len] := 0X;
    msg.op := insert; msg.beg := pos; msg.end := pos + 1; msg.delta := 1;
    Models.Broadcast(m, msg)
END InsertDirect;

PROCEDURE DeleteDirect (m: Doc; beg, end: INTEGER);
    VAR i, n: INTEGER; msg: UpdateMsg;
BEGIN
    IF beg < 0 THEN beg := 0 END;
    IF end > m.len THEN end := m.len END;
    IF beg >= end THEN RETURN END;
    n := end - beg;
    i := beg;
    WHILE i + n <= m.len DO
        m.buf[i]   := m.buf[i + n];
        m.attrs[i] := m.attrs[i + n];   (* shift per-char attrs along with chars *)
        INC(i)
    END;
    m.len := m.len - n;
    m.buf[m.len] := 0X;
    msg.op := delete; msg.beg := beg; msg.end := beg; msg.delta := -n;
    Models.Broadcast(m, msg)
END DeleteDirect;


(* ─── InsertOp: undo/redo for InsertChar ─────────────────────── *)

PROCEDURE (op: InsertOp) Do*;
BEGIN
    IF ~op.done THEN
        InsertDirect(op.doc, op.pos, op.ch, op.attr)
    ELSE
        DeleteDirect(op.doc, op.pos, op.pos + 1)
    END;
    op.done := ~op.done
END Do;


(* ─── DeleteOp: undo/redo for DeleteRange ────────────────────── *)

PROCEDURE (op: DeleteOp) Do*;
    VAR i: INTEGER;
BEGIN
    IF ~op.done THEN
        (* Apply: delete [beg, beg+n). *)
        DeleteDirect(op.doc, op.beg, op.beg + op.n)
    ELSE
        (* Revert: re-insert saved chars at beg (in reverse to rebuild
           the original left-to-right sequence); restore saved attrs. *)
        IF op.nSaved > 0 THEN
            i := op.nSaved - 1;
            WHILE i >= 0 DO
                InsertDirect(op.doc, op.beg, op.buf[i], op.attrBuf[i]);
                DEC(i)
            END
        END
    END;
    op.done := ~op.done
END Do;


(** Insert one character at position `pos`, shifting everything from
    `pos` onwards one place to the right.  Creates an undo operation
    via the model's sequencer (or calls direct if none installed).
    pre: 0 <= pos <= m.len  AND  m.len < DocCapacity - 1 *)
PROCEDURE (m: DocDesc) InsertChar* (pos: INTEGER; ch: CHAR), NEW;
    VAR op: InsertOp;
BEGIN
    IF (pos < 0) OR (pos > m.len) THEN RETURN END;
    IF m.len >= DocCapacity - 1 THEN RETURN END;
    IF m.seq = NIL THEN
        Models.SetSequencer(m, Sequencers.dir.New())
    END;
    NEW(op);
    op.doc  := m(Doc); op.pos := pos; op.ch := ch;
    op.attr := NIL;   (* keyboard input inherits the view's default attr *)
    op.done := FALSE;
    Models.Do(m, "Insert", op)
END InsertChar;


(** Delete characters in [beg, end).  Creates an undo operation that
    saves the deleted chars for reversal.  If the range is too large
    to save (> DocCapacity), the deletion is applied but is not undoable.
    pre: 0 <= beg <= end <= m.len *)
PROCEDURE (m: DocDesc) DeleteRange* (beg, end: INTEGER), NEW;
    VAR op: DeleteOp; i, n: INTEGER;
BEGIN
    IF beg < 0 THEN beg := 0 END;
    IF end > m.len THEN end := m.len END;
    IF beg >= end THEN RETURN END;
    n := end - beg;
    IF m.seq = NIL THEN
        Models.SetSequencer(m, Sequencers.dir.New())
    END;
    NEW(op);
    op.doc := m(Doc); op.beg := beg; op.n := n; op.done := FALSE;
    (* Save chars and attrs for reversal. *)
    IF n <= DocCapacity THEN
        i := 0;
        WHILE i < n DO
            op.buf[i]     := m.buf[beg + i];
            op.attrBuf[i] := m.attrs[beg + i];
            INC(i)
        END;
        op.nSaved := n
    ELSE
        op.nSaved := 0   (* too large to save — deletion is not undoable *)
    END;
    Models.Do(m, "Delete", op)
END DeleteRange;


(** Apply `attr` to every character in [beg, end).
    NIL resets those characters to the view's default attribute
    (plain style).  Broadcasts a `replace` update so views repaint.
    This call is NOT undoable — use it for formatting commands that
    operate on selections. *)
PROCEDURE (m: DocDesc) SetAttrRange* (beg, end: INTEGER; attr: Attributes), NEW;
    VAR i: INTEGER; msg: UpdateMsg;
BEGIN
    IF beg < 0 THEN beg := 0 END;
    IF end > m.len THEN end := m.len END;
    IF beg >= end THEN RETURN END;
    i := beg;
    WHILE i < end DO m.attrs[i] := attr; INC(i) END;
    msg.op    := replace;
    msg.beg   := beg;
    msg.end   := end;
    msg.delta := 0;
    Models.Broadcast(m, msg)
END SetAttrRange;


(** Construct a fresh Attributes record with the given fields.
    Callers that need a styled attribute (Bold, Italic, …) use this
    factory rather than accessing the read-only `-` fields directly. *)
PROCEDURE NewAttributes* (color: Ports.Color; font: Fonts.Font;
                          offset: INTEGER): Attributes;
    VAR a: Attributes;
BEGIN
    NEW(a);
    a.init   := TRUE;
    a.color  := color;
    a.font   := font;
    a.offset := offset;
    RETURN a
END NewAttributes;


BEGIN
    NEW(std);
    NEW(std.attr);
    std.attr.init   := TRUE;
    std.attr.color  := 0;       (* Ports.black — packed 0 = opaque black *)
    std.attr.font   := NIL;     (* no font until Fonts.dir is installed *)
    std.attr.offset := 0;
    stdDir := std;
    dir    := std

END TextModels.
