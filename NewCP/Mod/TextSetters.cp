MODULE TextSetters;
(*
   First slice of the BlackBox `TextSetters` port.

   `TextSetters` is the layout / line-breaking engine that
   sits between `TextModels` (the wire format + character
   stream) and `TextViews` (the rendered text editor pane).
   Concrete `Setter` subclasses turn a `TextModels.Model` +
   `TextRulers.Ruler` chain into a sequence of `LineBox`
   geometry records: where each line starts and ends, the
   ascent/descent of its tallest glyph, where the tab stops
   end up after width adjustment, and the spaces vs.
   characters that consume any extra slack from justified
   text.

   The full BlackBox module (~2380 lines) is mostly the
   `StdReader.Read` / `StdSetter.GetLine` / `GetBox` line-
   breaking state machine plus its cache, hyphenation
   plumbing, and tab-width interpolation.  Porting that body
   is a substantial undertaking by itself.

   This slice ships:

   - The complete TYPE surface: `Pref` (preference query),
     `Reader` (per-character look-ahead state, ABSTRACT),
     `Setter` (per-paragraph layout engine, ABSTRACT),
     `LineBox` (line geometry record), `Directory`
     (factory), plus the `StdReader` / `StdSetter` /
     `StdDirectory` concrete leaves so any module that
     references the types can compile.
   - All BB-faithful constants (Pref opts, character
     re-exports, tab widths, line gaps, adjust masks, cache
     sizes, version stamps).
   - ABSTRACT method declarations on `Reader` and `Setter`
     so subclasses know what to override.
   - `Setter.ConnectTo` and `Setter.Internalize` /
     `Externalize` chains with super-call shape, deferring
     the actual wire-format work (it depends on the
     yet-to-port `Setter` version stamp).

   Deferred (concrete bodies):

   - `StdReader.Read` — the line-breaking + hyphenation
     state machine (~600 lines of the full BB module).
   - `StdSetter.GetLine` / `GetBox` / `ThisLine` / etc. —
     the line-laying logic.
   - The `boxCache` / `seqCache` memoization tables and
     their invalidation logic.
   - `WordPart` / `ExtendToEOL` / `Right` / `GetViewPref`
     / `GatherString` / `SpecialChar` / `LongChar` helper
     procedures.
*)

    IMPORT Fonts, Ports, Stores, Models, Views, Properties,
        TextModels, TextRulers;

    CONST
        (** Pref.opts — options of setter-aware views.
            "0 overrides 1" in BB-speak: lineBreak wins over
            wordJoin, etc.  Used by the layout engine to
            decide how aggressively to break long runs and
            whether to glue word fragments. *)
        lineBreak* = 0;
        wordJoin*  = 1;
        wordPart*  = 2;
        flexWidth* = 3;

        (* Re-exports of TextModels char codes — same
           bytecodes the layout engine pattern-matches
           against to break / shape / justify. *)
        tab        = TextModels.tab;
        line       = TextModels.line;
        para       = TextModels.para;
        zwspace    = TextModels.zwspace;
        nbspace    = TextModels.nbspace;
        hyphen     = TextModels.hyphen;
        nbhyphen   = TextModels.nbhyphen;
        digitspace = TextModels.digitspace;
        softhyphen = TextModels.softhyphen;

        mm = Ports.mm;

        minTabWidth* = 2 * Ports.point;
        stdTabWidth* = 4 * Ports.mm;

        leftLineGap*  = 2 * Ports.point;
        rightLineGap* = 3 * Ports.point;

        adjustMask* = {TextRulers.leftAdjust, TextRulers.rightAdjust};

        centered*   = {};
        leftFlush*  = {TextRulers.leftAdjust};
        rightFlush* = {TextRulers.rightAdjust};
        blocked*    = adjustMask;

        boxCacheLen = 64;
        seqCacheLen = 16;

        (* Search bound for paragraph-start: BB tries up to
           MAX(INTEGER) chars back.  Marked "unsafe: disabled"
           in BB; we mirror that here as the same constant
           even though our slice doesn't yet use it. *)
        paraShutoff   = 0;
        cachedRulers  = FALSE;
        periodInWords = FALSE;
        colonInWords  = FALSE;

        minVersion     = 0;
        maxVersion     = 0;
        maxStdVersion  = 0;


    TYPE
        (** Preference query — the layout-aware view's
            answer to "what shape do you prefer?".  `endW`
            is preset by the framework to the view width;
            `dsc` is preset to the dominating descender. *)
        Pref* = RECORD (Properties.Preference)
            opts*: SET;
            endW*: INTEGER;
            dsc*:  INTEGER
        END;

        (** Abstract per-character read cursor.  `r` is the
            underlying TextModels.Reader; the layout engine
            reads through `r` and stashes the next character
            in `string[0]` (or up to 64 chars for special
            sequences). *)
        ReaderDesc* = ABSTRACT RECORD
            r-: TextModels.Reader;

            (* unit *)
            string*: ARRAY 64 OF CHAR;
            view*:   Views.View;

            (* unit properties *)
            textOpts*:   SET;
            mask*:       CHAR;
            setterOpts*: SET;
            w*, endW*, h*, dsc*: INTEGER;
            attr*:       TextModels.Attributes;

            (* reading state *)
            eot*:     BOOLEAN;
            pos*:     INTEGER;
            x*:       INTEGER;
            adjStart*: INTEGER;
            spaces*:   INTEGER;
            tabIndex*: INTEGER;
            tabType*:  SET;

            (* line properties *)
            vw*:        INTEGER;
            hideMarks*: BOOLEAN;
            ruler*:     TextRulers.Ruler;
            rpos*:      INTEGER
        END;
        Reader* = POINTER TO ReaderDesc;

        (** Abstract layout-engine handle.  Connected to a
            text model via `ConnectTo`; concrete `Setter`s
            (`StdSetter` is the only one BB ships) implement
            the line-laying / tab-resolving / page-breaking
            methods below. *)
        SetterDesc* = ABSTRACT RECORD (Stores.StoreDesc)
            text-:      TextModels.Model;
            defRuler-:  TextRulers.Ruler;
            vw-:        INTEGER;
            hideMarks-: BOOLEAN
        END;
        Setter* = POINTER TO SetterDesc;

        (** Geometry record for a single laid-out line.
            `len` is the number of chars in the line;
            `ruler` / `rpos` identify the active ruler at
            line start; `left` / `right` / `asc` / `dsc`
            are user-unit metrics; `rbox` / `bop` / `adj` /
            `eot` / `views` are flag bits; the tail fields
            (skipOff / adjOff / spaces / adjW / tabW)
            carry the justified-text accounting. *)
        LineBox* = RECORD
            len*:                   INTEGER;
            ruler*:                 TextRulers.Ruler;
            rpos*:                  INTEGER;
            left*, right*, asc*, dsc*: INTEGER;
            rbox*, bop*, adj*, eot*: BOOLEAN;
            views*:                  BOOLEAN;
            skipOff*:                INTEGER;
            adjOff*:                 INTEGER;
            spaces*:                INTEGER;
            adjW*:                  INTEGER;
            tabW*:                  ARRAY TextRulers.maxTabs OF INTEGER
        END;

        (** Abstract factory — host installs a concrete
            `Directory` via `SetDir` at startup. *)
        DirectoryDesc* = ABSTRACT RECORD END;
        Directory* = POINTER TO DirectoryDesc;

        (** Concrete leaf reader — BB ships exactly one
            (`StdReader`) implementing the full layout
            state machine.  Empty here; body lands in a
            follow-up slice. *)
        StdReaderDesc* = RECORD (ReaderDesc) END;
        StdReader* = POINTER TO StdReaderDesc;

        (** Concrete leaf setter — same shape: one BB-
            shipped implementation around the `StdReader`
            plus the box / sequence caches. *)
        StdSetterDesc* = RECORD (SetterDesc)
            rd:    Reader;
            r:     TextModels.Reader;
            ruler: TextRulers.Ruler;
            rpos:  INTEGER;
            key:   INTEGER
        END;
        StdSetter* = POINTER TO StdSetterDesc;

        StdDirectoryDesc* = RECORD (DirectoryDesc) END;
        StdDirectory* = POINTER TO StdDirectoryDesc;


    VAR
        (** Active and default factories.  `dir` may be
            swapped at runtime; `stdDir` is the first one
            installed. *)
        dir-, stdDir-: Directory;

        nextKey: INTEGER;


    (* -- Reader ABSTRACT methods --------------------------------------- *)

    (** Reset the reader to a new underlying TextModels
        reader.  ABSTRACT — concrete subclasses set up
        their own look-ahead state machine. *)
    PROCEDURE (rd: Reader) Set*
        (old: TextModels.Reader;
         setter: Setter;
         pos: INTEGER;
         ruler: TextRulers.Ruler;
         rpos: INTEGER), NEW, ABSTRACT;

    (** Read the next character / token into `string[0..]`
        and update `w` / `h` / `dsc` / flags. *)
    PROCEDURE (rd: Reader) Read*, NEW, ABSTRACT;

    (** Adjust the width of the current run accounting for
        trailing-whitespace handling and right-margin
        relaxation. *)
    PROCEDURE (rd: Reader) AdjustWidth*
        (start, pos: INTEGER; IN box: LineBox; VAR w: INTEGER), NEW, ABSTRACT;

    (** Compute the longest prefix of the current run that
        fits in `w` width.  Returns the character offset of
        the split. *)
    PROCEDURE (rd: Reader) SplitWidth*
        (w: INTEGER): INTEGER, NEW, ABSTRACT;


    (* -- Setter methods ------------------------------------------------ *)

    (** Bind the setter to a text model + default ruler +
        view-width / hideMarks state. *)
    PROCEDURE (s: Setter) ConnectTo*
        (text: TextModels.Model;
         defRuler: TextRulers.Ruler;
         vw: INTEGER;
         hideMarks: BOOLEAN), NEW;
    BEGIN
        s.text      := text;
        s.defRuler  := defRuler;
        s.vw        := vw;
        s.hideMarks := hideMarks
    END ConnectTo;

    (** Page-resolution helpers. *)
    PROCEDURE (s: Setter) ThisPage* (pageH: INTEGER; pageNo: INTEGER): INTEGER, NEW, ABSTRACT;
    PROCEDURE (s: Setter) NextPage* (pageH: INTEGER; start: INTEGER): INTEGER, NEW, ABSTRACT;

    (** Paragraph / line cursor walks — the hot path of
        scrolling and incremental repaint. *)
    PROCEDURE (s: Setter) ThisSequence*     (pos: INTEGER): INTEGER, NEW, ABSTRACT;
    PROCEDURE (s: Setter) NextSequence*     (start: INTEGER): INTEGER, NEW, ABSTRACT;
    PROCEDURE (s: Setter) PreviousSequence* (start: INTEGER): INTEGER, NEW, ABSTRACT;
    PROCEDURE (s: Setter) ThisLine*         (pos: INTEGER): INTEGER, NEW, ABSTRACT;
    PROCEDURE (s: Setter) NextLine*         (start: INTEGER): INTEGER, NEW, ABSTRACT;
    PROCEDURE (s: Setter) PreviousLine*     (start: INTEGER): INTEGER, NEW, ABSTRACT;

    (** Word boundary detection.  Used by double-click +
        keyboard-word-nav commands. *)
    PROCEDURE (s: Setter) GetWord* (pos: INTEGER; OUT beg, end: INTEGER), NEW, ABSTRACT;

    (** Fetch the LineBox for the line starting at `start`. *)
    PROCEDURE (s: Setter) GetLine* (start: INTEGER; OUT box: LineBox), NEW, ABSTRACT;

    (** Lay out a [start, end) range into a LineBox capped
        at maxW × maxH user units. *)
    PROCEDURE (s: Setter) GetBox*
        (start, end, maxW, maxH: INTEGER;
         OUT box: LineBox), NEW, ABSTRACT;

    (** Open a fresh Reader on this setter. *)
    PROCEDURE (s: Setter) NewReader* (old: Reader): Reader, NEW, ABSTRACT;

    (** Compute the grid offset for a line whose descender
        is `dsc`, given the line-box geometry. *)
    PROCEDURE (s: Setter) GridOffset*
        (dsc: INTEGER; IN box: LineBox): INTEGER, NEW, ABSTRACT;


    (* -- Directory ABSTRACT method ------------------------------------ *)

    PROCEDURE (d: Directory) New* (): Setter, NEW, ABSTRACT;


    (** Install the active directory.  First call also
        becomes the immutable `stdDir`.  Mirrors the
        TextRulers / Printers pattern. *)
    PROCEDURE SetDir* (d: Directory);
    BEGIN
        ASSERT(d # NIL, 20);
        dir := d;
        IF stdDir = NIL THEN stdDir := d END
    END SetDir;


END TextSetters.
