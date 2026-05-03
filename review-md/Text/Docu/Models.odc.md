**TextModels**

DEFINITION TextModels;

    IMPORT Fonts, Ports, Stores, Models, Views, Properties, Containers;

    CONST

        viewcode = 2X;

        tab = 9X; line = 0DX; para = 0EX;

        zwspace = 8BX; nbspace = 0A0X; digitspace = 8FX;

        hyphen = 90X; nbhyphen = 91X; softhyphen = 0ADX;

        maskChar = 0; hideable = 1;

        offset = 0; code = 1;

        store = 0;

        replace = 0; insert = 1; delete = 2;

    TYPE

        Model = POINTER TO ABSTRACT RECORD (Containers.Model)

            (m: Model) Length (): INTEGER, NEW, ABSTRACT;

            (m: Model) NewReader (old: Reader): Reader, NEW, ABSTRACT;

            (m: Model) NewWriter (old: Writer): Writer, NEW, ABSTRACT;

            (m: Model) Insert (pos: INTEGER; m0: Model; beg0, end0: INTEGER), NEW, ABSTRACT;

            (m: Model) InsertCopy (pos: INTEGER; m0: Model; beg0, end0: INTEGER), NEW, ABSTRACT;

            (m: Model) Delete (beg, end: INTEGER), NEW, ABSTRACT;

            (m: Model) Append (m0: Model), NEW, ABSTRACT;

            (m: Model) Replace (beg, end: INTEGER; m0: Model; beg0, end0: INTEGER), NEW, ABSTRACT

            (m: Model) SetAttr (beg, end: INTEGER; attr: Attributes), NEW, EXTENSIBLE;

            (m: Model) Prop (beg, end: INTEGER): Properties.Property, NEW, ABSTRACT;

            (m: Model) Modify (beg, end: INTEGER; old, p: Properties.Property), NEW, ABSTRACT;

            (m: Model) ReplaceView (old, new: Views.View), ABSTRACT;

        END;

        Attributes = POINTER TO EXTENSIBLE RECORD (Stores.Store)

            init-: BOOLEAN;

            color-: Ports.Color;

            font-: Fonts.Font;

            offset-: INTEGER;

            (a: Attributes) Equals (b: Attributes): BOOLEAN, NEW, EXTENSIBLE;

            (a: Attributes) Prop (): Properties.Property, NEW, EXTENSIBLE;

            (a: Attributes) InitFromProp (p: Properties.Property), NEW, EXTENSIBLE;

            (a: Attributes) ModifyFromProp- (p: Properties.Property), NEW, EXTENSIBLE

        END;

        AlienAttributes = POINTER TO RECORD (Attributes)

            store-: Stores.Alien

        END;

        Prop = POINTER TO RECORD (Properties.Property)

            offset: INTEGER;

            code: CHAR

        END;

        Context = POINTER TO ABSTRACT RECORD (Models.Context)

            (c: Context) ThisModel (): Model, ABSTRACT;

            (c: Context) Pos (): INTEGER, NEW, ABSTRACT;

            (c: Context) Attr (): Attributes, NEW, ABSTRACT

        END;

        Pref = RECORD (Properties.Preference)

            opts: SET;

            mask: CHAR

        END;

        Reader = POINTER TO ABSTRACT RECORD

            eot: BOOLEAN;

            attr: Attributes;

            char: CHAR;

            view: Views.View;

            w, h: INTEGER;

            (rd: Reader) Base (): Model, NEW, ABSTRACT;

            (rd: Reader) SetPos (pos: INTEGER), NEW, ABSTRACT;

            (rd: Reader) Pos (): INTEGER, NEW, ABSTRACT;

            (rd: Reader) Read, NEW, ABSTRACT;

            (rd: Reader) ReadChar (OUT ch: CHAR), NEW, ABSTRACT;

            (rd: Reader) ReadView (OUT v: Views.View), NEW, ABSTRACT;

            (rd: Reader) ReadRun (OUT attr: Attributes), NEW, ABSTRACT;

            (rd: Reader) ReadPrev, NEW, ABSTRACT;

            (rd: Reader) ReadPrevChar (OUT ch: CHAR), NEW, ABSTRACT;

            (rd: Reader) ReadPrevView (OUT v: Views.View), NEW, ABSTRACT;

            (rd: Reader) ReadPrevRun (OUT attr: Attributes), NEW, ABSTRACT

        END;

        Writer = POINTER TO ABSTRACT RECORD

            attr-: Attributes;

            (wr: Writer) Base (): Model, NEW, ABSTRACT;

            (wr: Writer) Pos (): INTEGER, NEW, ABSTRACT;

            (wr: Writer) SetPos (pos: INTEGER), NEW, ABSTRACT;

            (wr: Writer) SetAttr (attr: Attributes), NEW;

            (wr: Writer) WriteChar (ch: CHAR), NEW, ABSTRACT;

            (wr: Writer) WriteView (view: Views.View; w, h: INTEGER), NEW, ABSTRACT

        END;

        InfoMsg = RECORD (Models.Message)

            op: INTEGER

        END;

        UpdateMsg = RECORD (Models.UpdateMsg)

            op: INTEGER;

            beg, end, delta: INTEGER

        END;

        Directory = POINTER TO ABSTRACT RECORD

            attr-: Attributes;

            (d: Directory) New (): Model, NEW, ABSTRACT;

            (d: Directory) NewFromString (s: ARRAY OF CHAR): Model, NEW, EXTENSIBLE;

            (d: Directory) SetAttr (attr: Attributes), NEW, EXTENSIBLE

        END;

    VAR dir-, stdDir-: Directory;

    PROCEDURE NewColor (a: Attributes; color: Ports.Color): Attributes;

    PROCEDURE NewFont (a: Attributes; font: Fonts.Font): Attributes;

    PROCEDURE NewOffset (a: Attributes; offset: INTEGER): Attributes;

    PROCEDURE NewTypeface (a: Attributes; typeface: Fonts.Typeface): Attributes;

    PROCEDURE NewSize (a: Attributes; size: INTEGER): Attributes;

    PROCEDURE NewStyle (a: Attributes; style: SET): Attributes;

    PROCEDURE NewWeight (a: Attributes; weight: INTEGER): Attributes;

    PROCEDURE ReadAttr (VAR rd: Stores.Reader; VAR a: Attributes);

    PROCEDURE WriteAttr (VAR wr: Stores.Writer; a: Attributes);

    PROCEDURE ModifiedAttr (a: Attributes; p: Properties.Property): Attributes;

    PROCEDURE CloneOf (source: Model): Model;

    PROCEDURE WriteableChar (ch: CHAR): BOOLEAN;

    PROCEDURE SetDir (d: Directory);

END TextModels.

*TextModels* are container models which contain sequences of attributed characters and embedded views. Characters may be drawn from the BlackBox character set, conforming to the Unicode specification. However, the current implementation supports the ASCII character set and its Latin-1 extension, only.

CONST **viewcode**

Signals that the read character is actually an embedded view. *viewcode* is used as a general projection value when reading an embedded view as a *CHAR*.

CONST **tab**

The tabulation character. When encountered by a text formatter, formatting continues at the next tab stop as defined by some tabulation raster used by the formatter (usually some kind of ruler).

CONST **line**

The line separation character. When encountered by a text setter, setting continues on the next line. However, *line* does not introduce a new paragraph (cf. *para* below).

CONST **para**

The paragraph separation character. When encountered by a text setter, setting continues on the next line and a new paragraph is opened (cf. *line* above).

CONST **zwspace**

The zero-width space character. Separates words, but takes no space in its own right.

CONST **nbspace**

The non-breaking space character. Has the same width as a normal space character. When encountered by a text setter, *nbspace* does not separate words.

CONST **digitspace**

In most fonts, a digit space has the width of digit zero (0) which is equivalent to the width of all digits in most fonts. When encountered by a text setter, *digitspace* does not separate words. (*Note: *The recommendation made in earlier versions of the text system, namely to use *digitspace* for number formatting, is outdated. Use right-aligning tab stops instead. See [<u>TextRulers</u>](Rulers.odc.md).)

CONST **hyphen**

The hyphen character. To be used for explicit and visible hyphenation. A text setter may choose to break lines just after a *hyphen*.

CONST **nbhyphen**

The non-breaking hyphen. Just as *hyphen*, but a text setter will not break lines just after an *nbhyphen*.

CONST **softhyphen**

The soft-hyphen. Just as *hyphen*, but rendered as a zero-width character unless actually used to break a line. *softhyphen* can be used to give hints to a text setter on where to break longer words.

CONST **maskChar**

Option element

Can be used as a set element of *Pref.opts*. Signals that the embedded view prefers to be masked to a normal character, making it behave like that character in most situations. The primary purpose of masking is to simplify the task for text scanning applications. (For example, a text ruler might be masked to behave like a *para* character. This would allow a text scanner to count the number of paragraphs in a text simply by counting the number of returned *para* characters while scanning the text.)

CONST **hideable**

Option element

Can be used as a set element of *Pref.opts*. Signals that the embedded view accepts to be hidden or revealed depending on a mode of the text view. (For example, text rulers might be hideable enabling the the user to hide or reveal the rulers in a displayed text.)

CONST **offset**, **code**

Property field selectors

Signals that the indicated field of a *Prop* property is known, valid, or readOnly.

CONST **store**

Possible value of *InfoMsg.op*, signalling the completed storing of the broadcasting text. This is the only value currently defined for this field.

CONST **replace**

Possible value of *UpdateMsg.op*, signalling the successful replacement of a stretch in the broadcasting text.

CONST **insert**

Possible value of *UpdateMsg.op*, signalling the successful insertion of a stretch in the broadcasting text.

CONST **delete**

Possible value of *UpdateMsg.op*, signalling the successful deletion of a stretch in the broadcasting text.

TYPE **Model (Containers.Model)**

ABSTRACT

Text models are container models, containing sequences of attributed characters and embedded views.

PROCEDURE (m: Model) **Length** (): INTEGER

NEW, ABSTRACT

Length of the text *m*, where each character and each embedded view counts as one.

Post

result >= 0

PROCEDURE (m: Model) **NewReader** (old: Reader): Reader

NEW, ABSTRACT

Returns a reader connected to *m*. An old reader may be passed as input parameter, if it isn't in use anymore. Note that *NewReader* does not reposition *old* if *old* is reused and its position is in the valid range. (*NewReader* may or may not use the old reader, depending on internal compatibility criteria.)

Post

result # NIL

result = old  &  old'.Base() = m  &  old'.Pos() <= m.Length()

    result.Pos() = old'.Pos()

~(result = old  &  old'.Base() = m  &  old'.Pos() <= m.Length())

    result.Pos() = 0

PROCEDURE (m: Model) **NewWriter** (old: Writer): Writer

NEW, ABSTRACT

Returns a writer connected to *m*. An old writer may be passed as input parameter, if it isn't in use anymore. Note that *NewWriter* does not reposition *old* if *old* is reused and its position is in the valid range. (*NewWriter* may or may not use the old writer, depending on internal compatibility criteria.)

Post

result # NIL

result.attr = dir.attr

result = old  &  old'.Base() = m  &  old'.Pos() <= m.Length()

    result.Pos() = old'.Pos()

~(result = old  &  old'.Base() = m  &  old'.Pos() <= m.Length())

    result.Pos() = m.Length()

PROCEDURE (m: Model) **Insert** (pos: INTEGER; m0: Model; beg0, end0: INTEGER)

NEW, ABSTRACT, Operation

Extract the stretch [*beg0*, *end0*) from *m0* and insert it into *m* at position *pos*. In case that *m0* is of a different implementation than *m*, rider conversion is used to project the stretch from *m0* into *m*'s implementation. Model *m0* is made shorter, while model *m* is made longer by *(end0 - beg0)*.

Pre

0 <= pos    21

pos <= Length(m)    22

0 <= beg0    23

beg0 <= end0    24

end0 <= Length(m0)    25

m0 # NIL    *not explicitly checked*

Post

m = m0

    Length(m) = Length(m0) = Length(m') = Length(m0')

m # m0

    Length(m) = Length(m') + (end0 - beg0)

    Length(m0) = Length(m0') - (end0 - beg0)

PROCEDURE (m: Model) **InsertCopy** (pos: INTEGER; m0: Model; beg0, end0: INTEGER)

NEW, ABSTRACT, Operation

Copy the stretch [*beg0*, *end0*) from text *m0* and insert it into text *m* at position *pos*. In case that *m0* is of a different implementation than *m*, rider conversion is used to project the stretch from *m0* into *m*'s implementation.

Pre

0 <= pos    21

pos <= Length(m)    22

0 <= beg0    23

beg0 <= end0    24

end0 <= Length(m0)    25

m0 # NIL    *not explicitly checked*

Post

Length(m) = Length(m') + (end0 - beg0)

m0 = m

    Length(m0) = Length(m)

m0 # m

    Length(m0) = Length(m0')

PROCEDURE (m: Model) **Delete** (beg, end: INTEGER);

NEW, ABSTRACT, Operation

Delete the stretch [*beg*, *end*) from *m*.

Pre

0 <= beg    20

beg <= end    21

end <= Length(m)    22

Post

Length(m) = Length(m') - (end - beg)

PROCEDURE (m: Model) **Append** (m0: Model)

NEW, ABSTRACT, Operation

Append *m0* to *m*.

Except for performance, equivalent to:

    m.Insert(m.Length(), m0, 0, m0.Length())

PROCEDURE (m: Model) **Replace** (beg, end: INTEGER; m0: Model; beg0, end0: INTEGER)

NEW, ABSTRACT, Operation

Extract the stretch [*beg0*, *end0*) from *m0* and replace stretch [*beg*, *end*) in *m* by the stretch extracted from *m0*.

Pre

0 <= beg    20

beg <= end    21

end <= Length(m)    22

0 <= beg0    23

beg0 <= end0    24

end0 <= Length(m0)    25

m # m0    26

m0 # NIL    *not explicitly checked*

Except for performance, equivalent to:

    VAR script: Stores.Operation;

    Models.BeginScript(m, "#System:Replacing", script);

    m.Delete(beg, end); m.Insert(beg, m0, beg0, end0);

    Models.EndScript(m, script)

PROCEDURE (m: Model) **SetAttr** (beg, end: INTEGER; attr: Attributes)

NEW, ABSTRACT, Operation

Set the attributes of all items in the stretch [*beg*, *end*) of *m* to *attr*.

Pre

0 <= beg    20

beg <= end    21

end <= Length(m)    22

attr # NIL    *not explicitly checked*

attr.init    23

PROCEDURE (m: Model) **Prop** (beg, end: INTEGER): Properties.Property

NEW, ABSTRACT

Return a property structure describing the properties of the items in the stretch [*beg*, *end*) of *m*. Only properties returned by encountered attributes and those captured by *Prop* are considered. There is no recursion into embedded views.

The result is the intersection of the attribute-describing properties of the homogeneous substretches of [*beg*, *end*). For example, if the first half of the stretch is "bold & italic", while the second half is "italic", then *Prop* returns a property "italic", i.e. the homogeneous subset.

Pre

0 <= beg    20

beg <= end    21

end <= Length(m)    22

PROCEDURE (m: Model) **Modify** (beg, end: INTEGER; old, p: Properties.Property)

NEW, ABSTRACT, Operation

Modify the stretch [*beg*, *end*) of *m* according to the property structure *p*. If *old* is given, modification takes place only if the stretch is homogeneous in the properties specified in *old*, and if the stretch carries exactly the same property values as those specified in *old*.

Pre

0 <= beg    20

beg <= end    21

end <= Length(m)    22

Post

old = NIL  OR  old = Intersect(m'.Prop(beg, end), old)

    m.Prop(beg, end) = p

PROCEDURE (m: Model) **ReplaceView** (old, new: Views.View)

NEW, ABSTRACT, Operation

Retain the context of *old*, but replace *old* by *new*.

Pre

old # NIL    20

new # NIL    21

old.context # NIL    22

new.context = NIL OR new.context = old.context    23

Post

new.context = old.context

TYPE **Attributes (Stores.Store)**

EXTENSIBLE

Every character or embedded view that forms part of a text carries a set of attributes. Such attributes are described by objects of type *Attributes*. The base type carries the standard attributes of every element of a text: A color, a font, and a vertical offset (in universal units).

Once created and initialized, attributes objects can no longer be modified, and hence can be freely shared among many attributed objects. Changing the attributes of an attributed object is done by replacing the whole attributes object attached to the attributed object, usually by a modified copy of the original attributes object.

(*Note:* attributes objects are *stores* and as such belong to a domain: where the attributes held by an attributes objects are to be applied to another attributed object in a different domain, the attributes object *must be copied*.)

**init**-: BOOLEAN

Object has been initialized and can no longer be modified.

**color**-: Ports.Color

Persistent

The color to be used to render the attributed object. For characters, this is the foreground color; for embedded views, this attribute is either ignored, or used in a view-specific way by text-aware views.

**font**-: Fonts.Font

Persistent

The font to be used to render the attributed object. For characters, this is the font carrying the glyph to be used; for embedded views, this attribute is either ignored, or used in a view-specific way by text-aware views.

**offset**-: INTEGER

Persistent

The vertical offset from the base line (value in universal units) to be used for the attributed object.

PROCEDURE (a: Attributes) **Equals** (b: Attributes): BOOLEAN

NEW, EXTENSIBLE

Compare two attributes objects for attribute equality.

Pre

b # NIL    NIL dereference

a.init    20

(b # NIL) & b.init    20

Post

result = (TypeOf(a) = TypeOf(b)) & (a.color = b.color) & (a.font = b.font) & (a.offset = b.offset)

PROCEDURE (a: Attributes) **Prop** (): Properties.Property

NEW, EXTENSIBLE

Return property list reflecting attribute values.

Post

result # NIL

PROCEDURE (a: Attributes) **ModifyFromProp**- (p: Properties.Property)

NEW, EXTENSIBLE

Initialize new attributes object to attributes of a source object, but modified according to the property list. (Values valid in the property list are taken from it, others from the source attributes object.) This is called by procedure *ModifiedAttr.*

PROCEDURE (a: Attributes) **InitFromProp** (p: Properties.Property)

NEW, EXTENSIBLE

Initialize according to property list.

Pre

~a.init    20

Post

a.init

TYPE **AlienAttributes (Attributes)**

Type of alien attributes objects, as returned by *ReadAttr*.

**store**-: Stores.Alien

The alien store enclosed by an alien attributes objects.

TYPE **Prop (Properties.Property)**

Text specific properties: Vertical offsets and character codes.

**offset**: INTEGER

Vertical offset property.

**code**: CHAR

Character code.

TYPE **Context (Models.Context)**

ABSTRACT

Context for views embedded in texts.

PROCEDURE (c: Context) **ThisModel** (): Model

ABSTRACT

Result type is narrowed.

PROCEDURE (c: Context) **Pos** (): INTEGER

NEW, ABSTRACT

Position of the embedded view in the text.

PROCEDURE (c: Context) **Attr** (): Attributes

NEW, ABSTRACT

Attributes valid for the embedded view.

TYPE **Pref (Properties.Preference)**

Preferences a view may have when embedded in a text.

**opts**: SET

Option set, preset to {}. Possible values are from {*maskChar*, *hideable*}.

**mask**: CHAR

If *maskChar* IN *opts*, *mask* is the desired masking character code.

TYPE **Reader**

ABSTRACT

A rider to read characters and embedded views from a text.

**eot**: BOOLEAN

Last read was tried at the end of the text.

**attr**: Attributes    ~eot => attr # NIL

The attributes of the most recently read element.

**char**: CHAR

Character read most recently; takes value *viewcode* if last element read was a view that does not mask itself with a different character code (see *Pref*).

**view**: Views.View

Embedded view most recently read; takes value NIL if last element read was a character.

**w**, **h**: INTEGER    valid if *view* # NIL

Width and height of view most recently read.

PROCEDURE (rd: Reader) **Base** (): Model

NEW, ABSTRACT

The rider base: The text the reader is attached to.

PROCEDURE (rd: Reader) **Pos** (): INTEGER

NEW, ABSTRACT

Position of the reader in the text.

Post

0 <= result

result <= rd.Base().Length()

PROCEDURE (rd: Reader) **SetPos** (pos: INTEGER)

NEW, ABSTRACT

Reposition the reader.

Pre

0 <= pos    20

rd.Base() # NIL    21

pos <= rd.Base().Length()    22

Post

rd.Pos() = pos

PROCEDURE (rd: Reader) **Read**

NEW, ABSTRACT

Read the next element of the text.

Post

~rd.eot

    rd.Pos() = rd'.Pos() + 1, rd.attr # NIL, rd.attr.init

    rd.view # NIL

        maskChar IN Prefs(view).opts, ch = Prefs(view).mask

            rd.char = ch

        ~(maskChar IN Prefs(view).opts)

            rd.char = viewcode

rd.eot

    rd.Pos() = rd'.Pos(), rd.attr = NIL, rd.char = 0X, rd.view = NIL

PROCEDURE (rd: Reader) **ReadPrev**

NEW, ABSTRACT

Read the previous element of the text: First, decrements *rd.Pos*(), then reads element at *rd.Pos*().

Post

~rd.eot

    rd.Pos() = rd'.Pos() - 1, rd.attr # NIL, rd.attr.init

    rd.view # NIL

        maskChar IN Prefs(view).opts, ch = Prefs(view).mask

            rd.char = ch

        ~(maskChar IN Prefs(view).opts)

            rd.char = viewcode

rd.eot

    rd.Pos() = rd'.Pos(), rd.attr = NIL, rd.char = 0X, rd.lchar = 0, rd.view = NIL

PROCEDURE (rd: Reader) **ReadChar** (OUT ch: CHAR)

NEW, ABSTRACT

Read the next character or projection value of the next embedded view.

Except for performance, equivalent to:

    rd.Read; ch := rd.char

PROCEDURE (rd: Reader) **ReadPrevChar** (OUT ch: CHAR)

NEW, ABSTRACT

Read the previous character or projection value of the previous embedded view.

Except for performance, equivalent to:

    rd.ReadPrev; ch := rd.char

PROCEDURE (rd: Reader) **ReadView** (OUT v: Views.View)

NEW, ABSTRACT

Read next view.

Except for performance, equivalent to:

    REPEAT rd.Read UNTIL (rd.view # NIL) OR rd.eot;

    v := rd.view

PROCEDURE (rd: Reader) **ReadPrevView** (OUT v: Views.View)

NEW, ABSTRACT

Read previous view.

Except for performance, equivalent to:

    REPEAT rd.ReadPrev UNTIL (rd.view # NIL) OR rd.eot;

    v := rd.view

PROCEDURE (rd: Reader) **ReadRun** (OUT attr: Attributes)

NEW, ABSTRACT

Read next attribute run, stops at next view.

Except for performance, equivalent to:

    VAR a: Attributes;

    a := rd.attr;

    REPEAT rd.Read UNTIL (rd.attr # a) OR (rd.view # NIL) OR rd.eot;

    IF rd.eot THEN attr := NIL ELSE attr := rd.attr END

Post

~rd.eot

    attr # NIL, rd.attr.init

    rd.view = ViewAt(rd.Pos() - 1)

PROCEDURE (rd: Reader) **ReadPrevRun** (OUT attr: Attributes)

NEW, ABSTRACT

Read next attribute run, stops at next view.

Except for performance, equivalent to:

    VAR a: Attributes;

    a := rd.attr;

    REPEAT rd.ReadPrev UNTIL (rd.attr # a) OR (rd.view # NIL) OR rd.eot;

    IF rd.eot THEN attr := NIL ELSE attr := rd.attr END

Post

~rd.eot

    attr # NIL, rd.attr.init

    rd.view = ViewAt(rd.Pos())

TYPE **Writer**

ABSTRACT

Write characters, long characters, or embed views into a text.

**attr**-: Attributes    attr # NIL, attr.init

The attributes object to attach to the next written element. (Attributes are immutable, thus they can be shared and need not be copied if used somewhere else *in the same domain*.)

PROCEDURE (wr: Writer) **Base** (): Model

NEW, ABSTRACT

Return the rider base, i.e. the text the writer is currently attached to.

PROCEDURE (wr: Writer) **Pos** (): INTEGER

NEW, ABSTRACT

Position of the writer in the text.

Post

0 <= result

result <= wr.Base().Length()

PROCEDURE (wr: Writer) **SetPos** (pos: INTEGER)

NEW, ABSTRACT

Reposition the writer.

Pre

0 <= pos    20

wr.Base() # NIL    21

pos <= wr.Base.Length()    22

Post

wr.Pos() = pos

PROCEDURE (wr: Writer) **SetAttr** (attr: Attributes)

NEW

Sets the current attributes of the writer.

Pre

attr # NIL    *not explicitly checked*

attr.init    20

Post

wr.attr = a    [ a.Equals(attr) ]

PROCEDURE (wr: Writer) **WriteChar** (ch: CHAR)

NEW, ABSTRACT, Operation

Write character with attributes *wr.attr*. Nothing happens if *ch* is not writeable (see procedure *WriteableChar*).

Post

WriteableChar(ch)

    wr.Pos() = wr.Pos'() + 1

~WriteableChar(ch)

    wr.Pos() = wr.Pos'()

PROCEDURE (wr: Writer) **WriteView** (view: Views.View; w, h: INTEGER)

NEW, ABSTRACT, Operation

Write view with width *w*, height *h*, and attributes *wr.attr*. *w* and *h* may have the value *Views.undefined*.

Pre

view # NIL    20

view.context = NIL    21

Post

view.context # NIL

view.context.Pos() = wr.Pos()'

view.context.Attr() = w    [ w.Equals(wr.attr) ]

TYPE **InfoMsg (Models.Message)**

Message notifying about a non-destructive operation on a text.

**op**: INTEGER

For standard texts, there is only one op code defined: *store*.

TYPE **UpdateMsg (Models.UpdateMsg)**

Message notifying about a destructive operation on a text.

**op**: INTEGER

Kind of operation performed. For standard texts, *op* IN {*replace*, *insert*, *delete*}.

**beg**, **end**, **delta**: INTEGER

The operation was performed on the stretch [*beg*, *end*) measured in the text after the operation took place, and changed the length of the text by *delta*. For deletions of length *l, end = beg + 1, delta = -l*; for insertions of length *l, end = beg + l, delta = l*; for replacements of length *l* but not changing the text length, *end = beg + l, delta = 0*.

TYPE **Directory**

ABSTRACT

Directory for text models.

**attr**-: Attributes

Default attributes, used when opening a writer on a text.

PROCEDURE (d: Directory) **New** (): Model;

NEW, ABSTRACT

Return a new text model.

PROCEDURE (d: Directory) **NewFromString** (s: ARRAY OF CHAR): Model

NEW, EXTENSIBLE

Return a new text model with *s* written into it using initial default attributes.

Except for performance, equivalent to:

    VAR m: Model; w: Writer; i: INTEGER;

    m := d.New(); w := m.NewWriter(NIL);

    i := 0; WHILE s[i] # 0X DO w.WriteChar(s[i]); INC(i) END;

    RETURN m

PROCEDURE (d: Directory) **SetAttr** (attr: Attributes)

NEW, EXTENSIBLE

Set the default attributes.

Pre

attr # NIL    *not explicitly checked*

attr.init    20

Post

d.attr = a    [ a.Equals(attr) ]

VAR **dir**-, **stdDir**-: Directory    dir # NIL, stdDir # NIL, stable stdDir = d

Directory and standard directory for text models.

PROCEDURE  **NewColor** (a: Attributes; color: Ports.Color): Attributes

PROCEDURE  **NewTypeface** (a: Attributes; typeface: Fonts.Typeface): Attributes

PROCEDURE  **NewSize** (a: Attributes; size: INTEGER): Attributes

PROCEDURE  **NewStyle** (a: Attributes; style: SET): Attributes

PROCEDURE  **NewWeight** (a: Attributes; weight: INTEGER): Attributes

PROCEDURE  **NewOffset** (a: Attributes; offset: INTEGER): Attributes

Take an existing attributes object and return a new one with equal attributes except for the specified one.

Pre

a # NIL    20

a.init    21

Post

result # NIL

PROCEDURE  **NewFont** (a: Attributes; font: Fonts.Font): Attributes

Changes the entire font attribute, i.e., weight, style, size, and typeface.

Pre

a # NIL    20

a.init    21

Except for performance, equivalent to:

    NewTypeface(NewSize(NewStyle(NewWeight(

        a, font.weight), font.style), font.size), font.typeface)

PROCEDURE  **ReadAttr** (VAR rd: Stores.Reader; VAR a: Attributes)

Reads an attributes object. In case the reader returns an alien store, the store is wrapped into an alien attributes object and the wrapper is returned.

Pre

~rd.rider.eof

NextStore(rd) IS Attributes

Post

a # NIL

~MapsToAlien(NextStore(rd'))

    a = NextStore(rd')

MapsToAlien(NextStore(rd'))

    a IS AlienAttributes

    a.store = NextStore(rd')

PROCEDURE  **WriteAttr** (VAR wr: Stores.Writer; a: Attributes)

Writes an attributes object. In case *a* is an alien attributes object, its contained alien stores is unwrapped and written.

Pre

a # NIL    20

Except for performance, equivalent to:

    WITH a: AlienAttributes DO wr.WriteStore(a.store) ELSE wr.WriteStore(a) END

PROCEDURE  **ModifiedAttr** (a: Attributes; p: Properties.Property): Attributes

Return new attributes object that shares attribute settings of *a*, except where overridden by settings in *p*.

Pre

a # NIL    *not explicitly checked*

a.init    20

Post

*x* IN p.valid

    *x*-attr(result) = *x*-attr(p)

~(*x* IN p.valid)

    *x*-attr(result) = *x*-attr(a)

Except for performance, equivalend to:

    VAR h: Attributes;

    h := Stores.CopyOf(a)(Attributes); h.ModifyFromProp(p);

    RETURN h

PROCEDURE  **WriteableChar** (ch: CHAR): BOOLEAN

Determines whether *ch* may be written to a text.

Post

result =

    ch = tab  OR  ch = line  OR  ch = para  OR

    ch >= " "  &  ch < 07FX  OR

    ch = zwspace  OR  ch = digitspace  OR  ch = hyphen  OR  ch = nbhyphen  OR

    ch >= 0A0X    20

PROCEDURE  **CloneOf** (source: Model): Model

This procedure should be used to obtain a new text from the same type as another existing text.

Pre

source # NIL    20

Except for performance, equivalent to:

        RETURN Containers.CloneOf(source)(Model)

PROCEDURE  **SetDir** (d: Directory)

Set model directory.

Pre

d # NIL    20

d.attr # NIL    21

Post

dir = d

