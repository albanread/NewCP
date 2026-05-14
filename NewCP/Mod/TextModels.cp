MODULE TextModels;
(*
   First slice of the BlackBox `TextModels` port.

   `StdModel` extends the typed `HostStores.Store` abstract base
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

IMPORT HostStores, Stores, Models, Containers, Properties, Ports, Fonts, Views;

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


    (* -- Stage-1 wire-format-only StdModel (kept for the
          existing TextViews probes that decode .odc bodies
          through HostStores.StoreDesc).  Future slices will
          flip this to extend ModelDesc and use the BB
          Reader/Writer.  Two coexist for now. *)
    StdModelDesc* = RECORD (HostStores.StoreDesc)
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
        buf-: ARRAY DocCapacity OF CHAR;
        len-: INTEGER
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
    END;
    DocWriter* = POINTER TO DocWriterDesc;

PROCEDURE (m: StdModelDesc) Internalize* (rd: HostStores.Reader);
    VAR i, j, len, w, h, ascii, attrIdx: INTEGER;
        attrPoolSize, totalBufBytes: INTEGER;
        b, ano: BYTE;
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

    (* Run-list length prefix. *)
    m.result := OkRunListLenTrunc;
    rd.ReadInt(m.runListLen);
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
            IF ~rd.SkipInlineStore() THEN
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
            rd.ReadInt(w); IF rd.eof THEN RETURN END;
            rd.ReadInt(h); IF rd.eof THEN RETURN END;
            IF ~rd.SkipInlineStore() THEN
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
        ELSE
            (* Skip the bytes this piece reserved in the chars
               buffer (2-byte text or view placeholder). *)
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
   wire-format reader and the in-memory model fuse once
   `HostStores.StoreDesc` and `Stores.StoreDesc` unify (same
   constraint that's keeping `StdView` and `Pane` separate in
   TextViews).

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
        rd.char := 0X
    ELSE
        rd.char := rd.doc.buf[rd.pos];
        rd.pos  := rd.pos + 1;
        rd.eot  := FALSE
    END
END ReadChar;

PROCEDURE (rd: DocReaderDesc) SetPos* (pos: INTEGER);
BEGIN
    IF pos < 0 THEN pos := 0 END;
    IF pos > rd.doc.len THEN pos := rd.doc.len END;
    rd.pos := pos;
    rd.eot := pos >= rd.doc.len;
    IF ~rd.eot THEN rd.char := rd.doc.buf[pos] ELSE rd.char := 0X END
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
BEGIN
    IF wr.wpos < DocCapacity - 1 THEN
        wr.doc.buf[wr.wpos] := ch;
        wr.wpos := wr.wpos + 1;
        IF wr.wpos > wr.doc.len THEN
            wr.doc.len := wr.wpos;
            wr.doc.buf[wr.doc.len] := 0X    (* keep sentinel *)
        END
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
    (* Attribute-aware writes are deferred — Doc carries plain
       chars without per-run formatting until the run-list slice
       ports.  The call is accepted as a no-op so callers using
       BB-style WriteString-after-SetAttr patterns don't trap. *)
END SetAttr;

PROCEDURE (wr: DocWriterDesc) Base* (): Model;
BEGIN
    RETURN wr.doc
END Base;


(* -- Doc concrete overrides ----------------------------------- *)

PROCEDURE (m: DocDesc) NewReader* (old: Reader): Reader;
    VAR rd: DocReader;
BEGIN
    NEW(rd);
    rd.doc  := m(Doc);
    rd.pos  := 0;
    rd.char := 0X;
    rd.eot  := m.len = 0;
    IF ~rd.eot THEN rd.char := m.buf[0] END;
    RETURN rd
END NewReader;

PROCEDURE (m: DocDesc) NewWriter* (old: Writer): Writer;
    VAR wr: DocWriter;
BEGIN
    NEW(wr);
    wr.doc  := m(Doc);
    wr.wpos := m.len;       (* append by default *)
    RETURN wr
END NewWriter;

PROCEDURE (m: DocDesc) Length* (): INTEGER;
BEGIN
    RETURN m.len
END Length;

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


END TextModels.
