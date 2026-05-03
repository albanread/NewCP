**Fonts**

DEFINITION Fonts;

    CONST

        mm = 36000;

        point = 12700;

        italic = 0; underline = 1; strikeout = 2;

        normal = 400; bold = 700;

        default = "*";

    TYPE

        Typeface = ARRAY 64 OF CHAR;

        TypefaceInfo = POINTER TO RECORD

            next: TypefaceInfo;

            typeface: Typeface

        END;

        Font = POINTER TO ABSTRACT RECORD

            typeface-: Typeface;

            size-: INTEGER;

            style-: SET;

            weight-: INTEGER;

            (f: Font) Init (typeface: Typeface; size: INTEGER; style: SET; weight: INTEGER, NEW;

            (f: Font) GetBounds (OUT asc, dsc, w: INTEGER), NEW, ABSTRACT;

            (f: Font) StringWidth (IN s: ARRAY OF CHAR): INTEGER, NEW, ABSTRACT;;

            (f: Font) SStringWidth (IN s: ARRAY OF SHORTCHAR): INTEGER, NEW, ABSTRACT;;

            (f: Font) IsAlien (): BOOLEAN, NEW, ABSTRACT;

        END;

        Directory = POINTER TO ABSTRACT RECORD

            (d: Directory) This (typeface: Typeface; size: INTEGER; style: SET;

                                        weight: INTEGER): Font, NEW, ABSTRACT;

            (d: Directory) Default (): Font, NEW, ABSTRACT;

            (d: Directory) TypefaceList (): TypefaceInfo, NEW, ABSTRACT

        END;

    VAR dir-, stdDir-: Directory;

    PROCEDURE SetDir (d: Directory);

END Fonts.

A *font* is a collection of character glyphs, i.e., a collection of distinct visual representations of characters. Visual representations of the same character may differ in size (e.g., 12 point vs. 16 point), style (e.g., plain vs. italic), typeface (e.g., Times vs. Helvetica), and weight (e.g., bold vs. normal).

In BlackBox, most distances are measured in universal units. Several important distance values in these units are defined below:

 um    = 36    micrometer

 mm    = 36000    millimeter

 cm    = 10 * mm    centimeter

 m    = 1000 * mm    meter

 inch    = 914400    inch

Font sizes are measured in these universal units as well. The following values are used in connection with font sizes:

 point    = 12700    1/72 inch    (desktop publishing point)

 pica    = 12636    0.351 mm

 didot    = 13500    0.375 mm

 cicero    = 163800    4.55 mm

However, it should be mentioned that in modern typography the millimeter is the dominating measure, followed by the point as established in desktop publishing software.

Module *Fonts* provides an abstract type *Font*, which mainly allows to measure the widths of characters and strings in universal units. These measures are completely device-independent. There is no device-specific information (e.g., character bitmap) in a font object. Font objects are only used for measurements and for the identification of a font. In the latter capacity, they can be passed as parameters to output routines. These output routines generate or access (device-dependent) character bitmaps in a way not specified by BlackBox.

An application need not be aware whether font bitmaps are stored permanently ("bitmapped fonts") or whether they are generated on demand (using "outline fonts").

The meanings of two important font metrics, namely ascent and descent, are illustrated in the following diagram:

Figure 1.  Base Line, Ascent, Descent

Characters of a word are placed side by side on a so-called *base line*. The *ascent* measures how far any character of a font may extend above the base line. The *descent* measures how far any character of a font may extend below the base line. The line spacing is the sum of ascent and descent.

In BlackBox, the ascent must be large enough to accomodate oversized characters plus the minimal required distance between lines. Thus the ascent includes "line gap", "internal leading", and "external leading" as defined in other font models.

Not all character codes need to be represented in a font. A character which is not represented in a font is displayed by a special "missing" symbol, e.g., an empty rectangle. Alternatively, BlackBox may use another font to display the character.

It is often desirable to display text on the screen in a way which is similar to the way it is printed on paper. This is known as WYSIWYG display (What You See Is What You Get). However, there are several factors which make true WYSIWYG display a problematic proposition.

The most fundamental problem is the large difference between today's screen and printer resolutions. Screens have typical spatial resolutions of about 70 to 100 dpi (dot per inch), while laser printers have resolutions of at least 300 dpi. This large difference forces the programmer to decide whether

ꀢ to tune text drawing for maximal legibility on screen, and thereby giving up device-independence and reducing the quality of hard copy, or

ꀢ to tune text drawing for maximal precision, which results in reduced legibility on screen due to rounding effects, or

ꀢ to give up the strict WYSIWYG requirements to some degree.

All three solutions have their merits and problems, and all three solutions can be found in commercial word processors.

Another problem for pure WYSIWYG display is that not all fonts are available on every machine. This means that a document containing a particular font cannot be shown correctly on a computer where this font is not installed. In order to make it possible to open a document containing such a missing font (without converting this font permanently) a mechanism is provided in BlackBox to temporarily substitute a place holder for a missing font, a so-called "alien" font.

A font can be looked up in a font directory. Module *Fonts* provides an abstract type *Directory* for this purpose. If the directory cannot find a font, it creates an alien font object. An alien font internally uses an existing font for measurements and display, such that it can be used like any other font.

An application which needs a font but has no preferences should use the *default font*. The default font is a system- or user-made choice out of one of the available fonts. The identity of the default font may vary over time.

Fonts don't know about persistence. If you need to store a font description, you can use the procedures *Views.ReadFont* and *Views.WriteFont*.

CONST **mm**, **point**

These are the most important font size measures in universal units.

CONST **italic**, **underline, strikeout**

Three standard font attributes.

CONST **normal, bold**

Two major font weights.

CONST **default**

This "pseudo typeface" is a placeholder name for the current default typeface. In BlackBox, the user can configure the default typeface to his or her needs, and may change it at run-time.

TYPE **Typeface**

String type for the typeface name of a font.

TYPE **TypefaceInfo**

This type gives the name of an available typeface. Typeface info records are connected in a linear list. No ordering is defined.

**next**: TypefacInfo

Next element of the list.

**typeface**: Typeface    typeface # ""

Name of this element's typeface.

TYPE **Font**

ABSTRACT

This is the base type for fonts, which allows to identify fonts and to measure font information in universal units.

Fonts are allocated by font directories.

Fonts are used by models which contain formatted text, by views which draw text, and by commands which operate on text.

Fonts are extended by BlackBox, internally.

**typeface**-: Typeface    typeface # ""

The font's typeface name.

**size**-: INTEGER    size > 0

The font's size in universal units.

**style**-: SET    subset of {italic, underline, strikeout}

The set of the font's style attributes.

**weight**-: INTEGER    0 <= weigth <= 1000

A font's weight, i.e., the thickness of the strokes.

PROCEDURE (f: Font) **Init** (typeface: Typeface; size: INTEGER; style: SET; weight: INTEGER)

NEW

Initialize font fields.

*Init* is called by BlackBox, internally.

Pre

f.size = 0    20    font must not be initialized yet

size > 0    21

style is subset of {italic, underline, strikeout}    22

0 <= weight <= 1000    23

Post

f.fingerprint = fingerprint

f.typeface = typeface  &  f.size = size  &  f.style = style  &  f.weight = weight

PROCEDURE (f: Font) **GetBounds** (OUT asc, dsc, w: INTEGER)

NEW, ABSTRACT

Get font ascent, descent, and the width of the widest character in the font.

Post

asc >= 0  &  dsc >= 0  &  w >= 0

PROCEDURE (f: Font) **StringWidth** (IN s: ARRAY OF CHAR): INTEGER

NEW, ABSTRACT

Measures the width of a string in universal units. The string may contain arbitrary Unicode characters.

*StringWidth* is used by models or views which need to format text.

Pre

s is terminated by 0X    index trap

Post

result >= 0    width of string

PROCEDURE (f: Font) **SStringWidth** (IN s: ARRAY OF SHORTCHAR): INTEGER

NEW, ABSTRACT

Measures the width of a short string in universal units. The string can only contain Latin-1 characters.

*SStringWidth* is used by models or views which need to format text.

Pre

s is terminated by 0X    index trap

Post

result >= 0    width of string

PROCEDURE (f: Font) **IsAlien** (): BOOLEAN

NEW, ABSTRACT

Tells whether *f* is an alien font.  An alien font is returned upon lookup of a font which cannot be found or generated. It is used as a place holder for the missing font. Alien fonts can be displayed, but their metrics are usually not the same as the correct font's metrics and their glyphs usually differ significantly from the correct font's glyphs.

*IsAlien* is used in commands which inform users about the existence of alien fonts in a document.

TYPE **Directory**

ABSTRACT

Directory for the lookup of fonts.

Font directories are allocated by BlackBox.

Font directories are used in models, views, and commands which need to specify a font for later use.

Font directories are extended by BlackBox, internally.

PROCEDURE (d: Directory) **This** (typeface: Typeface; size: INTEGER;

                                                            style: SET; weight: INTEGER): Font

NEW, ABSTRACT

Returns the font with the attributes *(typeface, size, style, weight)*. If the font information cannot be found or generated, an alien font is returned instead. An alien font has the requested attributes, even though a different font is actually used.

If a font is requested which has the same attributes as another, previously requested font, the directory attempts to return the same font object (i.e., the same pointer value) as it did before. However, if a large number of fonts is used, it may happen that another font object is returned instead. Such an object has the same attributes and provides the same metrics and identical glyphs as the older font object.

*This* is used to look up a font when specific font attributes are given.

Pre

size > 0    20

Post

result # NIL

result.typeface = typeface

result.size = size

result.style = style

result.weight = weight

PROCEDURE (d: Directory) **Default** (): Font

NEW, ABSTRACT

Returns the current default font.

*Default* is used when a font is needed and no specific font attributes are desired.

Post

result # NIL

PROCEDURE (d: Directory) **TypefaceList** (): TypefaceInfo

NEW, ABSTRACT

Returns information about the available typefaces. The result is a linear list of typeface names, in no particular order.

VAR **dir**-, **stdDir**-: Directory    dir # NIL  &  stdDir # NIL

Directories for the lookup of fonts.

PROCEDURE **SetDir** (d: Directory)

Assigns directory.

*SetDir* is used in configuration routines.

Pre

d # NIL    20

Post

stdDir' = NIL

    stdDir = d

stdDir' # NIL

    stdDir = stdDir'

dir = d

