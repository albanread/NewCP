**TextRulers**

DEFINITION TextRulers;

    IMPORT Stores, Models, Views, Properties, TextModels;

    CONST

        first = 0; left = 1; right = 2; lead = 3; asc = 4; dsc = 5; grid = 6; opts = 7; tabs = 8;

        leftAdjust = 0; rightAdjust = 1; noBreakInside = 2; pageBreak = 3; parJoin = 4;

        rightFixed = 5;

        maxTabs = 32;

        centerTab = 0; rightTab = 1; barTab = 2;

    TYPE

        Tab = RECORD

            stop: INTEGER;

            type: SET

        END;

        TabArray = RECORD

            len: INTEGER;

            tab: ARRAY maxTabs OF Tab

        END;

        Attributes = POINTER TO EXTENSIBLE RECORD (Stores.Store)

            init-: BOOLEAN;

            first-, left-, right-, lead-, asc-, dsc-, grid-: INTEGER;

            opts-: SET;

            tabs-: TabArray;

            (a: Attributes) InitFromProp (p: Properties.Property), NEW, EXTENSIBLE;

            (a: Attributes) ModifyFromProp- (p: Properties.Property), NEW, EXTENSIBLE;

            (a: Attributes) Equals (b: Attributes): BOOLEAN, NEW, EXTENSIBLE;

            (a: Attributes) Prop (): Properties.Property, NEW, EXTENSIBLE

        END;

        AlienAttributes = POINTER TO RECORD (Attributes)

            store-: Stores.Alien

        END;

        Style = POINTER TO ABSTRACT RECORD (Models.Model)

            attr-: Attributes;

            (s: Style) SetAttr (attr: Attributes), NEW, EXTENSIBLE

        END;

        Ruler = POINTER TO ABSTRACT RECORD (Views.View)

            style-: Style

        END;

        Prop = POINTER TO  RECORD (Properties.Property)

            first, left, right, lead, asc, dsc, grid: INTEGER;

            opts: RECORD

                val, mask: SET

            END;

            tabs: TabArray

        END;

        UpdateMsg = RECORD (Models.UpdateMsg)

            style: Style;

            oldAttr: Attributes

        END;

        Directory = POINTER TO RECORD

            attr-: Attributes;

            (d: Directory) New (style: Style): Ruler, NEW, ABSTRACT;

            (d: Directory) NewFromProp (p: Prop): Ruler, NEW, EXTENSIBLE;

            (d: Directory) NewStyle (attr: Attributes): Style, NEW, ABSTRACT;

            (d: Directory) SetAttr (attr: Attributes), NEW, EXTENSIBLE

        END;

    VAR dir-, stdDir-: Directory;

    PROCEDURE ReadAttr (VAR rd: Stores.Reader; VAR a: Attributes);

    PROCEDURE WriteAttr (VAR wr: Stores.Writer; a: Attributes);

    PROCEDURE ModifiedAttr (a: Attributes; p: Properties.Property): Attributes;

    PROCEDURE SetCentered (r: Ruler);

    PROCEDURE SetJustified (r: Ruler);

    PROCEDURE SetLeftFlush (r: Ruler);

    PROCEDURE SetRightFlush (r: Ruler);

    PROCEDURE SetNoBreakInside (r: Ruler);

    PROCEDURE SetPageBreak (r: Ruler);

    PROCEDURE SetParJoin (r: Ruler);

    PROCEDURE SetAsc (r: Ruler; h: INTEGER);

    PROCEDURE SetDsc (r: Ruler; h: INTEGER);

    PROCEDURE SetFirst (r: Ruler; x: INTEGER);

    PROCEDURE SetGrid (r: Ruler; h: INTEGER);

    PROCEDURE SetLead (r: Ruler; h: INTEGER);

    PROCEDURE SetLeft (r: Ruler; x: INTEGER);

    PROCEDURE SetRight (r: Ruler; x: INTEGER);

    PROCEDURE SetFixedRight (r: Ruler; x: INTEGER);

    PROCEDURE AddTab (r: Ruler; x: INTEGER);

    PROCEDURE MakeBarTab (r: Ruler);

    PROCEDURE MakeCenterTab (r: Ruler);

    PROCEDURE MakeRightTab (r: Ruler);

    PROCEDURE CopyOf (r: Ruler; shallow: BOOLEAN): Ruler;

    PROCEDURE GetValidRuler (text: TextModels.Model; pos, hint: INTEGER;

                                                VAR ruler: Ruler; VAR rpos: INTEGER);

    PROCEDURE Deposit;

    PROCEDURE SetDir (d: Directory);

END TextRulers.

*TextRulers* are text-aware views that, if embedded in a text, affect the text setting. A ruler supports interactive and program controlled adjustment of paragraph and limited page formatting. However, the ruler does not perform the actual setting of the text according to the format defined by the ruler. Text setters are used to actually set a text (cf. Section 6.4; [<u>TextSetters</u>](Setters.odc.md)).

CONST **first**, **left**, **right**, **lead**, **asc**, **dsc**, **grid**, **opts**, **tabs**

Property field selectors

Signals that the named field of a *Prop* property is known, valid, or readOnly.

CONST **leftAdjust**, **rightAdjust**

Possible elements of *Attributes.opts*. If none is included, request centered lines; if leftAdjust is included alone, request left flushed lines; if rightAdjust is included alone, request right flushed lines; if both are included, request fully adjusted lines.

CONST **noBreakInside**, **pageBreak**, **parJoin**

Possible elements of *Attributes.opts*, where the inclusion of *pageBreak* overrides a possible inclusion of *parJoin*. If *noBreakInside* is included, request setting of the following paragraphs (up to the next ruler or the end of text) such that no page break is crossed by any of the paragraphs; a page break may be inserted between the paragraphs, but see *parJoin* below. (This may not be possible, or lead to unacceptably large white space, in which case a setter's heuristics may insert page breaks anyway.) If *pageBreak* is included, request setting to continue on the next fresh page. (If already at the top of a new page, the request is ignored.) If *parJoin* is included, the text following the ruler is requested to be set on the same page as the text following the next ruler. (Again, the setter may not be able to honour this request and be forced to insert page breaks anyway.)

CONST **rightFixed**

Possible element of *Attributes.opts*. If included, it means that the right margin of the text is fixed by the ruler. Otherwise the right border of the setting environment, usually a text view, is used as margin.

CONST **maxTabs**

Maximum number of tab stops that can be held by a TabArray structure. This limits the number of tab stops that can be defined for a ruler.

CONST **centerTab**, **rightTab**, **barTab**

Possible element of *Tab.type*. Set elements to be drawn from {*centerTab, rightTab, barTab*}. Tabulation is centred if centerTab IN type. It is right aligned if rightTab IN type. If neither option is set, tabulation is left aligned. Setting both options is illegal. If barTab IN type, then tab stop serves as the position of a vertical line drawn from the ruler containing this tab definition to the next ruler, or the end of text, if no further ruler follows. (The actual drawing of such a bar is left to the view displaying the text containing the ruler.)

TYPE **Tab**

Defines a single tab stop as a pair of a stop position and a tabulation type.

**stop**: INTEGER    [units]

The tab stop position, measured from the left margin of the setting context of the text, i.e., usually the view displaying the set text.

**type**: SET

Tabulation type to be used for text controlled by this stop. Options drawn from {*centerTab*, *rightTab*, and *barTab*}, as defined above.

TYPE **TabArray**

**len**: INTEGER    0 <= len <= *maxTabs*

Number of tab stops defined.

**tab**-: ARRAY 32 OF Tab    [units]    tab[0 .. len) valid

Array holding defined tab stops.

TYPE **Attributes (Stores.Store)**

EXTENSIBLE

The style of a ruler holds an attributes object.

Once created and initialized, attributes objects can no longer be modified, and hence can be freely shared among many ruler styles (*in the same domain*). Changing the attributes of a style is done by replacing the whole attributes object attached to the style; usually by a modified copy of the original attributes object. (Styles in turn may be shared by multiple rulers, causing all these rulers to change in synch when changing the attributes of the style.)

**init**-: BOOLEAN

Attributes object has been initialized and cannot be changed anymore.

**first**-, **left**-, **right**-: INTEGER    [units]

First line indentation of every following paragraph, left margin (other than for first line of a paragraph), and right margin. All measured from the left margin of the view displaying the set text.

**lead**-: INTEGER    [units]

Vertical lead space inserted before every following paragraph, unless the paragraph happens to start a new page.

**asc**-, **dsc**-: INTEGER    [units]

Lower bounds on the ascender and descender of the lines of the following paragraphs.

**grid**-: INTEGER    [units]    1 <= grid < 10000 * Ports.mm

Grid spacing (if 1: no grid). Each line of the following paragraphs is set such that its base line falls on a grid line. (Depending on the setter used, this may cause overlaps of high lines, or it may cause higher lines to be moved to the next possible grid line in order to avoid overlaps.)

**opts**-: SET

Formatting options, drawn from the set {*leftAdjust*, *rightAdjust*, *noBreakInside*, *pageBreak*, *parJoin*}. The options *noBreakInside* and *parJoin* are requests that, depending on the setter's heuristics, may or may not be followed.

**tabs**-: TabArray

PROCEDURE (a: Attributes) **Equals** (b: Attributes): BOOLEAN

NEW, EXTENSIBLE

Compare two attributes objects for attribute equality.

Pre

a.init    20

b # NIL    *not explicitly checked*

b.init    21

Post

result =  TypeOf(a) = TypeOf(b)  &  a.first = b.first  & ... &  a.tabs = b.tabs

PROCEDURE (a: Attributes) **Prop** (): Properties.Property

NEW, EXTENSIBLE

Return property list reflecting attribute values.

Pre

a.init    20

Post

result # NIL

PROCEDURE (a: Attributes) **ModifyFromProp**- (p: Properties.Property)

NEW, EXTENSIBLE

Initialize new attributes object according to the property list. (Values valid in the property list are taken from it, others are left intact.) Called by *ModifiedAttr*.

PROCEDURE (a: Attributes) **InitFromProp** (p: Properties.Property)

NEW, EXTENSIBLE

Initialize according to property list; use *dir.attr* for defaults.

Pre

~a.init    20

Post

a.init

TYPE **AlienAttributes (Attributes)**

Type of alien attributes objects, as returned by *ReadAttr*.

**store**-: Stores.Alien

The alien store enclosed by an alien attributes objects.

TYPE **Style (Models.Model)**

ABSTRACT

A style is a ruler model (and thus may be shared my multiple rulers).

**attr**-: Attributes

The attributes defining this style's formatting requests.

PROCEDURE (s: Style) **SetAttr** (attr: Attributes)

NEW, EXTENSIBLE, Operation

Set style attributes.

Pre

attr # NIL    *not explicitly checked*

attr.init    20

Post

s'.attr.Equals(attr)

    s.attr = s'.attr

~s'.attr.Equals(attr)

    s.attr = a    [ a.Equals(attr) ]

TYPE **Ruler (Views.View)**

ABSTRACT

Rulers are the standard view on styles.

**style**-: Style

The ruler's style.

PROCEDURE (r: Ruler) **InitDomain** (d: Domains.Domain)

EXTENSIBLE

Precondition added.

Pre

r.style # NIL    20

TYPE **Prop (Properties.Property)**

Properties of style attributes.

**first**, **left**, **right**, **lead**, **asc**, **dsc**, **grid**: INTEGER

**opts**: SET

**tabs**: TabArray

Property fields

Each property field corresponds to the attribute field of same name. (Property field *tabs* corresponds to (*tab*, *tabs*) in *Attributes*.)

TYPE **UpdateMsg (Models.UpdateMsg)**

Message notifying of a change of attributes attached to a certain style. The message is *domaincast* and typically interpreted by all text views which display a text that contains a ruler that shares the changed style.

**style**: Style

The style that has changed.

**oldAttr**: Attributes

The attributes that were attached to *style* previously.

TYPE **Directory**

ABSTRACT

Directory for rulers and styles.

**attr**-: Attributes    attr # NIL

Default attributes used when initializing new style.

PROCEDURE (d: Directory) **SetAttr** (attr: Attributes)

NEW, EXTENSIBLE

Set default attributes. Ignores tab settings and page break options in *attr*. (Note that preset tabs in default rulers collide with tab settings frequently added under program control. Likewise, page break options are not normally useful global defaults.)

Pre

attr # NIL    *not explicitly checked*

attr.init    20

Post

d.attr = ModifiedAttr(attr, p)

    [ p.valid = {opts, tabs},

        p.tabs.len = 0, p.opts.mask = {noBreakInside, pageBreak, parJoin}, p.opts.val = {}

    ]

PROCEDURE (d: Directory) **NewStyle** (attr: Attributes): Style

NEW, ABSTRACT

Return new style object initialized to hold attributes *attr*.

Pre

attr # NIL

    attr.init    20

Post

result # NIL

attr = NIL

    result.attr = d.attr

attr # NIL

    result.attr = attr

PROCEDURE (d: Directory) **New** (style: Style): Ruler

NEW, ABSTRACT

Return new ruler object initialized to hold given style.

Post

result # NIL

style = NIL

    result.style = d.NewStyle(NIL)

style # NIL

    result.style = style

PROCEDURE (d: Directory) **NewFromProp** (p: Prop): Ruler

NEW, EXTENSIBLE

Return new ruler object with style attributes initialized from give property list, using *d.attr* for default values.

Except for performance, equivalent to:

    VAR st: Stores.Stores; a: Attributes;

    st := Stores.Clone(d.attr); a := st(Attributnes);

    a.ModifyFrom(d.attr, p);

    **RETURN** d.New(d.NewStyle(a))

VAR **dir**-, **stdDir**-: Directory    dir # NIL, stdDir # NIL, stable stdDir = d

Directory and standard directory objects for rulers and styles.

PROCEDURE  **ReadAttr** (VAR rd: Stores.Reader; VAR a: Attributes)

Read an attributes object; in case the reader returns an alien store, the store is wrapped into an alien attributes object and the wrapper is returned.

Pre

~rd.rider.eof    20

NextStore(rd) IS Attributes    21

Post

a # NIL

~MapsToAlien(NextStore(rd'))

    a = NextStore(rd')

MapsToAlien(NextStore(rd'))

    a IS AlienAttributes

    a.store = NextStore(rd')

PROCEDURE  **WriteAttr** (VAR wr: Stores.Writer; a: Attributes)

Read an attributes object; in case the reader returns an alien store, the store is wrapped into an alien attributes object and the wrapper is returned.

Pre

a # NIL    20

Except for performance, equivalent to:

    WITH a: AlienAttributes DO wr.WriteStore(a.store) ELSE wr.WriteStore(a) END

PROCEDURE **ModifiedAttr** (a: Attributes; p: Properties.Property): Attributes

Return copy of *a* with attribute settings identical to those in *a*, except where overridden by *p*.

Except for performance, equivalent to:

    VAR h: Attributes;

    h := Stores.CopyOf(a)(Attributes); h.ModifyFromProp(p);

    RETURN h

PROCEDURE **SetCentered** (r: Ruler)

PROCEDURE **SetJustified** (r: Ruler)

PROCEDURE **SetLeftFlush** (r: Ruler)

PROCEDURE **SetRightFlush** (r: Ruler)

Operation

Set ruler's text formatting mode by setting r.style.attr.opts * {*leftAdjust*, *rightAdjust*} to {}, {*leftAdjust*, *rightAdjust*}, {*leftAdjust*}, or {*rightAdjust*}, respectively.

Pre

r.style # NIL    20

PROCEDURE **SetNoBreakInside** (r: Ruler)

PROCEDURE **SetPageBreak** (r: Ruler)

PROCEDURE **SetParJoin** (r: Ruler)

Operation

Set ruler's page breaking options by including *noBreakInside*, *pageBreak*, or *parJoin* into r.style.attr.opts, respectively.

Pre

r.style # NIL    20

PROCEDURE **SetFirst** (r: Ruler; x: INTEGER)

PROCEDURE **SetLeft** (r: Ruler; x: INTEGER)

PROCEDURE **SetLead** (r: Ruler; h: INTEGER)

PROCEDURE **SetAsc** (r: Ruler; h: INTEGER)

PROCEDURE **SetDsc** (r: Ruler; h: INTEGER)

PROCEDURE **SetGrid** (r: Ruler; h: INTEGER)

Operation

Set ruler's formatting attributes by setting r.style.attr.first, left, lead, asc, dsc, or grid, respectively.

Pre

r.style # NIL    20

PROCEDURE **SetRight** (r: Ruler; x: INTEGER)

PROCEDURE **SetFixedRight** (r: Ruler; x: INTEGER)

Operation

Set ruler's right margin by setting r.style.attr.right = x and setting (*SetFixedRight*) or clearing (*SetRight*) r.attr.opts * {*rightFixed*}.

Pre

r.style # NIL    20

PROCEDURE **AddTab** (r: Ruler; x: INTEGER)

Operation

Add a new tab to the right of the existing tabs set in *r*.

Pre

r.style # NIL    20

r.style.attr.tabs.len = i

    i < maxTabs    21

    i > 0

        r.style.attr.tabs.tab[i - 1].stop < x    22

Post

r.style.attr.tabs.len = r.style'.attr.tabs.len + 1

r.style.attr.tabs.tab[r.style.attr.tabs.len - 1].stop = x

PROCEDURE **MakeBarTab** (r: Ruler)

Operation

Change the last tab of *r* to a bar tab.

Pre

r.style # NIL    20

r.style.attr.tabs.len > 0    21

Post

barTab IN r.style.attr.tabs.tab[r.style.attr.tabs.len - 1].type

PROCEDURE **MakeCenterTab** (r: Ruler)

Operation

Change the last tab of *r* to a center tab.

Pre

r.style # NIL    20

r.style.attr.tabs.len > 0    21

Post

r.style.attr.tabs.tab[r.style.attr.tabs.len - 1].type * {centerTab, rightTab} = centerTab

PROCEDURE **MakeRightTab** (r: Ruler)

Operation

Change the last tab of *r* to a bar tab.

Pre

r.style # NIL    20

r.style.attr.tabs.len > 0    21

Post

r.style.attr.tabs.tab[r.style.attr.tabs.len - 1].type * {centerTab, rightTab} = righTab

PROCEDURE **CopyOf** (r: Ruler; shallow: BOOLEAN): Ruler

Create [shallow] copy of ruler.

Pre

r # NIL    20

Except for performance, equivalent to:

    VAR v: Views.View;

    v := Views.CopyOf(r, shallow); **RETURN** v(Ruler)

PROCEDURE **GetValidRuler** (text: TextModels.Model; pos, hint: INTEGER;

                                                    VAR ruler: Ruler; VAR rpos: INTEGER);

Locate the ruler in *text* that dominates position *pos*. If a ruler position is known, it can be passed as a hint: (*ruler*, *rpos*) is first ruler before *hint*. Otherwise, pass *hint* = -1. On return, either the dominating ruler is returned as (*ruler*, *rpos*), or *ruler* and *rpos* remain unchanged.

Pre

hint < 0  OR  rpos = Pos(ruler) &  no ruler in (rpos, hint]  &  0 <= pos <= t.Length()

Post

hint < rpos <= pos  &  rpos = Pos(ruler)  &  no ruler in (rpos, pos]  OR  (ruler, rpos) unmodified

Except for performance, equivalent to:

    VAR view: Views.View; rd: TextModels.Reader;

    IF pos < text.Length() THEN INC(pos) END;    *(* let a ruler dominate its own position! *)*

    IF pos < hint THEN hint := -1 END;

    rd := text.NewReader(rd); rd.SetPos(pos);

    REPEAT

        rd.ReadPrevView(view)

    UNTIL rd.eot OR (view IS Ruler) OR (rd.Pos() < hint);

    IF (view # NIL) & (view IS Ruler) THEN

        ruler := view(Ruler); rpos := rd.Pos()

    END

PROCEDURE **Deposit**

Create new ruler and deposit in Views queue.

Except for performance, equivalent to:

    Views.Deposit(dir.New(NIL))

PROCEDURE  **SetDir** (d: Directory)

Set model directory.

Pre

d # NIL    20

d.attr # NIL    *not explicitly checked*

d.attr.init    21

Post

dir = d
