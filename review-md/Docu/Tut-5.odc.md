**5 Texts**

Graphical user interfaces have introduced graphical elements into everyday computing. While this was a very positive step towards overcoming the old-style modal textual user interfaces, texts have by no means become obsolete. Even in reports containing many illustrations, texts serve as indispensible glue.

In typical database applications, no graphical contents is manipulated. The data entered and retrieved are mostly textual, and reports to be printed usually are texts arranged in a tabular fashion.

Because of the importance of texts, this chapter demonstrates how the BlackBox Component Builder's text abstraction can be used in various ways. In several examples, we will reuse the simple database module introduced in the previous chapter.

We will work "bottom-up" by providing examples first for writing new texts, then for reading existing text, and finally for modifying existing texts. When discussing these examples, we will meet text models/carriers and riders, mappers, and auxiliary abstractions such as fonts, attributes, rulers, and others.

**5.1 Writing text**

In the BlackBox Component Builder, texts use the Carrier-Rider-Mapper design pattern (see Chapter 3). They provide text models as carriers for texts, and text writers and text readers as riders on text models. Both types are defined in the Text subsystem's core module *TextModels*. A text writer maintains a current position and current attributes, which will be used when writing the next element into the text. The full definition of a writer looks as follows:

    Writer = POINTER TO ABSTRACT RECORD

        attr-: TextModels.Attributes;

        (wr: Writer) Base (): TextModels.Model, NEW, ABSTRACT;

        (wr: Writer) Pos (): INTEGER, NEW, ABSTRACT;

        (wr: Writer) SetPos (pos: INTEGER), NEW, ABSTRACT;

        (wr: Writer) SetAttr (attr: TextModels.Attributes), NEW;

        (wr: Writer) WriteChar (ch: CHAR), NEW, ABSTRACT;

        (wr: Writer) WriteView (view: Views.View; w, h: INTEGER), NEW, ABSTRACT

    END;

Listing 5-1. Definition of TextModels.Writer

The most important procedure of a writer is *WriteChar*, it allows to write a new character at the current position. If the character position lies within an existing text (i.e., in 0 .. text.Length() - 1), the character at this text position is inserted. (This is a difference to the BlackBox files abstraction, where old data would be overwritten by the writer in this case. With texts, writers *always* insert and never overwrite.) If the writer's position is at the end of the text (i.e., at text.Length()), the character is appended to the text and thus makes the text one element longer. After *WriteChar*, the writer's text position is increased by one. The current position can be inquired by calling the writer's *Pos* function, and it can be modified by calling the *SetPos* procedure.

Since text models are capable of containing views as well as normal characters, there exists a procedure *WriteView* which allows to write a view, given an appropriate size (*w* for the width, *h* for the height).

Like most geometrical distances in BlackBox, a view's width and height are specified in so-called *universal units* of 1/36000 millimeters. This value was chosen to eliminate rounding errors for many common screen and printer resolutions. For example, an inch, a 1/300-th of an inch, a millimeter, and a desktop publishing point can be represented in units without rounding errors. Where not specified otherwise, distances can be assumed to be represented in universal units.

If no specific size for the view is desired, passing the value *Views.undefined* for the width and/or height allows the text to choose a suitable default size for the written view.

How is a writer created? Like all riders, a writer is created by a factory function of its carrier. For this purpose, a text model provides the procedure *NewWriter*, which creates a writer and connects it to the text. A writer can return its text through the *Base* function. A newly created writer is positioned at the end of the text.

Text elements don't have character codes only, they also have attributes. A writer's *attr* field contains the attributes to be used when the next character or view is written. The current attributes can be changed by calling the writer's *SetAttr* procedure.

In terms of attributes, the text subsystem of the BlackBox Component Builder supports **colors** and vertical offsets. Colors are encoded as integers (*Ports.Color*). Module *Ports* predefines the constants *black*, *grey6*, *grey12*, *grey25*, *grey50*, *grey75*, *white*, *red*, *green*, and *blue*. "grey6" means a mixture of 6% white and 94% black, which is almost black. The other grey values are increasingly lighter. Vertical offsets are measured in universal units.

Besides colors and vertical offset, a text element also has a font attribute. A font is a collection of glyphs. A glyph is a visual rendering of a character code. Different fonts may contain different glyphs for the same characters. For example, Helvetica looks different from Times and Frutiger. A font may not provide glyphs for all character codes that can be represented in Component Pascal. This is not surprising, since the *CHAR* datatype of Component Pascal conforms to the Unicode standard. Unicode is a 16-bit standard, and thus can represent over 65000 different characters. The first 256 characters are the Latin-1 characters. The first 128 characters of Latin-1 are the venerable ASCII characers.

If a font doesn't contain a particular glyph, BlackBox's font framework either returns a glyph from some other font for the same character, or it yields a generic "missing glyph" symbol; e.g., a small empty rectangle.

Font objects are defined in module *Fonts.Font*. It won't be necessary to take a closer look at this module here; it is sufficient to interpret a pointer of type *Fonts.Font* as the identification of a font with its particular typeface, style, weight, and size:

**Typeface**

This is the font family to which a font belongs; it defines a common "look and feel" for the whole family. Examples of typefaces are Helvetica, Times, or Courier. Typeface names are represented as character arrays of type *Fonts.Typeface*.

**Style**

A font may optionally be *italicized*, <u>underlined</u>, struck out or *<u>a combination thereof</u>*. Styles are represented as sets. Currently, the set elements *Fonts.italic*, *Fonts.underline* and *Fonts.strikeout* are defined.

**Weight**

A font might exist in several weights, e.g., ranging from light to normal to **bold** to black to **ultra black**. Usually, a font is either normal or bold. Weights are represented as integers. The values *Fonts.normal* and *Fonts.bold* are predefined.

**Size**

A font may be rendered in different sizes, taking into account the resolution of the used output device. Often, sizes are given in points. A point is 1/72-th of an inch. In BlackBox, font sizes are measured in universal units, which are chosen such that points (*Ports.point*) and other typographical units can be represented without round-off errors. Typical font sizes are *10 * Ports.point* or *12 * Ports.point*.

In principle, typeface, style, weight and size are independent of each other. This means that a given typeface can be combined with arbitrary styles, weights and sizes. However, from a typographical point of view, some combinations are more desirable than others, and may be optimized by the font designer and the underlying operating system's font machinery.

The text attributes font, color and vertical offset are packaged into objects of type *TextModels.Attributes*. Slightly simplified, this type looks as follows:

    Attributes = POINTER TO RECORD (Stores.Store)

        color-: Ports.Color;

        font-: Fonts.Font;

        offset-: INTEGER

    END;

Listing 5-2. Simplified definition of TextModels.Attributes

This is the type of the writer's *attr* field. It is an extension of *Stores.Store*. This means that such an object can be stored persistently in a file. Other examples of stores are *TextModels.Model* and *TextViews.View*, but not *TextModels.Writer*. A more detailed description of stores is given in Part III.

We have now seen the capabilities of a text writer, including the text attributes that it supports. The typical use of text writers is shown in the code pattern below. A code pattern is an excerpt of a procedure which shows one or a few important aspects of the objects currently discussed. Such a code fragment constitutes a recipe that should be known by a programmer "fluent" in this topic.

        VAR t: TextModels.Model; wr: TextModels.Writer; ch: CHAR;

    BEGIN

        t := TextModels.dir.New();    *(* allocate new empty text model *)*

        wr := t.NewWriter(NIL);

        *... produce ch ...*

        WHILE *condition* DO

            wr.WriteChar(ch);

            *... produce ch ...*

        END;

Listing 5-3. Code pattern for TextModels.Writer

The code pattern is not very interesting since it doesn't show how the generated text can be displayed. A text object only represents a text with its attributes. It doesn't know about how to draw this text, not even how to do text setting; i.e., how to break lines and pages such that they fit a given rectangle (view, paper size, etc.). When it is desired to make a text visible, it is necessary to provide a text view for the text object. Data objects that may be displayed in several views are generally called models. The separation of model and view is an important design pattern that was discussed in Chapter 2.

Module *TextModels* only exports an abstract record type for text models, no concrete implementation. Instead, it exports a directory object (*TextModels.dir*) which provides the necessary factory function (*TextModels.dir.New*). This indirection pattern for object creation was motivated in Chapter 2.

Working directly with text writers is inconvenient, they don't even support the writing of strings. For this reason, the following examples will use *formatters* when writing text. A formatter is a mapper (see Chapter 3) that contains a text writer and performs the mapping of higher-level symbols, such as the Component Pascal symbols, to a stream of characters that can be fed to the writer. In the next example, we we will use a formatter to write a text; a very simple "report". The text consists of the phone database contents, one line per entry. This first version of a phone database report is implemented in module *ObxPDBRep0*:

MODULE ObxPDBRep0;

    IMPORT Views, TextModels, TextMappers, TextViews, ObxPhoneDB;

    PROCEDURE **GenReport***;

        VAR t: TextModels.Model; f: TextMappers.Formatter; v: TextViews.View;

            i: INTEGER; name, number: ObxPhoneDB.String;

    BEGIN

        t := TextModels.dir.New();    *(* create empty text carrier *)*

        f.ConnectTo(t);                    *(* connect a formatter to the text *)*

        i := 0;

        ObxPhoneDB.LookupByIndex(i, name, number);

        WHILE name # "" DO

            f.WriteString(name);        *(* first string *)*

            f.WriteTab;    *                (* tab character *)*

            f.WriteString(number);    *(* second string *)*

            f.WriteLn;                        *(* carriage return *)*

            INC(i);

            ObxPhoneDB.LookupByIndex(i, name, number)

        END;

        v := TextViews.dir.New(t);    *(* create a text view for the text generated above *)*

        Views.OpenView(v)            *(* open the text view in its own window *)*

    END GenReport;

END ObxPDBRep0.

Listing 5-4. Writing the phone database using a formatter

Note that name and number are separated by a tab character (09X). Since no tab stops are defined for this text, a tab will be treated as a wide fixed-width space (whose width is a multiple of the normal space character glyph). We will later see how tab stops can be defined.

Execution of the *ObxPDBRep0.GenReport* command results in the following window:

Figure  5-5. Text editor window containing the phone database

The text in the newly opened window is a fully editable text view. It can be edited; stored as a BlackBox document; printed; and so on. If you look at the source code, you can recognize the following code pattern:

        VAR t: TextModels.Model; f: TextMappers.Formatter; v: Views.View;

    BEGIN

        t := TextModels.dir.New();

        f.ConnectTo(t);

        *... use formatter procedures to construct the text ...*

        v := TextViews.dir.New(t);

        Views.OpenView(v)

Listing 5-6. Code pattern for TextMappers.Formatter and for open a text view

This code pattern occurs in many BlackBox commands, e.g., in *DevDebug.ShowLoadedModules*. It is useful whenever you need to write tabular reports of varying length, possibly spanning several printed pages.

It also clearly shows that texts embody several design patterns simultaneously: a text object is a model which can be observed by a view (Observer pattern), it is a carrier (Carrier-Rider-Mapper pattern), and it is a container (Composite pattern).

An interesting aspect of the above code pattern is that the text carrier *t* is created first, then its contents is constructed using a formatter, and only then is a view *v* on the text carrier created. Finally, the view is opened in its own document window.

In BlackBox it is important that a model can be manipulated before there exists any view for it. In fact, it is far more efficient to create a text before a view on it is opened, because screen updates and some internal housekeeping of the undo mechanism are avoided this way, which can cause a dramatic speed difference.

BlackBox text models represent sequences of text elements. A text element is either a Latin-1 character (*SHORTCHAR*), a Unicode character (a *CHAR* *> 0FFX*) or a view (an extension of type *Views.View*). Text stretches may be attributed with font information, color and vertical offset. Since a text model may contain views, it is a *container*. In the following examples, we will see how these text facilities can be used when creating new texts. Our phone book database is used as source of the material that we want to put into texts, applying different textual representations in the various examples.

The following example demonstrates how a text can be generated out of our example database. In this text, names are written in green color. The differences to module *ObxPDBRep0* are marked with <u>underlined</u> text.

MODULE ObxPDBRep1;

    IMPORT <u>Ports,</u> Views, TextModels, TextMappers, TextViews, ObxPhoneDB;

    PROCEDURE **GenReport***;

        VAR t: TextModels.Model; f: TextMappers.Formatter; v: TextViews.View;

            i: INTEGER; name, number: ObxPhoneDB.String;

            <u>default, green: TextModels.Attributes;</u>

    BEGIN

        t := TextModels.dir.New();    *(* create empty text carrier *)*

        f.ConnectTo(t);                    *(* connect a formatter to the text *)*

        <u>default := f.rider.attr;</u>    *(* save old text attributes for later use *)*

        <u>green := TextModels.NewColor(default, Ports.green);</u>    *(* use green color *)*

        i := 0;

        ObxPhoneDB.LookupByIndex(i, name, number);

        WHILE name # "" DO

            <u>f.rider.SetAttr(green);</u>    *(* change current attributes of formatter's rider *)*

            f.WriteString(name);        *(* first string *)*

            <u>f.rider.SetAttr(default);</u>    *(* change current attributes of formatter's rider *)*

            f.WriteTab;    *                (* tab character *)*

            f.WriteString(number);    *(* second string *)*

            f.WriteLn;                        *(* carriage return *)*

            INC(i);

            ObxPhoneDB.LookupByIndex(i, name, number)

        END;

        v := TextViews.dir.New(t);    *(* create a text view for the text generated above *)*

        Views.OpenView(v)            *(* open the text view in its own window *)*

    END GenReport;

END ObxPDBRep1.

Listing 5-7. Writing the phone database using green color

Note that a formatter's rider contains a set of current attributes (*TextModels.Writer.attr*). This value includes the current color and is read-only; it can be set with the writer's *SetAttr* procedure.

Our first text examples, *ObxPDBRep0* and *ObxPDBRep1*, don't produce nice-looking output, because the phone numbers are not aligned below each other in a tabular fashion.

*ObxPDBRep1* remedies this defect by inserting a ruler at the beginning of the text. This ruler defines a tab stop, which causes all phone numbers - separated from their phone names by tabs - to line up nicely. The differences to module *ObxPDBRep0* are marked with <u>underlined</u> text.

MODULE ObxPDBRep2;

    IMPORT Ports, Views, TextModels, TextMappers, TextViews, <u>TextRulers</u>, ObxPhoneDB;

    <u>PROCEDURE WriteRuler (VAR f: TextMappers.Formatter);</u>

        CONST cm = 10 * Ports.mm;    *(* universal units *)*

        VAR ruler: TextRulers.Ruler;

    BEGIN

        ruler := TextRulers.dir.New(NIL);

        TextRulers.AddTab(ruler, 4 * cm);    *(* define a tab stop, 4 cm from the left margin *)*

        TextRulers.SetRight(ruler, 12 * cm);    *(* set right margin *)*

        f.WriteView(ruler)                            *(* a ruler is a view, thus can be written to the text *)*

    END WriteRuler;

    PROCEDURE **GenReport***;

        VAR t: TextModels.Model; f: TextMappers.Formatter; v: TextViews.View;

            i: INTEGER; name, number: ObxPhoneDB.String;

    BEGIN

        t := TextModels.dir.New();    *(* create empty text carrier *)*

        f.ConnectTo(t);                    *(* connect a formatter to the text *)*

        <u>WriteRuler(f);</u>

        i := 0;

        ObxPhoneDB.LookupByIndex(i, name, number);

        WHILE name # "" DO

            f.WriteString(name);        *(* first string *)*

            f.WriteTab;    *                (* tab character *)*

            f.WriteString(number);    *(* second string *)*

            f.WriteLn;                        *(* carriage return *)*

            INC(i);

            ObxPhoneDB.LookupByIndex(i, name, number)

        END;

        v := TextViews.dir.New(t);    *(* create a text view for the text generated above *)*

        Views.OpenView(v)            *(* open the text view in its own window *)*

    END GenReport;

END ObxPDBRep2.

Listing 5-8. Writing the phone database using a formatter and a ruler

Execution of the *ObxPDBRep2.GenReport* command results in a window with the improved presentation of the phone database. Executing Text->Show Marks causes the ruler to be displayed:

Figure  5-9. Text editor window containing the tabular phone database

A ruler is a view. A ruler on its own doesn't make sense. Like all controls, a ruler makes only sense in a container. Unlike many other controls, a ruler is an example of a control that can only function properly in a particular container; in this case the container must be a text model. You may copy a ruler into a form or other container, but it will just be passive and have no useful effect there. If it is embedded in a text model that is displayed in a text view, a ruler can influence the way in which the subsequent text is set. For example, a ruler can define tab stops, margins, and adjustment modes.

The attributes of a ruler are valid in the ruler's *scope*. The scope starts with the ruler itself and ends right before the next ruler, or at the end of the text if there follows no ruler anymore. A ruler introduces a new paragraph, possibly with its own distinct set of paragraph attributes. Inserting a paragraph character (0EX) instead of a ruler also starts a new paragraph, which inherits all paragraph attributes from the preceding ruler. Paragraph characters are the normal way to start a new paragraph; rulers are only used if particular paragraph attributes should be enforced. Note that a carriage return (0DX) doesn't start a new paragraph.

A text view contains an invisible default ruler that defines the text setting of the text before the first ruler in the text, or of all text if no ruler has been written to the text at all. Interactively, this default ruler, and default attributes, can be set with the menu commands *Text->Make Default Ruler* and *Text->Make Default Attributes*.

A ruler has many attributes that can be set. Unlike character attributes, which can be applied to arbitrary text stretches, these attributes apply to the ruler or the whole paragraph(s) in its scope.

We will discuss these attributes so that you know about ruler functionality when you need it. We won't discuss rulers in more detail though, since you can learn about the necessary details on demand if and when you need the more advanced ruler capabilities (see on-line documentation of module *TextRulers*).

**Styles**

A ruler's attributes are not directly stored in the ruler view itself. Instead, they are bundled into a separate object of type *TextRulers.Attributes*, which in turn is contained in an object of type *TextRulers.Style*.

A style is the model of one or several ruler views. If several views share the same model, the user can change the attributes of all the rulers simultaneously. For example, changing the left margin will affect all rulers bound to the same style. However, models cannot be shared across documents, which means that all rulers bound to the same style must be embedded in the same document.

*TextRulers.Attributes* are persistent collections of ruler attributes, much in the same way as *TextModels.Attributes* are persistent collections of character attributes.

    Attributes = POINTER TO EXTENSIBLE RECORD (Stores.Store)

        first-, left-, right-, lead-, asc-, dsc-, grid-: INTEGER;

        opts-: SET;

        tabs-: TextRulers.TabArray

    END;

Listing 5-10. Simplified definition of TextRulers.Attributes

In the following paragraphs, we discuss the individual attributes. For these attributes, there exist auxiliary procedures in module *TextRulers* which make it simple to set up a text ruler in the desired way: just create a new ruler with *TextRulers.dir.New(NIL)*, and then apply the auxiliary procedures that you need to obtain the correct attribute settings. An example was given above in the procedure *WriteRuler* of module *ObxPDBRep2*.

These auxiliary procedures greatly simplify setting up text rulers. For example, they hide that the attributes are actually not contained in the ruler itself, but rather in attributes objects which are used by style objects which are used by the rulers. Providing such simplified access to a complex feature is called a *facade* in the design patterns terminology [GHJV94]. Here the mechanism is particularly simple because the facade merely consists of simple procedures, there is not even a facade object that would have to be created and managed.

**Tab stops**

The *tabs* field of type *TextRulers.TabArray* contains information about tab stops. The number of tab stops is given in *tabs.len*, and must not be larger than *TextRulers.maxTabs* (which is currently 32). If the user enters more than *maxTabs* tabs, or more tabs than given in *len*, then the superfluous tabs will be treated as fixed size space characters. The fixed size used is an integer multiple of the width of a space in the used font, approximately 4 mm in total.

    Tab = RECORD

        stop: INTEGER;    *(* stop >= 0 *)*

        type: SET    *(* type IN {TextRulers.centerTab, TextRulers.rightTab, TextRulers.barTab} *)*

    END;

    TabArray = RECORD

        len: INTEGER;    *(* 0 <= len <= TextRulers.maxTabs *)*

        tab: ARRAY TextRulers.maxTabs OF TextRulers.Tab

        *    (* tab[0 .. len-1] sorted in ascending order without duplicates *)*

    END

Listing 5-11. Definition of TextRulers.Tab and TabArray

In *tabs.tab.stop*, the tab's distance from the left border of the text view is specified (in universal units of course). The tab stops in the *tabs.tab* array must be sorted in ascending order, and there must be no duplicate values.

In addition to the stop position kept in field *stop*, a tab stop can be modified using a set of possible options kept in field type. In particular, these options allow to center text under a tab stop or to right-align text under a tab stop (the default is left alignment):

**tab type    tabs.tab[i].type**

left adjusted    {}

right adjusted    {TextRulers.rightTab}

centered    {TextRulers.centerTab}

For all of the above tab types, it can optionally be specified whether the tab itself is drawn as a vertical bar. Bar tabs are specified by including the value *TextRulers.barTab* in the tab type set. Since the bars occupy the whole line height, they are useful to to format simple tables.

When created with *TextRulers.dir.New(NIL)*, a ruler has no tabs.

The following auxiliary procedures are provided to modify a ruler:

    PROCEDURE AddTab (r: Ruler; x: INTEGER)

    PROCEDURE MakeRightTab (r: Ruler)

    PROCEDURE MakeCenterTab (r: Ruler)

    PROCEDURE MakeLineTab (r: Ruler)

*AddTab* adds a new left-adjusted normal tab. It can be changed into a right-adjusted or centered tab by calling *MakeRightTab* or *MakeCenterTab*. Independently of the adjustment mode, the tab can be turned into a bar tab by calling *MakeLineTab*.

**Margins**

Text is set within a left and a right margin. These margins can be specified as distances from the left border of the text view. The left margin must be smaller than the right margin.

    left-, right-: INTEGER    *(* (left >= 0)  &  (right >= left) *)*

It is possible to signal to the text setter that it should ignore the right margin and use the text view for line breaking instead. In fact, this is the default behavior. It means that when the view size is changed, the text's line breaking will probably change also. Even if the right margins is ignored for text setting, it is still useful: a text view uses it as a hint for how wide the view should be when it is opened.

Line breaking at a fixed right margin is only needed in rare circumstances, for example when this particular paragraph should be narrower than the others. If you want line breaking to be controlled by the right margin, then you have to include *TextRulers.rightFixed* in the *opts* set (see further below).

Note that the *Tools->Document Size...* command allows to flexibly specify the width (and height) of a document's outermost view, the so-called *root view*. The width can be set to a fixed size, which is useful for page layout types of application.

Usually however, the width is defined by the page setup, which derives the view's width from the currently selected paper size, reduced by the user-specified print margins. This is adequate for letters, program sources, and similar kinds of documents.

As a third possibility, the root view's width can be bound to the window size (this is the case mentioned above, when the view uses the first ruler's right margin as a hint, whether the right margin is fixed or not). Whenever the user resizes the window, the root view is resized accordingly, such that it is kept always as large as the window allows. This is desirable in particular for on-line documentation, where the documentation should use as much screen space as granted by the user, adapting the text flow to the available width whenever the window is resized. An example for this style is the default formatting of Web pages.

Note that these features are independent of the root view's type. They work the same way for all resizable views, such as text views, form views, and most other kinds of editor view.

When created with *TextRulers.dir.New(NIL)*, a ruler has a left margin of 0 and an implementation-specific non-fixed right margin > 0.

The following auxiliary procedures are provided to modify a ruler:

    PROCEDURE SetLeft (r: Ruler; x: INTEGER)

    PROCEDURE SetRight (r: Ruler; x: INTEGER)

    PROCEDURE SetFixedRight (r: Ruler; x: INTEGER)

*SetLeft* and *SetRight* set the left or right margin. *SetFixedRight* sets the right margin and marks it as fixed.

**First line indentation**

A ruler defines the *first line indentation*, i.e., how much the first line after the ruler is indented from the left border of the text view. Note that it is possible to define a first line indentation that is smaller than the left margin (first < left). This is the only case where a text view draws text to the left of the specified left margin.

    first-: INTEGER;    *(* first >= 0 *)*

First line indentation is applied to the first line of each paragraph in the ruler's scope.

When created with *TextRulers.dir.New(NIL)*, a ruler has no first line indentation (first = 0).

The following auxiliary procedure is provided to modify a ruler:

    PROCEDURE SetFirst (r: Ruler; x: INTEGER)

**Ascender and descender**

When a character is drawn, its rendering depends on its font attributes, such as typeface (Helvetica or Times?), style (straight or slanted?), weight (normal or bold?) and size. When a whole string is drawn, all characters are placed on a common base line. The ascender is the largest extent to which any character of a line extends above the base line. Some characters extend below the base line, such as "g" and "y". The descender is the largest extent to which any character of a line extends below the base line (see Figure 5-12).

A text setter should calculate base line distances in a way that one line's descender doesn't overlap the next line's ascender, in order to avoid overlapping characters. In order to avoid degenerate cases, such as empty lines or lines that only contain tiny font sizes, a ruler can specify minimal values for ascender (asc) and descender (dsc).

    asc-, dsc-: INTEGER;    *(* (asc >= 0)  &  (dsc >= 0) *)*

Figure 5-12. Ascender, descender, and base line

When created with *TextRulers.dir.New(NIL)*, a ruler uses the current default font's (*Fonts.dir.Default()*) ascender and descender as initial values.

The following auxiliary procedures are provided to modify a ruler:

    PROCEDURE SetAsc (r: Ruler; h: INTEGER)

    PROCEDURE SetDsc (r: Ruler; h: INTEGER)

**Paragraph lead**

The paragraph lead defines the additional vertical space to be inserted between the first line of a paragraph and the previous paragraph's last line. Where the ruler's grid attribute (see below) is used to define a line grid, it is normally considered good style to choose a lead that is a multiple of the grid spacing, or sometimes half of the grid spacing. It allows to visually distinguish new paragraphs from mere carriage returns.

    lead-: INTEGER;    *(* lead >= 0 *)*

When created with *TextRulers.dir.New(NIL)*, a ruler has a lead of 0.

The following auxiliary procedure is provided to modify a ruler:

    PROCEDURE SetLead (r: Ruler; h: INTEGER)

**Line grid**

If *grid* is zero, then lines are packed as closely as possible, so that there remains no free room between one line's descender and the next line's ascender. Different lines may have different ascenders and descenders, depending on the text and fonts that they contain. For this reason, distances between base lines need not be regular. For typographical reasons, irregular line spacing is often undesirable.

To ensure a more regular spacing of grid lines, it is possible to switch on a line grid, by setting *grid* to a value larger than zero. This value defines a vertical grid, onto which all base lines are forced. If a line is too high to fit on the next grid line, it is placed one or more grid distances further below (see Figure 5-14).

    grid-: INTEGER;    *(* grid >= 0 *)*

Figure 5-13. Line grid

When created with *TextRulers.dir.New(NIL)*, a ruler has a line grid of 1.

The following auxiliary procedure is provided to modify a ruler:

    PROCEDURE SetGrid (r: Ruler; h: INTEGER)

**Adjustment modes**

For longer text stretches, it becomes necessary to break lines that don't fit between the text view's margins. This text setting may occur in several different ways. Text may be left adjusted (left flush), right adjusted (right flush), centered, or aligned to both left and right margins (full justification). These four adjustment modes can be controlled by two option flags: *TextRulers.leftAdjust* and *TextRulers.rightAdjust*.

    **leftAdjust    rightAdjust    adjustment mode**

    FALSE    FALSE    centered

    FALSE    TRUE    right adjusted

    TRUE    FALSE    left adjusted (default)

    TRUE    TRUE    fully adjusted

Table 5-14. Adjustment modes

These adjustment modes can be included in or excluded from a more comprehensive option set, which is represented as a *SET*:

    opts-: SET    *(* opts is subset of {leftAdjust, rightAdjust, pageBreak, rightFixed, noBreakInside, parJoin; the other set elements are reserved *)*

When created with *TextRulers.dir.New(NIL)*, a ruler's option set is *{leftAdjust}*.

The following auxiliary procedures are provided to modify a ruler:

    PROCEDURE SetLeftFlush (r: Ruler)

    PROCEDURE SetCentered (r: Ruler)

    PROCEDURE SetRightFlush (r: Ruler)

    PROCEDURE SetJustified (r: Ruler)

**Page breaks**

A ruler can force a page break, by including the *TextRulers.pageBreak* option element in the *opts* set.

The element *TextRulers.noBreakInside* is valid for the entire scope of a paragraph. As the name indicates, it forces the beginning of the paragraph to start on a new page, if this isn't the case anyway and if the page would be broken before the next paragraph.

The element *TextRulers.parJoin* is similar to *noBreakInside*, except that it additionally forces at least the first line of the next paragraph onto the same page.

When created with *TextRulers.dir.New(NIL)*, a ruler has none of the above special attributes.

The following auxiliary proceduresare provided to modify a ruler:

    PROCEDURE SetPageBreak (r: Ruler)

    PROCEDURE SetNoBreakInside (r: Ruler)

    PROCEDURE SetParJoin (r: Ruler)

This concludes our discussion of ruler attributes. Ruler attributes apply to the ruler, or to all the paragraphs in the ruler's scope. In contrast, *character attributes* apply to arbitrary stretches of characters. The BlackBox text model supports all font attributes plus color and vertical offsets.

Rulers are views that can be inserted into a text, since texts are containers. Rulers are special in that they are known to the text system, since a text view needs to know about a ruler's paragraph attributes when setting the text. However, arbitrary other views, which are not known to the text system, may be inserted into a text. Particularly interesting effects can be achieved when these views know about texts. Fold views and link views are examples of such text-aware views.

The following example shows how fold views can be generated. Fold views are standard objects of the BlackBox Component Builder; they are implemented in the *StdFolds* module, only using BlackBox and its Text subsystem. Fold views come in pairs: a left fold and a right fold. The left fold contains a hidden text. When the user clicks on one of the fold views, the text between the two views is swapped with the hidden text. Typically, fold views are used to hide a large text behind a short one; e.g., a book chapter may be hidden, while the chapter's name is visible. This is a way to achieve abstraction of document parts, by hiding details. For this reason, one state of a fold view is called its *collapsed* state, and the other is called its expanded *state*. Technically there is no reason why the hidden text should be longer than the visible one; the naming is more a reflection of the typical use of folds.

Figure 5-15. Fold views in collapsed and expanded states

Figure 5-16. Structure of text folds

MODULE ObxPDBRep3;

    IMPORT Views, TextModels, TextMappers, TextViews, <u>StdFolds</u>, ObxPhoneDB;    <u>                </u>

<u>    PROCEDURE WriteOpenFold (VAR f: TextMappers.Formatter;</u>

<u>                                                            IN shortForm: ARRAY OF CHAR);</u>

        VAR fold: StdFolds.Fold; t: TextModels.Model;

    BEGIN

        t := TextModels.dir.NewFromString(shortForm);    *(* convert a string into a text model *)*

        fold := StdFolds.dir.New(StdFolds.expanded, "", t);

        f.WriteView(fold)

    END WriteOpenFold;

    <u>PROCEDURE WriteCloseFold (VAR f: TextMappers.Formatter);</u>

        VAR fold: StdFolds.Fold; len: INTEGER;

    BEGIN

        fold := StdFolds.dir.New(StdFolds.expanded, "", NIL);

        f.WriteView(fold);

        fold.Flip;    *(* swap long-form text, now between the two fold views, with hidden short-form text *)*

        len := f.rider.Base().Length();    *(* determine the text carrier's new length *)*

        f.SetPos(len)    *(* position the formatter to the end of the text *)*

    END WriteCloseFold;

    PROCEDURE **GenReport***;

        VAR t: TextModels.Model; f: TextMappers.Formatter; v: TextViews.View;

            i: INTEGER; name, number: ObxPhoneDB.String;

    BEGIN

        t := TextModels.dir.New();    *(* create empty text carrier *)*

        f.ConnectTo(t);                    *(* connect a formatter to the text *)*

        i := 0;

        ObxPhoneDB.LookupByIndex(i, name, number);

        WHILE name # "" DO

            <u>WriteOpenFold(f, name$);</u>    *(* write left fold view into text, with *name* as its short-form text *)*

            *(* now write the long-form text *)*

            f.WriteString(name);        *(* first string *)*

            f.WriteTab;    *                (* tab character *)*

            f.WriteString(number);    *(* second string *)*

            <u>WriteCloseFold(f);</u>            *(* write closing fold, and swap short- and long-form texts *)*

            f.WriteLn;                        *(* carriage return *)*

            INC(i);

            ObxPhoneDB.LookupByIndex(i, name, number)

        END;

        v := TextViews.dir.New(t);    *(* create a text view for the text generated above *)*

        Views.OpenView(v)            *(* open the text view in its own window *)*

    END GenReport;

END ObxPDBRep3.

Listing 5-17. Writing the phone database using folds

Procedure *WriteCloseFold* has two interesting properties. First, the fold is written in its expanded form. This is reasonable, since the expanded form is often more complex than the collapsed form, which thus can be used conveniently in *WriteOpenFold* (note the useful text directory procedure *TextModels.dir.NewFromString*). The collapsed version is more complex, and thus it is convenient to use the already existing text formatter. After the left fold view, the expanded text, and the right fold view have been written, the text is collapsed using the *StdFolds.Flip* procedure.

This leads us to the second interesting point. Collapsing the text modifies the text model. This invalidates all riders, and consequently all formatters, that operate on the text. For example, formatter *f* has always been at the end of the created text, but now that the text is collapsed, its position is beyond the end of the text and thus illegal. For this reason, the correct position is set up again.

This is a general observation when working with mutable carriers: whenever you modify a carrier's state (or whenever someone else may do so!) you possibly have to update some state of your riders. In this example, the text's length was modified, which made it necessary to update the formatter's position.

The last example in our series of demonstrations on how to generate texts uses link views. Link views are standard objects of the BlackBox Component Builder; they are implemented in the *StdLinks* module, only using BlackBox and its Text subsystem. Like fold views, link views are used in pairs. Their implementation uses a special interface of the text subsystem (*TextControllers.FilterPollCursorMsg*, *TextControllers.FilterTrackMsg*) to cause the cursor to turn into a hand cursor when it lies above the text stretch between the pair of link views. This behavior is known from typical hypertext systems, such as Web browsers, when the mouse is moved over a text stretch that symbolizes a hyperlink. This use of link views is what gave them their name. However, they are much more general than simple hyperlink delimiters. A hyperlink contains a passive reference to another document or to another location in the same document. For example, a document reference may be stored as a string, such as "http://www.oberon.ch" or "BlackBox/System/Rsrc/About".

Link views in contrast store a Component Pascal command, optionally with one or two string literals as parameters. The command is executed when the user clicks the mouse button when the mouse is between the two link views.

The command is stored in the left link view of a pair. Typical commands are:

    DevDebug.ShowLoadedModules

    StdCmds.OpenDoc('System/Rsrc/Menus')

    StdCmds.OpenAux('System/Rsrc/About', 'About BlackBox')

In our example, we will create link views with commands similar to the following one:

    ObxPDBRep4.Log('Daffy Duck    310-555-1212')

Note: consult the documentation of module *StdInterpreter*, since there may be some restrictions on the possible parameter lists supported for such commands.

The *Log* procedure of our example module simply writes the parameter string to the log. In a more realistic example, the parameter string could be used to dial the respective number.

MODULE ObxPDBRep4;

    IMPORT Views, TextModels, TextMappers, TextViews, <u>StdLinks</u>, <u>StdLog,</u> ObxPhoneDB;

    CONST cmdStart = "ObxPDBRep4.Log('"; cmdEnd = "')";

    PROCEDURE **GenReport***;

        VAR t: TextModels.Model; f: TextMappers.Formatter; v: TextViews.View;

            i: INTEGER; name, number: ObxPhoneDB.String; <u>link: StdLinks.Link;</u>

    BEGIN

        t := TextModels.dir.New();    *(* create empty text carrier *)*

        f.ConnectTo(t);                    *(* connect a formatter to the text *)*

        i := 0;

        ObxPhoneDB.LookupByIndex(i, name, number);

        WHILE name # "" DO

            <u>link := StdLinks.dir.NewLink(cmdStart + name + "    " + number + cmdEnd);</u>

            <u>f.WriteView(link);</u>

            f.WriteString(name);        *(* the string shown between the pair of link views *)*

            <u>link := StdLinks.dir.NewLink("");</u>

<u>            f.WriteView(link);</u>

            f.WriteLn;                        *(* carriage return *)*

            INC(i);

            ObxPhoneDB.LookupByIndex(i, name, number)

        END;

        v := TextViews.dir.New(t);    *(* create a text view for the text generated above *)*

        Views.OpenView(v)            *(* open the text view in its own window *)*

    END GenReport;

    <u>PROCEDURE </u>**<u>Log</u>**<u>* (param: ARRAY OF CHAR);</u>

    BEGIN

        StdLog.String(param); StdLog.Ln

    END Log;

END ObxPDBRep4.

Listing 5-18. Writing the phone database using links

Note that commands are normal procedure calls; like all procedure calls from outside of a module, they can only call those procedures of the module that are exported. Thus procedure *Log* of the above module must be exported, otherwise the program wouldn't work.

Another interesting property of link views is that they display themselves in different ways depending on whether the text view via which the user interacts with them displays or hides marks. If there are several text views on the same model, the user can independently switch marks on or off, such as inactive soft hyphens, ruler views or link views. The text subsystem provides an interface which allows views to distinguish between the two states (*TextSetters.Pref*). In particular, a view may decide to hide itself when no marks are shown, by setting its width to zero in this case. This is what link views do: when text marks are switched off, they are invisible, so that only the text between them is visible. Typically, this text is blue and underlined, to indicate that it represents a link. A ruler in contrast always occupies the whole width between its paragraph's left and right margin, but it sets its height to zero when the text marks are turned off. We will learn more about how containers and embedded views can cooperate to achieve such effects in Part III of this tutorial.

The above example demonstrates that link views can cause arbitrary behavior when the user clicks between them. This generality is powerful, but it should be used with caution. A command in a document is a kind of "implicit import" relation between the document and the command's module, and thus constitutes a dependency. This is not fundamentally different from the dependencies that are caused by views embedded in the document, they also function correctly only when their implementing component(s) are available. Like all dependencies, those caused by links, buttons embedded in a control, editors embedded in documents, and so on also need to be tracked and updated when necessary.

**5.2 The Model-View-Controller pattern applied to texts**

The next major examples demonstrate how existing text can be read. It is assumed that the text to be processed is visible, in the topmost window. On how to access texts that are stored in files, please refer to the on-line Obx examples *ObxOpen0*, *ObxOpen1*, and *ObxAscii*. Before starting with the specifics of text reading, we need to discuss how the topmost window's contents can be accessed from within a Component Pascal command.

Basically, this is done by calling *Controllers.FocusView()*. This function returns *NIL* if no document is open. Otherwise it returns the topmost window's contents, which is a view.

We have learned earlier that views, whose common base type is *Views.View*, are *the* pivotal BlackBox abstractions for interactive components, such as editors and controls. Everything revolves around views. In simple cases, such as controls, a view implementation only implements an extension to *Views.View*, and that's all. In more complex cases, the data that are visualized by a view are split off into a separate model. This was discussed in detail in Chapter 2.

Since container implementations are among the most complex programming tasks for which the BlackBox Component Builder was designed, the framework provides some help in how a container should be constructed. The abstract design of a container is given in module *Containers*. This module declares extensions of the types *Views.View*, *Models.Model* and *Controllers.Controller*; these types constitute a mini-framework for container construction.

Typical containers are implemented in at least four modules: one for the model, one for the view, one for the controller, and one for standard commands. For example, the text subsystem contains the modules *TextModels*, *TextViews*, *TextControllers*, *TextCmds*; plus the auxiliary modules *TextMappers*, *TextRulers* and *TextSetters*. The form subsystem consists of *FormModels*, *FormViews*, *FormControllers*, and *FormCmds*.

Figure 5-19. Variations of the generic container module decomposition

We won't look at all the details of these types; but it is important to know that a container object is not only split into a model and one or several views, but that a view is further split into a view and a so-called controller. For the time being, we can regard the management of selections and carets as the main functionality that a controller encapsulates. This is why we now look at controllers, because some of the following examples will operate on the current selection.

Once you have a controller, you can obtain its view by calling its *ThisView* procedure. Other procedures allow to get or set the current selection.

There may be several views displaying the same model (many-to-one relationship), but there is exactly one controller per view (one-to-one relationship). (This holds for views derived from *Containers.View*; other views may or may not have a separate controller.)You can get a controller's view by calling its *ThisView* procedure, and you can get a container view's controller by calling its *ThisController* procedure.

Figure 5-20. Model-View-Controller separation

More specific container abstractions typically provide a suitable *Focus* function which yields the controller of the currently focused view, if there is one and if it has the desired type.

We now know enough about the container scenario that we can tell how to access focus view, model, or controller:

    The following declarations are assumed:

        VAR v: Views.View; m: Models.Model; c: Controllers.Controller;

    Simple view:

        v := Controllers.FocusView();

    View with a model:

        v := Controllers.FocusView();

        IF v # NIL THEN m := v.ThisModel() ELSE m := NIL END;

    Extension of a *Containers.View*:

        v := Controllers.FocusView();

        IF v # NIL THEN m := v.ThisModel() ELSE m := NIL END;

        IF (v # NIL) & (v IS Containers.View) THEN

            c := v(Containers.View).ThisController()

        ELSE

            c := NIL

        END

Listing 5-21. Accessing focus view, model, and controller

Typical container abstractions, such as the Text and Form subsystems, use the above mechanisms to provide more convenient access functions tailored to the specific container type. For example, module *TextControllers* exports the function *Focus*, which returns the focus view's controller *if and only if* the focus view is a text view. Otherwise it returns *NIL*. This approach is appropriate, since once you have a container controller, you can easily access its view, its model, information about its selection or caret, and so on. The typical code pattern that arises is some subset of the following one, depending on the controller's parts that you need:

    PROCEDURE **SomeCommand***;

        VAR c: TextControllers.Controller;

            v: TextViews.View; t: TextModels.Model; from, to, pos: INTEGER;

            ...

    BEGIN

        c := TextControllers.Focus();

        IF c # NIL THEN

            ...

            v := c.ThisView();    *(* if you need the text view... *)*

            ...

            t := v.ThisModel();    *(* or if you need the text model... *)*

            ...

            IF c.HasSelection() THEN    *(* or if you need the selection range... *)*

                c.GetSelection(from, to);

                ...

            END;

            ...

            IF c.HasCaret() THEN    *(* or if you need the caret position... *)*

                pos := c.CaretPos();

                ...

            END;

            ...

        ELSE    *(* no open document window, or focus has wrong type *)*

*            (* no error handling is necessary if this command is guarded by appropriate guard command *)*

        END

    END SomeCommand;

Listing 5-22. Code pattern for TextControllers.Controller

A menu command can use the guard *TextCmds.FocusGuard* to make sure that the command is only executed when a text view is focus. *TextCmds.SelectionGuard* additionally checks whether there exists some (non-empty) text selection.

**5.3 Reading text**

Now that we have seen how to access the focus views' controller, we can write our first command that reads the focus view's text. The command counts the number of Latin-1 characters (1 byte), Unicode characters above Latin-1 (2 byte), and embedded views.

MODULE ObxCount0;

    IMPORT TextModels, TextControllers, StdLog;

    PROCEDURE **Do***;

    *(** use TextCmds.SelectionGuard as guard for this command **)*

        VAR c: TextControllers.Controller; from, to, schars, chars, views: INTEGER;

            rd: TextModels.Reader;

    BEGIN

        c := TextControllers.Focus();

        IF (c # NIL) & c.HasSelection() THEN

            c.GetSelection(from, to);    *(* get selection range; from < to *)*

            rd := c.text.NewReader(NIL);    *(* create a new reader for this text model *)*

            rd.SetPos(from);    *(* set the reader to beginning of selection *)*

            rd.Read;                *(* read the first element of the text selection *)*

            schars := 0; chars := 0; views := 0;    *(* counter variables *)*

            WHILE rd.Pos() # to DO    *(* read all elements of the text selection *)*

                IF rd.view # NIL THEN    *(* element is a view *)*

                    INC(views)

                ELSIF rd.char < 100X THEN    *(* element is a Latin-1 character *)*

                    INC(schars)

                ELSE    *(* element is Unicode character *)*

                    INC(chars)

                END;

                rd.Read        *(* read next element of the text selection *)*

            END;

            StdLog.String("Latin-1 characters: "); StdLog.Int(schars); StdLog.Ln;

            StdLog.String("Unicode characters: "); StdLog.Int(chars); StdLog.Ln;

            StdLog.String("Views: "); StdLog.Int(views); StdLog.Ln;

            StdLog.Ln

        END

    END Do;

END ObxCount0.

Listing 5-23. Counting Latin-1 characters, Unicode characters, and views in a text

The above example uses a *TextModels.Reader*, which is the text rider type for reading. Each reader maintains a current position on its base text; there can be several independent readers on the same text simultaneously. A text reader has the following definition:

TYPE

    Reader = POINTER TO ABSTRACT RECORD

        eot: BOOLEAN;

        attr: TextModels.Attributes;

        char: CHAR;

        view: Views.View;

        w, h: INTEGER;

        (rd: Reader) Base (): TextModels.Model, NEW, ABSTRACT;

        (rd: Reader) Pos (): INTEGER, NEW, ABSTRACT;

        (rd: Reader) SetPos (pos: INTEGER), NEW, ABSTRACT;

        (rd: Reader) Read, NEW, ABSTRACT;

        (rd: Reader) ReadChar (OUT ch: CHAR), NEW, ABSTRACT;

        (rd: Reader) ReadView (OUT v: Views.View), NEW, ABSTRACT;

        (rd: Reader) ReadRun (OUT attr: TextModels.Attributes), NEW, ABSTRACT;

        (rd: Reader) ReadPrev, NEW, ABSTRACT;

        (rd: Reader) ReadPrevChar (OUT ch: CHAR), NEW, ABSTRACT;

        (rd: Reader) ReadPrevView (OUT v: Views.View), NEW, ABSTRACT;

        (rd: Reader) ReadPrevRun (OUT attr: TextModels.Attributes), NEW, ABSTRACT

    END;

Listing 5-24. Definition of TextModels.Reader

Given a text model *text*, a reader can be created by calling *rd := text.NewReader(NIL)*. Such a newly allocated reader will be positioned at the beginning of the text; i.e., at *rd.Pos() = 0*. The position can be changed whenever necessary by calling *rd.SetPos(pos)*, with *pos* in the range of *0..text.Length()*. The reader's text model can be obtained by calling its *Base* procedure.

Reading of the next text element is done by calling the reader's *Read* procedure, which increases the reader's current position unless the end of text has been reached, in which case the reader's *eot* field is set. After *Read* is called, the character code of the read element can be found in the reader's *char* field. If the character value is less than *100X*, the character fits in the Latin-1 range (1-byte character), otherwise it's a 2-byte Unicode character. For convenience, the auxiliary reader procedure *ReadChar* is provided, which performs a *Read* and returns the contents of *char.*

What happens if a view is read? In most cases, *char* is set to the reserved value *TextModels.viewcode* (2X). However, this is not necessarily the case. A text-aware view may choose to represent an arbitrary character code by using a special preference message (*TextModels.Pref*). In order to find out whether a view has been read, the field *view* should be tested. It returns the view just read, or *NIL* if it wasn't a view.

When a view has been read, the fields *w* and *h* indicate the size of the view in universal units. Otherwise, these fields are undefined.

It is expected that the implementation of a text model optimizes access to views. For this purpose, the auxiliary reader procedure *ReadView* is provided, which reads the next view after the current position, skipping as many non-view elements as necessary.

After an element (character or view) has been read, its text attributes are returned in the reader's *attr* field. The text attributes consist of font attributes, color and vertical offset; as described earlier. For efficiency and convenience reasons, the auxiliary reader procedure *ReadRun* is provided, which advances the reader position until the attributes change from their current setting. Text runs that have the same attributes can be drawn on the screen with one port procedure (*Ports.Frame.DrawString*), which is considerably faster than drawing them character by character.

Normally, text is read forward; from lower positions to higher positions. The reader procedures *ReadPrev*, *ReadPrevView* and *ReadPrevRun* are symmetrical to their *Read*, *ReadView* and *ReadRun* counterparts, but they read backwards. In particular, *ReadPrevView* is used by a text view's so-called text setter to find the most recent ruler view, which defines how the text must be set (The auxiliary procedure *TextRulers.GetValidRuler* implements this search.)

When one of the reading procedures has been called (whether one of the forward or one of the backward reading procedures), the reader's *eot* field is set. If an element could be read, *eot* is set to *FALSE*. If no element could be read (reading forward at the end of the text, or reading backward at the beginning of the text), *eot* is set to *TRUE*.

Typically, the following code pattern arises when working with a text reader. *condition* may be something like *rd.Pos() # end* or *~rd.eot*.

        VAR rd: TextModels.Reader; start: INTEGER;

    BEGIN

        *... define *start* ...*

        rd := text.NewReader(NIL);

        rd.SetPos(start);    *(* only necessary if *start # 0* *)*

        rd.Read;

        WHILE *condition* DO

            *... consume text element denoted by rd ...*

            rd.Read

        END;

Listing 5-25. Code pattern for TextModels.Reader

The next example uses a text scanner. Scanners are reading text mappers; like formatters, they are defined in module *TextMappers*. Text scanners are similar to text readers, except that they operate on whole symbols rather than characters. Strings and numbers are examples of Component Pascal symbols that are supported by text scanners.

When you compare the definition of a *TextMappers.Scanner* with the one of *TextModels.Reader*, you'll note that a scanner can also return characters (field *char*) and views with their sizes (fields *view*, *w*, and *h*); that a scanner has a position (*Pos*, *SetPos*), but that a scanner cannot read backwards. *Scan* corresponds to a reader's *Read* procedure. It skips white space (blanks, carriage returns) until it either reaches the end of the text, which is signaled by setting *type* to *TextMappers.eot*, or until it has read a symbol. The type of symbol that was read is returned in *type*. It may be one of the *TextMappers* constants

    *char, string, int, real, bool, set, view, tab, line, para, eot, invalid. *

By default, views and control characters are treated as white space and thus ignored. Using an option set (*opts*, *SetOpts*) it is possible to treat them as valid symbols, too. The set element *TextMappers.returnViews* indicates that views should be recognized, while *TextMappers.returnCtrlChars* indicates that the three control characters for tabulators (*tab*), carriage returns (*line*) and paragraphs (*para*) should be recognized.

When a symbol starts like a Component Pascal string or identifier, the scanner attempts to read a string. When successful, *type* is set to *string*; which may also be a short string, i.e., only containing Latin-1 characters. The option *TextMappers.returnQualIdents* lets the scanner recognize complete qualified identifiers as single symbols; e.g., the string  *ThisMod.ThatObj*  . The length of the string is returned in *len*.

If it is possible to interpret the symbol as an integer or real number, *type* is set accordingly, as are the fields *int* or *real*. Integer numbers are recognized in different number bases, the base that was recognized is returned in *base* (10 for decimal numbers).

Special characters such as  *!@#$*  are interpreted as *type = TextMappers.char*, which may also be a short character, i.e., a Latin-1 character.

By default, Boolean values and sets are not interpreted, except if the option elements *TextMappers.interpretBool* or *TextMappers.interpretSets* are set. Sets use the normal Component Pascal syntax, while Booleans are represented as *$TRUE* and *$FALSE*.

The scanner field *start* is the position of the first character of the most recently read symbol. The fields *lines* and *paras* count the number of carriage return and paragraph characters since the last *SetPos* (see below).

The scanner's *Skip* procedure is an auxiliary procedure that advances the scanner position until all white space characters - as specified in the *opts* set - are skipped and the first character of the symbol has been read. This character is returned in *Skip*'s *ch* parameter.

Since there may be an arbitrary number of specialized text mappers, in addition to the ones provided by *TextMappers*, the text model cannot create text mapper objects, like it creates text readers and writers. Instead, scanners (and formatters likewise) are connected to their texts explicitly, by calling their *ConnectTo* procedures. They may be called repeatedly to connect the same scanner to different models.

    Scanner = RECORD

        rider-: TextModels.Reader;

        opts-: SET;

        type: INTEGER;

        start, lines, paras: INTEGER;

        char: CHAR;

        int, base: INTEGER;

        real: REAL;

        bool: BOOLEAN;

        set: SET;

        len: INTEGER;

        string: TextMappers.String;

        view: Views.View;

        w, h: INTEGER;

        (VAR s: Scanner) ConnectTo (text: TextModels.Model), NEW;

        (VAR s: Scanner) Pos (): INTEGER, NEW;

        (VAR s: Scanner) SetPos (pos: INTEGER), NEW;

        (VAR s: Scanner) SetOpts (opts: SET), NEW;

        (VAR s: Scanner) Scan, NEW;

        (VAR s: Scanner) Skip (OUT ch: CHAR), NEW

    END;

Listing 5-26. Definition of TextModels.Scanner

The following example uses a text scanner to count the number of integer numbers, real numbers, and strings found in a text.

MODULE ObxCount1;

    IMPORT TextMappers, TextControllers, StdLog;

    PROCEDURE **Do***;

    *(** use TextCmds.SelectionGuard as guard for this command **)*

        VAR c: TextControllers.Controller; from, to, ints, reals, strings: INTEGER;

            s: TextMappers.Scanner;

    BEGIN

        c := TextControllers.Focus();

        IF (c # NIL) & c.HasSelection() THEN

            c.GetSelection(from, to);    *(* get selection range; from < to *)*

            s.ConnectTo(c.text);    *(* connect scanner to this text model *)*

            s.SetPos(from);    *(* set the reader to beginning of selection *)*

            s.Scan;                *(* read the first symbol of the text selection *)*

            ints := 0; reals := 0; strings := 0;    *(* counter variables *)*

            WHILE s.start < to DO    *(* read all symbols starting in the text selection *)*

                IF s.type = TextMappers.int THEN    *(* symbol is an integer number *)*

                    INC(ints)

                ELSIF s.type = TextMappers.real THEN    *(* symbol is a real number *)*

                    INC(reals)

                ELSIF s.type = TextMappers.string THEN    *(* symbol is a string/identifier *)*

                    INC(strings)

                END;

                s.Scan        *(* read next symbol of the text selection *)*

            END;

            StdLog.String("Integers: "); StdLog.Int(ints); StdLog.Ln;

            StdLog.String("Reals: "); StdLog.Int(reals); StdLog.Ln;

            StdLog.String("Strings: "); StdLog.Int(strings); StdLog.Ln;

            StdLog.Ln

        END

    END Do;

END ObxCount1.

Listing 5-27. Counting integers, reals, and strings in a text

The most typical code pattern for a scanner looks similar to the code pattern for a reader. *condition* may be something like *s.start < end* or *s.type # TextMappers.eot*.

        VAR s: TextMappers.Scanner; start: INTEGER;

    BEGIN

        *... define *start* ...*

        s.ConnectTo(text);

        s.SetPos(start);    *(* only necessary if *start # 0* *)*

        s.Scan;

        WHILE *condition* DO

            IF s.type = TextMappers.int THEN

                *... consume s.int ...*

            ELSIF s.type = TextMappers.real THEN

                *... consume s.real ...*

            ELSIF ...

                ...

            END;

            s.Scan

        END;

Listing 5-28. Code pattern for TextMappers.Scanner

**5.4 Modifying text**

In the previous sections, we have seen how new texts can be created using writers or formatters, and how existing texts can be read using readers or scanners. Many text commands first read a text stretch, perform some computation on it, and then replace it by the result of the computation. Examples are commands that set the font of a selection, set the color of a selection, or turn all small letters into capital letters.

The following example command reads the text selection and checks whether it is a string. If so, it interprets the string as a name and looks the name up in our example phone database. Then it replaces the name by the phone number.

MODULE ObxLookup0;

    IMPORT TextModels, TextMappers, TextControllers, ObxPhoneDB;

    PROCEDURE **Do***;

    *(** use TextCmds.SelectionGuard as guard command **)*

        VAR c: TextControllers.Controller; buf: TextModels.Model; from, to: INTEGER;

            s: TextMappers.Scanner; f: TextMappers.Formatter; number: ObxPhoneDB.String;

    BEGIN

        c := TextControllers.Focus();

        IF (c # NIL) & c.HasSelection() THEN

            c.GetSelection(from, to);

            s.ConnectTo(c.text);

            s.SetPos(from);

            s.Scan;

            IF s.type = TextMappers.string THEN

                buf := TextModels.CloneOf(c.text);

                f.ConnectTo(buf);

                ObxPhoneDB.LookupByName(s.string$, number);

                f.WriteString(number);

                from := s.start; to := s.Pos() - 1;    *(* scanner has already read on character beyond string! *)*

                c.text.Delete(from, to);                            *(* delete name *)*

                c.text.Insert(from, buf, 0, buf.Length());    *(* move phone number from buffer into text *)*

                c.SetSelection(from, from + LEN(number$))    *(* select the phone number *)*

            END

        END

    END Do;

END ObxLookup0.

Listing 5-29. Replacing a phone name by a phone number

The basic idea of the above command is that an auxiliary text object is used which acts as a buffer. The text selection is scanned, the scanned string is used as key to make a lookup in the phone database, the returned phone number is written into the buffer text, the original name is deleted from the text, and then the buffer contents is moved to where the name was. As a final step, the newly inserted text stretch is selected.

Note that the text model's *Delete* procedure shortens the text, while the *Insert* procedure makes the text longer. In particular, *Insert* doesn't overwrite existing text. *Insert* moves text, i.e., it removes them at the source and inserts them at the destination.

One way to make the above command more useful would be to accept a caret as input, instead of a selection. Then you could write a name and hit a keyboard shortcut for the command. The command then reads backwards from the caret position, until it finds the beginning of the name, and then substitutes the appropriate phone number for the name. This would be a simple way to implement keyboard macros.

Buffer texts, as used above, are particularly useful in helping to avoid screen flicker: since a buffer text is not displayed, it can be built up piecemeal, without causing any screen updates. When the buffer has been completed, its contents can be moved over to the visible destination text. This only causes one single screen update, which is fast and creates minimal flicker.

Moving text stretches from one text model to another one may be optimized if both text models have the same implementation. This is not necessarily the case, since different implementations of type *TextModels.Model* may coexist with each other. Two calls to *TextModels.dir.New()* may well return two text models that have different implementations.

For this reason, module *TextModels* exports the procedure *Clone*, which returns an empty text model of exactly the same type as its parameter. In fact, the standard text implementation even performs some further optimizations when a text model is cloned from another one, rather than allocated via *TextModels.dir.New()*.

We can now give a (slightly simplified) definition of *TextModels.Model*:

TYPE

    Model = POINTER TO ABSTRACT RECORD (Containers.Model)

        (m: Model) NewWriter (old: TextModels.Writer): TextModels.Writer, NEW, ABSTRACT;

        (m: Model) NewReader (old: TextModels.Reader): TextModels.Reader, NEW, ABSTRACT;

        (m: Model) Length (): INTEGER, NEW, ABSTRACT;

        (m: Model) Insert (pos: INTEGER; m0: TextModels.Model;

                                                    beg0, end0: INTEGER), NEW, ABSTRACT;

        (m: Model) InsertCopy (pos: INTEGER; m0: TextModels.Model;

                                                    beg0, end0: INTEGER), NEW, ABSTRACT;

        (m: Model) Delete (beg, end: INTEGER), NEW, ABSTRACT;

        (m: Model) Append (m0: TextModels.Model), NEW, ABSTRACT;

        (m: Model) Replace (beg, end: INTEGER; m0: TextModels.Model;

                                                    beg0, end0: INTEGER), NEW, ABSTRACT;

        (m: Model) SetAttr (beg, end: INTEGER; attr: TextModels.Attributes), NEW, ABSTRACT;

    END;

Listing 5-30. Definition of TextModels.Model

We have already met most of its procedures. *InsertCopy* is similar to *Insert*, but instead of moving, it copies a text stretch, without modifying the source. *Delete* removes a text stretch.

*Insert*, *InsertCopy*, and *Delete* are the elementary text operations. *Append* and *Replace* are provided for convenience and efficiency; they can be expressed in terms of the elementary operations:

    dst.Append(src)  ꉈ  dst.Insert(dst.Length(), src, 0, src.Length())

i.e., it moves the whole contents of the source text to the end of the destination text, thereby leaving an empty source text.

    t.Replace(b, e, t1, b1, e1)  ꉈ  t.Delete(b, e); t.Insert(b, t1, b1, e1)

i.e., it overwrites some part of the destination text by some part of the source text (which is thereby removed). Note that source and destination texts must be different.

*SetAttr* allows to set the text attributes of a whole text stretch.

**5.5 Text scripts**

When you have successfully applied the command *ObxLookup0.Do* to a text stretch, you can try out something that may surprise you: you can reverse ("undo") the operation by executing the *Edit->Undo* command! With *Edit->Redo* you can reverse the operation's reversal.

This capability is surprising because we have only implemented the plain *Do* procedure, but no code for undoing or redoing it. How come that the framework is able to undo/redo the operation anyway?

It is possible because the text model procedures which modify their model's state, such as *Model.Delete* and *Model.Insert*, are implemented in a particular way. They don't execute the modification directly. Instead, they create special operation objects and register them in the framework. The framework then can call the operation's appropriate procedure for performing the actual do/undo/redo functionality. This mechanism has been described in Chapter 3.

The following example shows how a sequence of text-modifying procedures is bracketed by *BeginScript* and *EndScript* to make the whole sequence undoable. *BeginScript* returns a script object, it gets the name of the compound operation as input.

MODULE ObxLookup1;

    IMPORT <u>Stores,</u> Models, TextModels, TextMappers, TextControllers, ObxPhoneDB;

    PROCEDURE **Do***;

    *(** use TextCmds.SelectionGuard as guard command **)*

        VAR c: TextControllers.Controller; buf: TextModels.Model; from, to: INTEGER;

            s: TextMappers.Scanner; f: TextMappers.Formatter; number: ObxPhoneDB.String;

            <u>script: Stores.Operation;</u>

    BEGIN

        c := TextControllers.Focus();

        IF (c # NIL) & c.HasSelection() THEN

            c.GetSelection(from, to);

            s.ConnectTo(c.text);

            s.SetPos(from);

            s.Scan;

            IF s.type = TextMappers.string THEN

                buf := TextModels.CloneOf(c.text);

                f.ConnectTo(buf);

                ObxPhoneDB.LookupByName(s.string$, number);

                f.WriteString(number);

                from := s.start; to := s.Pos() - 1;    *(* scanner has already read on character beyond string! *)*

                <u>Models.BeginScript(c.text, "#Obx:Lookup", script);</u>

                c.text.Delete(from, to);                            *(* delete name *)*

                c.text.Insert(from, buf, 0, buf.Length());    *(* move phone number from buffer into text *)*

                <u>Models.EndScript(c.text, script);</u>

                c.SetSelection(from, from + LEN(number$))    *(* select the phone number *)*

            END

        END

    END Do;

END ObxLookup1.

Listing 5-31. Example of a script (compound operation)

In *Models.BeginScript*, the name of the compound operation is given. It is mapped by the string mapping facility of BlackBox, because this name may be displayed to the user (in the undo/redo menu item).

In modules *ObxLookup0* and *ObxLookup1* use the following statement sequence to replace the name with the number:

    c.text.Delete(from, to);

    c.text.Insert(from, buf, 0, buf.Length());

To combine these two operations into a single compound operation (as experienced by the user), it was necessary to bracket the two calls with the *BeginScript / EndScript* pair. In this special case here, there actually would be a better solution:

    c.text.Replace(from, to, buf, 0, buf.Length());

*TextModels.Model.Replace* is a powerful procedure of which *Insert*, *Delete*,  and *Append* are special cases (provided for convenience):

    t.Delete(beg, end)    =    t.Replace(beg, end, t, 0, 0)

    t.Insert(pos, t0, beg, end)    =    t.Replace(pos, pos, t0, beg, end)

    t.Append(t0)    =    t.Replace(t.Length(), t.Length(), t0, 0, t0.Length())

Note that *InsertCopy* cannot be expressed in terms of *Replace*:

    t.InsertCopy(pos, t0, beg, end)    #    t.Replace(pos, pos, t0, beg, end)

The reason is that *Replace*, like *Insert*, deletes the source text. *InsertCopy* on the other hand is a pure copying operation which doesn't modify the source text (unless source and destination texts are identical and the destination position lies within the source text range).

**5.6 Summary**

We now conclude the discussion of the BlackBox Component Builder's Text subsystem. The goal of this tutorial was to give an overview over a relatively complex extension subsystem; to show the use of the previous part's design patterns; and to show the typical Text code patterns.

The treatment of the text subsystem cannot be complete, there are still many details for which the reference documentation of the various modules must be consulted. However, the general scope and typical applications should have become clearer. While we started with as little theory as possible and used simple examples only, the basic structure of a complex container subsystem will have become clearer by now.

Before we give a list of further on-line examples that you may want to study, here is a list of all standard modules of the Text subsystem (lowest module in the hierarchy first), with the most important types that they export:

**    module / type    description**

    TextModels    abstraction and default implementation of text models

        Attributes        attributes of a text elem (font, color, vertical offset)

        Context        link between a text model and a view embedded in it

        Directory        factory for text models

        Model        text carrier, factory for readers and writers

        Reader        rider for element-wise text reading

        Writer        rider for element-wise text writing

    TextMappers    abstraction and implementation of text mappers for Component Pascal symbols

        Formatter        mapper for symbol-wise text writing

        Scanner        mapper for symbol-wise text reading

    TextRulers    abstraction and default implementation for ruler views, which embody paragraph attributes

        Attributes        attributes of text ruler (tabs, margins, grid, etc.)

        Directory        factory for text rulers

        Ruler        view for rulers

        Style        model for rulers

    TextSetters    abstraction and default implementation for text setters, which perform line / page breaking

        Directory        factory for text setters

        Reader        text mapper which collects text into units of text setting

                (non-breakable sequences) and determines their size

        Setter        object which implements a text setting algorithm

    TextViews    abstraction and default implementation of text views

        Directory        factory for text views

        View        view for text models

    TextControllers    abstraction and default implementation of text controllers

        Controller        controller for text views

        Directory        factory for text controllers

    TextCmds    command package with most important text commands

Table 5-32. Table of Text system modules and most important types

    ObxHello0    write "Hello World" to log

    ObxHello1    write "Hello World" to new text

    ObxOpen0    open text document from a file, using standard file open dialog

    ObxOpen1    modify a text document on a file, using standard file open and save dialogs

    ObxCaps    change string to uppercase, using compound operation

    ObxDb    manage sorted list of records

    ObxTabs    transform some tabs into blanks

    ObxMMerge    mail merge of template and database

    ObxParCmd    interpret text which follows a command button

    ObxLinks    create a directory text with hyperlinks

    ObxAscii    traditional text file I/O

Table 5-33. Additional Obx on-line examples of Text subsystem

