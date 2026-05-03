**Printing**

DEFINITION Printing;

    IMPORT Fonts, Views, Dates;

    TYPE

        Par = POINTER TO LIMITED RECORD

            page: PageInfo;

            header, footer: Banner;

            copies-: INTEGER

        END;

        Banner = RECORD

            font: Fonts.Font;

            gap: INTEGER;

            left, right: ARRAY 128 OF CHAR

        END;

        PageInfo = RECORD

            first, from, to: INTEGER;

            alternate: BOOLEAN;

            title: Views.Title

        END;

    VAR par: Par;

    PROCEDURE NewPar (IN page: PageInfo; IN header, footer: Banner; copies: INTEGER): Par;

    PROCEDURE NewDefaultPar (title: Views.Title): Par;

    PROCEDURE PrintView (view: Views.View; p: Par);

    PROCEDURE Current (): INTEGER;

    PROCEDURE PrintBanner (f: Views.Frame; IN p: PageInfo; IN b: Banner; IN date: Dates.Date;

                                            IN time: Dates.Time; x0, x1, y: INTEGER);

END Printing.

This module allows to print a document, including headers and footers.

TYPE **Par**

Specification of the print job. Is passed as parameter to the function *PrintView*. During printing, it can be accessed via the global variable *par*. During printing, its state can be changed.

**page**: PageInfo

Information about the area of the pages to be printed; about page numbers; and whether printing of left and right pages is treated differently.

**header, footer**: Banner

Header/footer line to be printed on every page.

**copies-**: INTEGER    copies >= 1

Number of copies to be printed.

TYPE **Banner**

Describes a header or a footer line.

**font**: Fonts.Font

Font of the header/footer line. Only one font is possible.

**gap**: INTEGER

Distance to next base line. Currently ignored for footers.

**left, right**: ARRAY 128 OF CHAR

Text of the header/footer line. In this string the following abbreviations may occur:

    &p - replaced by current page number as arabic numeral

    &r - replaced by current page number as roman numeral

    &R - replaced by current page number as capital roman numeral

    &a - replaced by current page number as alphanumeric character

    &A - replaced by current page number as capital alphanumeric character

    &d - replaced by printing date

    &t - replaced by printing time

    &&- replaced by & character

    &; - specifies split point

    &f - title

The text is centered, except if a split point occurs. A split point acts like a spring: it "presses" the items to the left and right apart. For example, "&;&p" presses an arabic page number to the right page border.

TYPE **PageInfo**

Information about the pages to be printed.

**first**: INTEGER

Page number of the first page to be printed. Page number of the currently printed page is *Current() + first*. May be changed during printing.

**from, to**: INTEGER    (from >= 0) & (to >= from)

Range of pages to be printed. Specification is not in page numbers, but in absolute number. The first page of the document is number *0*. There will be *to - from + 1* pages printed.

**alternate**: BOOLEAN

Determines whether left and right pages are treated differntly. If *~alternate*, then every page will be printed with the banners specified for right pages. Otherwise, pages with odd page number use the right banners, while pages with even page numbers use the left banners. The page number of the current page is *first + Current()*.

**title**: Views.Title

Title of the document.

VAR **par**: Par

This variable is set during printing of the document and may be changed by a printed view. However, this variable should not be used to determine whether the document is printed or displayed (-> *Views.IsPrinterFrame*).

PROCEDURE **NewPar** (IN page: PageInfo; IN header, footer: Banner; copies: INTEGER): Par

Allocates and initializes a new printing parameters object.

Post

result.page = page

result.header = header

result.footer = footer

result.copies = copies

header.font # NIL => result.header.font = header.font

header.font = NIL => result.header.font = Fonts.dir.Default()

footer.font # NIL => result.footer.font = footer.font

footer.font = NIL => result.footer.font = Fonts.dir.Default()

PROCEDURE **NewDefaultPar** (title: Views.Title): Par

Allocates and initializes a new printing parameters object.

Post

result.page.first = 1

result.page.from = 0

result.page.to = 9999

result.page.alternate = FALSE

result.copies = 1

result.header.gap = 0

result.header.left = ""

result.header.right = ""

result.header.font = Fonts.dir.Default()

result.footer.gap = 0

result.footer.left = ""

result.footer.right = ""

result.footer.font = Fonts.dir.Default()

PROCEDURE **Current** (): INTEGER

Returns the page number of the page currently being printed.

Post

result >= 0

PROCEDURE **PrintView** (view: Views.View; par: Par)

Prints view on the (current) printer, provided that a printer is available.

For the parameter *par*, *NIL* can be passed. In this case, a default par parameter is generated. It is initialized

with

    copies := 1;

    page.first := 1, page.from := 0, page.to := 9999, page.alternate := FALSE; page.title := title;

    header.gap := 0; header.left := ""; header.right := "";

    header.font := Fonts.dir.Default();

    footer.gap := 0; footer.left := ""; footer.right := "";

    footer.font := Fonts.dir.Default();

otherwise

    par.title is overwritten by title.

Actually, not the view *view* is printed, but a copy of it. *PrintView* makes a shallow copy of the view that is being printed, in order to avoid the original view to be changed by pagination, scrolling, or similar modifications that may be performed during printing.

Pre

view # NIL    20

par # NIL =>

    par.page.first >= 0    23

    par.page.from >= 0    24

    par.page.to >= par.page.from    25

    par.copies > 0    25

PROCEDURE **PrintBanner** (f: Views.Frame; IN p: PageInfo; IN b: Banner;

                                                IN date: Dates.Date; IN time: Dates.Time; x0, x1, y: INTEGER)

Used internally.

