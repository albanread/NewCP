**TextSetters**

DEFINITION TextSetters;

    IMPORT Stores, Views, Properties, TextModels, TextRulers;

    CONST lineBreak = 0; wordJoin = 1; wordPart = 2; flexWidth = 3;

    TYPE

        Pref = RECORD (Properties.Preference)

            opts: SET;

            endW: INTEGER;

            dsc: INTEGER

        END;

        Reader = POINTER TO ABSTRACT RECORD

            r-: TextModels.Reader;

            sString: ARRAY 64 OF SHORTCHAR;

            string: ARRAY 64 OF CHAR;

            view: Views.View;

            textOpts: SET;

            mask: CHAR;

            setterOpts: SET;

            w, endW, h, dsc: INTEGER;

            attr: TextModels.Attributes;

            eot: BOOLEAN;

            pos: INTEGER;

            x: INTEGER;

            adjStart: INTEGER;

            spaces: INTEGER;

            tabIndex: INTEGER;

            tabType: SET;

            vw: INTEGER;

            hideMarks: BOOLEAN;

            ruler: TextRulers.Ruler;

            rpos: INTEGER;

            (rd: Reader) Set (old: TextModels.Reader; text: TextModels.Model; x, pos: INTEGER;

                                        ruler: TextRulers.Ruler; rpos, vw: INTEGER; hideMarks: BOOLEAN),

                                        NEW, EXTENSIBLE;

            (rd: Reader) Read, NEW, EXTENSIBLE;

            (rd: Reader) AdjustWidth (start, pos: INTEGER; IN box: LineBox; VAR w: INTEGER),

                                        NEW, ABSTRACT;

            (rd: Reader) SplitWidth (w: INTEGER): INTEGER, NEW, ABSTRACT

        END;

        Setter = POINTER TO ABSTRACT RECORD (Stores.Store)

            text-: TextModels.Model;

            defRuler-: TextRulers.Ruler;

            vw-: INTEGER;

            hideMarks-: BOOLEAN;

            (s: Setter) ConnectTo (text: TextModels.Model; defRuler: TextRulers.Ruler;

                                        vw: INTEGER; hideMarks: BOOLEAN), NEW, EXTENSIBLE;

            (s: Setter) ThisPage (pageH: INTEGER; pageNo: INTEGER): INTEGER, NEW, ABSTRACT;

            (s: Setter) NextPage (pageH: INTEGER; start: INTEGER): INTEGER, NEW, ABSTRACT;

            (s: Setter) ThisSequence (pos: INTEGER): INTEGER, NEW, ABSTRACT;

            (s: Setter) NextSequence (start: INTEGER): INTEGER, NEW, ABSTRACT;

            (s: Setter) PreviousSequence (start: INTEGER): INTEGER, NEW, ABSTRACT;

            (s: Setter) GetWord (pos: INTEGER; OUT beg, end: INTEGER), NEW, ABSTRACT;

            (s: Setter) GetLine (start: INTEGER; OUT box: LineBox), NEW, ABSTRACT;

            (s: Setter) GetBox (start, end, maxW, maxH: INTEGER; OUT w, h: INTEGER), NEW, ABSTRACT;

            (s: Setter) NewReader (old: Reader): Reader, NEW, ABSTRACT;

            (s: Setter) GridOffset (dsc: INTEGER; IN box: LineBox): INTEGER, NEW, ABSTRACT

        END;

        LineBox = RECORD

            len: INTEGER;

            ruler: TextRulers.Ruler;

            rpos: INTEGER;

            left, right, asc, dsc: INTEGER;

            rbox, bop, adj, eot, views: BOOLEAN;

            skipOff: INTEGER;

            adjOff: INTEGER;

            spaces: INTEGER;

            adjW: INTEGER;

            tabW: ARRAY TextRulers.maxTabs OF INTEGER

        END;

        Directory = POINTER TO RECORD

            (d: Directory) New (): Setter, NEW, ABSTRACT

        END;

    VAR dir-, stdDir-: Directory;

    PROCEDURE SetDir (d: Directory);

END TextSetters.

TextSetters set texts (one-dimensional streams of characters and embedded views) into two-dimensional columns or pages. Special text-aware views embedded in the set text, so-called rulers, are interpreted as requests for special setting formats.

CONST **lineBreak**, **wordJoin**, **wordPart**, **flexWidth**

Possible values of *Pref.opts* indicating preferences of setter-aware views embedded in a text. The inclusion of *lineBreak* overrides any possible inclusion of *wordJoin*.

If *lineBreak* is included, the setter is requested to break the line just after the view.

If *wordJoin* is included, the view requests that it should not be used as a position to break the words to the left and to the right of the view at the end of a line.

If *wordPart* is included, the view is treated as part of the word that it is embedded into (which effects the range selected by a "select word" operation).

If *flexWidth* is included, the view is treated the same way as ordinary blanks, i.e. its width is adjusted when setting a line in fully adjusted mode.

TYPE **Pref (Properties.Preference)**

Possible preferences of setter-aware views embedded into a text.

**opts**: SET

Setting options drawn from {*lineBreak*, *wordJoin*, *wordPart*, *flexWidth*}, as defined above.

**endW**: INTEGER    [units]    *preset to width of view*

If view happens to be placed at the end of a line, i.e., a line break is immediately following, the view may request a width different from its usual width. (For example, a soft-hyphen has a positive end width, while it has a zero width; a blank has a positive width, but a zero end width.)

**dsc**: INTEGER    [units]    *preset to dominating line descender*

A view may request a special descender value, thereby adjusting its placement relative to the baseline.

TYPE **Reader**

ABSTRACT

A reader to read through lines returned by a setter. The reader is a conceptual extension of a *TextModels.Reader*, but instead of just returning the elements of a stream, it also sets the elements on a line and returns placement coordinates relative to the lines baseline origin.

**r**-: TextModels.Reader

The text reader used to connect the reader to a text. The text reader state is used as a one element look-ahead state for the reader.

**sString**: ARRAY 64 OF SHORTCHAR

If sString # "", then the element read most recently was a short character or a string of short characters. Single short characters are returned in *sString*[0]. If *view* # NIL, then the element read most recently was a view masked as a short character.

**string**: ARRAY 64 OF CHAR

If string # "", then the element read most recently was a character or a string of characters. Single characters are returned in *string*[0]. If *view* # NIL, then the element read most recently was a view masked as a character.

**view**: Views.View

The element read most recently read was an embedded view.

**textOpts**: SET

**mask**: CHAR

**setterOpts**: SET

**w**, **endW**, **h**, **dsc**: INTEGER    [units]

**attr**: TextModels.Attributes

Properties of the element read most recently: Its text options (if text-aware, else preset default); its mask character (if *TextModels.maskChar* IN *textOpts*); its setter options (if setter-aware, else preset default); its width, end width (if setter-aware, else preset default), height, and descender (if setter-aware, else preset default); its text attributes.

**eot**: BOOLEAN

Set if the last trial to read hit the end of text.

**pos**: INTEGER

Position of the reader in the text (one past the element read most recently).

**x**: INTEGER    [units]

Horizontal position of the reader in the line. *To be advanced by client of reader*. For non-adjusted setting, increment by *w*, else utilize *AdjustWidth* below.

**adjStart**: INTEGER    [units]

The first position (inclusive) to begin space adjustment at. This is used to suppress space adjustment in all but the last section of several *tab*-separated sections of a line.

**spaces**: INTEGER    [units]

Number of spaces encountered by reader so far. (Reset to 0 when *adjStart* gets reset on reading a *tab*.)

**tabIndex**: INTEGER

Index of the most recently processed tab stop (initially -1).

**tabType**: SET

Type of the most recently processed tab stop (initially {}).

**vw**: INTEGER    [units]

Width to set text against.

**hideMarks**: BOOLEAN

Hideable marks are requested to be hidden in the correct line. If the reader encounters an embedded view that by its text preferences is *hideable* and *hideMarks* is set, then the view is reduced to zero width and height.

**ruler**: TextRulers.Ruler

Ruler dominating the setting of the current line.

**rpos**: INTEGER

Position of the dominating ruler in the text. (*rpos* = -1 if the ruler is not part of the text and there is no ruler in the text that dominates the current line. Typically, this is used to apply a default ruler to the beginning of a text that has no ruler at position 0.)

PROCEDURE (rd: Reader) **Set** (old: TextModels.Reader;

                                                        text: TextModels.Model; x, pos: INTEGER;

                                                        ruler: TextRulers.Ruler; rpos, vw: INTEGER;

                                                        hideMarks: BOOLEAN)

NEW, EXTENSIBLE

Connect the reader to a text line, possibly re-using an old text reader that is no longer in use. The reader is given the *text*, the line's horizontal left margin *x*, the line's starting position *pos* , the *ruler* and its position *rpos* dominating the line, and whether hideable marks are to be hidden. *vw* is the width against which text should be set.

Pre

text # NIL    20

0 <= pos    21

pos <= text.Length()    22

ruler # NIL    23

-1 <= rpos    24

rpos <= pos    25

Post

rd.r # NIL

rd.r.Base() = text

rd.r.eot OR rd.r.Pos() = pos + 1

rd.sString = ""

rd.string = ""

rd.view = NIL

rd.textOpts = {}

rd.setterOpts = {}

rd.w = 0, rd.endW = 0, rd.h = 0, rd.dsc = 0

rd.attr = NIL

rd.eot = FALSE

rd.pos = pos

rd.x = x

rd.adjStart = pos, rd.spaces = 0

rd.tabIndex = -1, rd.tabType = {}

rd.ruler = ruler, rd.rpos = rpos

rd.hideMarks = hideMarks

PROCEDURE (rd: Reader) **Read**

NEW, EXTENSIBLE (to be called by extensions first)

Read next element in line.

Post

~rd.r.eot

    rd.r.Pos() = rd.r'.Pos() + 1

~rd.eot

    view # NIL

        rd.pos = rd'.pos + 1

    string # ""

        rd.pos = rd'.pos + Length(rd.string)

    lstring # ""

        rd.pos = rd'.pos + Length(rd.string)

rd.eot

    rd.w = rd.endW = 0

    rd.h = ruler.style.attr.asc + ruler.style.attr.dsc

    rd.dsc = ruler.style.attr.dsc

PROCEDURE (rd: Reader) **AdjustWidth** (start, pos: INTEGER; IN box: LineBox;

                                                                        VAR w: INTEGER)

NEW, ABSTRACT

Given a line *box*, its starting position *start*, and the position *pos* of the element last read by the reader (normally *pos* = *rd.pos*), *AdjustWidth* takes that element's width *w* (normally *w* = *rd.w*) and adjusts it according to the formatting requirements of the line.

PROCEDURE (rd: Reader) **SplitWidth** (w: INTEGER): INTEGER

NEW, ABSTRACT

For the element read last by the reader, compute a split width. This is used for interactive purposes, where the split width splits coordinates into two half intervals: all coordinates to the left of the split point belong to the left edge of the element, all coordinates to the right of the split point belong to the right edge.

Post

0 <= result <= w

TYPE **Setter (Stores.Store)**

ABSTRACT

A setter can be used to set a text into lines, paragraphs, columns, and pages.

**text**-: TextModels.Model    setter connected iff text # NIL

Text to be set.

**defRuler**-: TextRulers.Ruler

Default ruler to be used to set the beginning of the text, in case the text has no ruler at position 0.

**vw**-: INTEGER    [units]

Total line width the setter should set against.

**hideMarks**-: BOOLEAN

If set, all embedded views that are by their preference hideable, will be reduced to zero width and zero height.

PROCEDURE (s: Setter) **ConnectTo** (text: TextModels.Model; defRuler: TextRulers.Ruler;

                                                                vw: INTEGER; hideMarks: BOOLEAN)

NEW, EXTENSIBLE

Disconnect setter from the text it was previously connected to (if any), and connect setter to given text (if any) using default ruler *defRuler* and hiding marks if requested by *hideMarks*. *vw* is the width against which text should be set.

Post

text = NIL

    s.text = NIL

    s.defRuler = NIL

text # NIL

    s.text = text

    s.defRuler = defRuler

    s.vw = vw

    s.hideMarks = hideMarks

PROCEDURE (s: Setter) **ThisPage** (pageH: INTEGER; pageNo: INTEGER): INTEGER

NEW, ABSTRACT

For a page of height *pageH*, determine the starting position for page number *pageNo* (with page numbering starting from 0).

Pre

Connected(s)    20

0 <= pageNo    21

Post

pageNo > LastPageNo(s.text, pageH)

    result = -1

pageNo <= LastPageNo(s.text, pageH)

    result = PageStart(pageNo)

PROCEDURE (s: Setter) **NextPage** (pageH: INTEGER; start: INTEGER): INTEGER

NEW, ABSTRACT

For a page of height *pageH* and a current page's starting position *start*, determine the starting position of the next page.

Pre

Connected(s)    20

0 <= start    21

start <= s.text.Length()    22

Exists pNo: s.ThisPage(pageH, pNo) = start    *not explicitly checked*

Post

start = LastPage(s.text, pageH)

    result = s.text.Length()

start < LastPage(s.text, pageH)

    result = PageStart(PageNo(s.text, start) + 1)

PROCEDURE (s: Setter) **ThisSequence** (pos: INTEGER): INTEGER

NEW, ABSTRACT

Locate the starting position of the (*line* or *para* separated) sequence containing position *pos*.

Pre

Connected(s)    20

0 <= pos    21

pos <= s.text.Length()    22

Post

result = 0  OR  char[result - 1] IN {line, para}

PROCEDURE (s: Setter) **NextSequence** (start: INTEGER): INTEGER

NEW, ABSTRACT

Locate the starting position of the next (*line* or *para* separated) sequence, given a starting position *start* of a current sequence.

Pre

Connected(s)    20

0 <= start    21

start <= s.text.Length()    22

s.ThisSequence(start) = start    23

Post

All pos: start < pos < s.text.Length(): s.ThisSequence(pos) = start

    result = start

s.ThisSequence(result - 1) = start

    result > start

    s.ThisSequence(result) = result

PROCEDURE (s: Setter) **PreviousSequence** (start: INTEGER): INTEGER

Locate the starting position of the previous (*line* or *para* separated) sequence, given a starting position *start* of a current sequence.

Pre

Connected(s)    20

0 <= start    21

start <= s.text.Length()    22

s.ThisSequence(start) = start    23

Post

start = 0

    result = 0

start > 0

    result = s.ThisSequence(start - 1)

PROCEDURE (s: Setter) **ThisLine** (pos: INTEGER): INTEGER

Locate the starting position of the line containing pos.

Pre

Connected(s)    20

0 <= pos    21

pos <= s.text.Length()    22

Post

result <= pos

pos < s.NextLine(result)  OR  LastLine(result)

PROCEDURE (s: Setter) **NextLine** (start: INTEGER): INTEGER

NEW, ABSTRACT

Locate the starting position of the next line, given the starting position *start* of the current line.

Pre

Connected(s)    20

0 <= start    21

start <= s.text.Length()    22

s.ThisLine(start) = start    23

Post

LastLine(start)

    result = start

~LastLine(start)

    result > start

    s.ThisLine(result - 1) = start

PROCEDURE (s: Setter) **PreviousLine** (start: INTEGER): INTEGER

NEW, ABSTRACT

Locate the starting position of the previous line, given the starting position *start* of the current line.

Pre

Connected(s)    20

0 <= start    21

start <= s.text.Length()    22

s.ThisLine(start) = start    23

Post

start = 0

    result = 0

start > 0

    result < start

    result = s.ThisLine(start - 1)

PROCEDURE (s: Setter) **GetWord** (pos: INTEGER; OUT beg, end: INTEGER)

Locate the beginning and ending positions of the word containing position *pos*. A word is a sequence of characters with code > " ", or views with mask > " ", or views with preference *wordPart*.

Pre

Connected(s)    20

0 <= pos    21

pos <= s.text.Length()    22

Post

beg <= pos <= end

PROCEDURE (s: Setter) **GetLine** (start: INTEGER; OUT box: LineBox)

NEW, ABSTRACT

Compute the characteristic box of the line with starting position start. (Cf. type *LineBox* below.)

Pre

Connected(s)    20

0 <= start    21

start <= s.text.Length()    22

s.ThisLine(start) = start    23

Post

min{box.left, box.first} <= box.left <= box.right <= ruler.right

~box.eot

    box.ruler # NIL

    box.len > 0

PROCEDURE (s: Setter) **GetBox** (start, end, maxW, maxH: INTEGER; OUT w, h: INTEGER)

NEW, ABSTRACT

Get the bounding box of a text stretch beginning at a line starting position *start* and ending at position *end*. The box computation will terminate if either the text stretch has been fully set, or if the box reached either of the limits, *maxW* bounding the box width, or *maxH* bounding the box height.

Pre

Connected(s)    20

0 <= start    21

start <= end    22

end <= s.text.Length()    23

Post

maxW > Views.undefined

    w <= maxW

maxH > Views.undefined

    h <= maxH

PROCEDURE (s: Setter) **NewReader** (old: Reader): Reader

Return a new reader, possibly reusing a given old reader that is no longer in use. (Whether the old reader is actually reused depends on internal compatibility conditions.)

Pre

Connected(s)    20

Post

result # NIL

PROCEDURE (s: Setter) **GridOffset** (dsc: INTEGER; VAR box: LineBox): INTEGER

Given the descender *dsc* of the preceding line and the current line characteristics *box*, return the grid correction to force the current line to the line grid. If the current line is the first line (of the text or on the current page), *dsc* = -1 should be passed.

Pre

Connected(s)

dsc >= -1

Post

~box.rbox

    Exists k: k >= 0: dsc + GridOffset(dsc, box) + box.asc =

                                    k * ruler.grid >= ruler.asc + ruler.grid

box.rbox

    result = 0

TYPE **LineBox**

The characteristics of a line set by a setter.

**len**: INTEGER

Length of the line.

**ruler**: TextRulers.Ruler

**rpos**: INTEGER

Ruler dominating the line, and its position in the text. (*rpos = -1* indicates that the line is dominated by the default ruler.)

**left**, **right**, **asc**, **dsc**: INTEGER    [units]

Left and right margins, and ascender and descender of the line's bounding box.

**rbox**: BOOLEAN

The line solely contains a ruler or a paragraph separator (*para* character or mask).

**bop**: BOOLEAN

The line is the first of a paragraph: Its left margin is *ruler.first*.; otherwise the left margin is *ruler.left*.

**adj**: BOOLEAN

The line needs adjustment when finally rendered: At least one element of the line needs to be artificially changed in width to achieve the requested formatting effect.

**eot**: BOOLEAN

The line is either empty, or it contains the last element of the text which is neither a *line* nor a *para* character or mask.

**views**: BOOLEAN

The line contains at least one embedded view.

**skipOff**: INTEGER    0 <= skipOff <= len

The characters in [*skipOff*, *len*) take on width *endW*.

**adjOff**: INTEGER    0 <= adjOff <= len

Offset of last block (sequence with no *tab* enclosed) in box. If the line is adjusted (centered, right flush, or fully adjusted), then this is the offset into the line where adjustment begins.

**spaces**: INTEGER    valid and > 0 if adj

Number of spaces subject to adjustment.

**adjW**: INTEGER    [units]    valid and > 0 if adj

The adjustment delta to be added either to the front of the last block for centered or right flushed formats, or to each space element (blank or view mapped to blank or view with preference *flexWidth*) for fully adjusted formats.

**tabW**: ARRAY TextRulers.maxTabs OF INTEGER    [units]    range [0 .. ruler.style.attr.tabs.len)

Widths of the gaps before tab stops.

TYPE **Directory**

ABSTRACT

Directory for setters.

PROCEDURE (d: Directory) **New** (): Setter

NEW, ABSTRACT

Return a new setter.

VAR **dir**-, **stdDir**-: Directory    dir # NIL, stdDir # NIL, stable stdDir = d

Directory and standard directory objects.

PROCEDURE **SetDir** (d: Directory)

Set the directory object.

Pre

d # NIL    20

Post

dir = d

