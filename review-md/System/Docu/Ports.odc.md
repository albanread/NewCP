**Ports**

DEFINITION Ports;

    IMPORT Fonts;

    CONST

        black = 00000000H; white = 00FFFFFFH;

        grey6 = 00F0F0F0H; grey12 = 00E0E0E0H; grey25 = 00C0C0C0H;

        grey50 = 00808080H; grey75 = 00404040H;

        red = 000000FFH; green = 0000FF00H; blue = 00FF0000H;

        defaultColor = 01000000H;

        mm = 36000; point = 12700; inch = 914400;

        fill = -1;

        openPoly = 0; closedPoly = 1; openBezier = 2; closedBezier = 3;

        invert = 0; hilite = 1; dim25 = 2; dim50 = 3; dim75 = 4;

        hide = FALSE; show = TRUE;

        arrowCursor = 0; textCursor = 1; graphicsCursor = 2; tableCursor = 3; bitmapCursor = 4; refCursor = 5;

        keepBuffer = FALSE; disposeBuffer = TRUE;

    TYPE

        Color = INTEGER;

        Point = RECORD

            x, y: INTEGER

        END;

        Port = POINTER TO ABSTRACT RECORD

            unit-: INTEGER;

            (p: Port) Init (unit: INTEGER; printerMode: BOOLEAN), NEW;

            (p: Port) GetSize (OUT w, h: INTEGER), NEW, ABSTRACT;

            (p: Port) SetSize (w, h: INTEGER), NEW, ABSTRACT;

            (p: Port) NewRider (): INTEGER, NEW, ABSTRACT;

            (p: Port) OpenBuffer (l, t, r, b: INTEGER), NEW, ABSTRACT;

            (p: Port) CloseBuffer, NEW, ABSTRACT

        END;

        Rider = POINTER TO ABSTRACT RECORD

            (rd: Rider) SetRect (l, t, r, b: INTEGER), NEW, ABSTRACT;

            (rd: Rider) GetRect (OUT l, t, r, b: INTEGER), NEW, ABSTRACT;

            (rd: Rider) Base (): Port, NEW, ABSTRACT;

            (rd: Rider) Move (dx, dy: INTEGER), NEW, ABSTRACT;

            (rd: Rider) SaveRect (l, t, r, b: INTEGER; OUT res: INTEGER), NEW, ABSTRACT;

            (rd: Rider) RestoreRect (l, t, r, b: INTEGER; dispose: BOOLEAN), NEW, ABSTRACT;

            (rd: Rider) DrawRect (l, t, r, b, s: INTEGER; col: Color), NEW, ABSTRACT;

            (rd: Rider) DrawOval (l, t, r, b, s: INTEGER; col: Color), NEW, ABSTRACT;

            (rd: Rider) DrawLine (x0, y0, x1, y1, s: INTEGER; col: Color), NEW, ABSTRACT;

            (rd: Rider) DrawPath (IN p: ARRAY OF Point; n, s: INTEGER; col: Color;

                                                                        path: INTEGER), NEW, ABSTRACT;

            (rd: Rider) MarkRect (l, t, r, b, s: INTEGER; mode: INTEGER; show: BOOLEAN), NEW, ABSTRACT;

            (rd: Rider) Scroll (dx, dy: INTEGER), NEW, ABSTRACT;

            (rd: Rider) SetCursor (cursor: INTEGER), NEW, ABSTRACT;

            (rd: Rider) Input (OUT x, y: INTEGER; OUT modifiers: SET;

                                                                        OUT isDown: BOOLEAN), NEW, ABSTRACT;

            (rd: Rider) DrawString (x, y: INTEGER; col: Color; IN s: ARRAY OF CHAR;

                                                                        font: Fonts.Font), NEW, ABSTRACT;

            (rd: Rider) DrawSString (x, y: INTEGER; col: Color; IN s: ARRAY OF SHORTCHAR;

                                                                        font: Fonts.Font), NEW, ABSTRACT;

            (rd: Rider) CharIndex (x, pos: INTEGER; IN s: ARRAY OF CHAR;

                                                                        font: Fonts.Font): INTEGER, NEW, ABSTRACT;

            (rd: Rider) SCharIndex (x, pos: INTEGER; IN s: ARRAY OF SHORTCHAR;

                                                                        font: Fonts.Font): INTEGER, NEW, ABSTRACT;

            (rd: Rider) CharPos (x, index: INTEGER; IN s: ARRAY OF CHAR;

                                                                        font: Fonts.Font): INTEGER, NEW, ABSTRACT;

            (rd: Rider) SCharPos (x, index: INTEGER; IN s: ARRAY OF SHORTCHAR;

                                                                        font: Fonts.Font): INTEGER, NEW, ABSTRACT

        END;

        Frame = POINTER TO ABSTRACT RECORD

            unit-: INTEGER;

            dot-: INTEGER;

            rider-: Rider;

            gx-, gy-: INTEGER;

            (f: Frame) ConnectTo (p: Port), NEW, EXTENSIBLE;

            (f: Frame) SetOffset (gx, gy: INTEGER), NEW, EXTENSIBLE;

            (f: Frame) SaveRect (l, t, r, b: INTEGER; OUT res: INTEGER), NEW;

            (f: Frame) RestoreRect (l, t, r, b: INTEGER; dispose: BOOLEAN), NEW;

            (f: Frame) DrawRect (l, t, r, b, s: INTEGER; col: Color), NEW;

            (f: Frame) DrawOval (l, t, r, b, s: INTEGER; col: Color), NEW;

            (f: Frame) DrawLine (x0, y0, x1, y1, s: INTEGER; col: Color), NEW;

            (f: Frame) DrawPath (IN p: ARRAY OF Point; n, s: INTEGER; col: Color; path: INTEGER), NEW;

            (f: Frame) MarkRect (l, t, r, b, s: INTEGER; mode: INTEGER; show: BOOLEAN), NEW;

            (f: Frame) Scroll (dx, dy: INTEGER), NEW;

            (f: Frame) SetCursor (cursor: INTEGER), NEW;

            (f: Frame) Input (OUT x, y: INTEGER; OUT modifiers: SET; OUT isDown: BOOLEAN), NEW;

            (f: Frame) DrawString (x, y: INTEGER; col: Color; IN s: ARRAY OF CHAR;

                                                font: Fonts.Font), NEW;

            (f: Frame) DrawSString (x, y: INTEGER; col: Color; IN s: ARRAY OF SHORTCHAR;

                                                font: Fonts.Font), NEW;

            (f: Frame) CharIndex (x, pos: INTEGER; IN s: ARRAY OF CHAR;

                                                font: Fonts.Font): INTEGER, NEW;

            (f: Frame) SCharIndex (x, pos: INTEGER; IN s: ARRAY OF SHORTCHAR;

                                                font: Fonts.Font): INTEGER, NEW;

            (f: Frame) CharPos (x, index: INTEGER; IN s: ARRAY OF CHAR;

                                                font: Fonts.Font): INTEGER, NEW;

            (f: Frame) SCharPos (x, index: INTEGER; IN s: ARRAY OF SHORTCHAR;

                                                font: Fonts.Font): INTEGER, NEW

        END;

    VAR background, dialogBackground: Color;

    PROCEDURE IsPrinterPort (p: Port): BOOLEAN;

    PROCEDURE RGBColor (red, green, blue: INTEGER): Color;

END Ports.

*Ports* are carriers for pixel data. Examples of ports are screen and printer ports.

*Riders* are access paths to ports. The drawing operations of a rider are performed in a coordinate system with positive x-axis and negative y-axis, i.e., x values increase towards the right, while y values increase towards the bottom. This coordinate system is the same for every rider on a port, with the origin at the upper-left corner of the port. Points are coordinate pairs (in device coordinates) which denote the upper-left corner of a pixel:

Figure 1.  Drawing Plane

A rider occupies a rectangle within the port area. Each rider acts as a clipping rectangle, to which all its drawing operations are clipped.

*Frames* are port mappers, which provide port output operations and input from mouse and keyboard. Frame coordinates are scaled and translated such that they are independent of the frame's position on a port, and independent of the port's spacial resolution. For this reason, all frame operations use universal units (-> *Fonts*) for coordinates, while all port and rider operations use pixel coordinates.

CONST **black**, **white**, **grey6, grey12, grey25**, **grey50**, **grey75**, **red**, **green**, **blue**

RGB values for several important colors.

CONST **defaultColor**

This is a pseudo color which is substituted by the currently set default foreground color for drawing.

CONST **mm**, **point, inch**

Three important distance measures in universal units.

CONST **fill**

This value may be passed to the procedures *DrawRect, DrawOval*, *DrawPath*, and *MarkRect* as *size* parameter, to cause the drawing of a filled shape, instead of the shape's outline only.

CONST **openPoly, closedPoly, openBezier, closedBezier**

These values may be passed to the procedure *DrawPath* as *path* parameter. They causes the drawing of a polyline, a polygon, an open Bezier curve, or of a closed Bezier curve. Note that with Bezier curves, only every third point lies on the curve.

Figure 2.  Various Path Examples

CONST **invert, hilite, dim25, dim50, dim75**

These values may be passed as *mode*-parameter to procedure *MarkRect*. They cause the marked rectangle to become inverted, hilighted, or dimmed. The exact interpretation of these modes is platform-dependent.

In the simplest case, all three modes are implemented the same way, namely by inverting each bit in the color value which represents a pixel. Ideally, *hilite* should replace an area's background color with a user-selectable hilight-color, and vice versa. The three dimming modes, applied to a white background, deliver light, medium, and dark grey values, respectively.

CONST **hide, show**

These values may be passed as *show*-parameter of the *MarkRect* procedures. *hide* means that an existing mark should be removed, and *show* means that the mark should be drawn. In some implementations, the operation may be identical for *hide* and *show*, but this is not guaranteed.

CONST **arrowCursor**

The default shape of the cursor.

CONST **textCursor**, **graphicsCursor**, **tableCursor**, **bitmapCursor**

Cursor shapes which correspond to the type of data currently being manipulated: sequential, large shapes, regularly arranged objects, small shapes.

CONST **refCursor**

Cursor shape for indicating references, such as hyperlinks.

CONST **keepBuffer, disposeBuffer**

These constants may be passed to the *dispose* parameters of procedures *Rider.RestoreRect* and *Frame.RestoreRect*.

TYPE **Color** = INTEGER

A color is a four-byte value where the least significant byte (interpreted as unsigned integer) specifies the red-intensity of an RGB triple. The next byte specifies the green-intensity, the third byte represents the blue-intensity. The most significant byte must be set to zero.

TYPE **Point**

This type is used to construct paths for the *DrawPath* procedure. A path consists of an array of points, where points are coordinate pairs.

Points are used in drawing routines that call *DrawPath*.

**x, y**: INTEGER

Coordinate pair.

TYPE **Port**

ABSTRACT

Carrier for pixel data.

Ports are allocated and used internally.

Ports are implemented internally.

**unit**-: INTEGER    unit > 0

The size of a pixel in universal units.

PROCEDURE (p: Port) **Init** (unit: INTEGER; printerMode: BOOLEAN)

Sets the spacial resolution (in universal units per pixel). Parameter *printerMode* determines whether the port acts as a printing object.

Pre

p.unit = 0  OR p.unit = unit    20

unit > 0    21

Post

p.unit = unit

PROCEDURE (p: Port) **GetSize** (OUT w, h: INTEGER)

NEW, ABSTRACT

Get the port's current size (in pixels).

Post

w >= 0

h >= 0

PROCEDURE (p: Port) **SetSize** (w, h: INTEGER)

NEW, ABSTRACT

Sets the port's size (in pixels).

Pre

w >= 0    20

h >= 0    21

PROCEDURE (p: Port) **NewRider** (): Rider

NEW, ABSTRACT

Returns a rider that has the appropriate type for this port implementation.

PROCEDURE (p: Port) **OpenBuffer** (l, t, r, b: INTEGER)

NEW, ABSTRACT

Opens an off-screen buffer for port *p*. The buffer is initialized with the contents of *p*'s rectangle *(l, t, r, b)*. *OpenBuffer* must be followed by a call to *CloseBuffer*. Calls to *OpenBuffer* must not be nested.

Used internally, for restoring a window flicker-free in the background. Not to be used for other purposes.

PROCEDURE (p: Port) **CloseBuffer**

NEW, ABSTRACT

Copy back the contents of the port's off-screen buffer, and release the buffer. *OpenBuffer* must have been called before.

Used internally, for restoring a window flicker-free in the background. Not to be used for other purposes.

TYPE **Rider**

ABSTRACT

Access path to a port (i.e., to a pixel carrier). A rider uses the same coordinate system as its port, with the origin being the upper-left corner of the port. All coordinates used for a rider are in device coordinates, i.e., in pixels. Riders also contain a clipping rectangle. Normally, it is manipulated automatically by the framework, but in special circumstances, it can be useful also for view programmers.

Riders are allocated by ports.

Riders are used internally (by frames, see below) and implemented internally.

PROCEDURE (rd: Rider) **SetRect** (l, t, r, b: INTEGER)

NEW, ABSTRACT

Sets the rider's clipping rectangle on the port (in pixels). Normally, this method is called by the framework only. If you use it explicitly, you should restore the old clipping rectangle after you are done. The framework sets up the clipping rectangle before a view's *Restore* method is called (-> Views.View.Restore). This ensures that drawing never occurs outside of the drawing area that belongs to the view, as long as the clipping rectangle is not modified by the view.

The clipping rectangle must never be made larger than the size set up by the framework, it may only be made smaller. Otherwise, the results are unpredictable.

Pre

0 <= l <= r  &  0 <= t <= b    20

PROCEDURE (rd: Rider) **GetRect** (OUT l, t, r, b: INTEGER)

NEW, ABSTRACT

Gets the rider's clipping rectangle on the port (in pixels).

Post

0 <= l <= r  &  0 <= t <= b

PROCEDURE (rd: Rider) **Base** (): Port

NEW, ABSTRACT

Returns the port to which *rd* is connected.

Post

result # NIL

PROCEDURE (rd: Rider) **Move** (dx, dy: INTEGER)

NEW, ABSTRACT

Used internally.

PROCEDURE (rd: Rider) **SaveRect** (l, t, r, b: INTEGER; OUT res: INTEGER)

NEW, ABSTRACT

Saves a rectangle (parallel to the coordinate axes) of width *r - l* and of height *b - t* in a background buffer, from where it can be restored later using *RestoreRect*. *SaveRect* must be balanced by *RestoreRect(l, t, r, b, disposeBuffer)*. All coordinates are in pixels.

Calls to *SaveRect* may not be nested, and they may not occur during restoration of a view (-> *Views.View.Restore*). The purpose of *SaveRect/RestoreRect* is to act as temporary buffering mechanism during mouse tracking (->*Views.View.HandleCtrlMsg*).

 *res = 0* means that the call was successful, otherwise *RestoreRect* must not be called.

Pre

(l <= r) & (t <= b)    20

PROCEDURE (rd: Rider) **RestoreRect** (l, t, r, b: INTEGER; dispose: BOOLEAN)

NEW, ABSTRACT

After a successful call to *SaveRect*, the same rectangle, i.e., its pixelmap contents as it was upon saving, can be restored with *RestoreRect*. All coordinates are in pixels. *RestoreRect* can be called several times in succession; the last time with *dispose = disposeBuffer* and all other times with *dispose = keepBuffer*.

Pre

(l <= r) & (t <= b)    20

PROCEDURE (rd: Rider) **DrawRect** (l, t, r, b, s: INTEGER; col: Color)

NEW, ABSTRACT

Draws a rectangle (parallel to the coordinate axes) of width *r - l* and of height *b - t*. All coordinates are in pixels. If *s < 0*, the rectangle is filled with color *col*. Otherwise, the rectangle is drawn as an outline of thickness *s*. The outline is drawn *inside* of the rectangle. If *s = 0*, a very thin outline (hairline) is used.

Pre

(l <= r) & (t <= b)    20

(s >= 0) OR (s = fill)    21

PROCEDURE (rd: Rider) **DrawOval** (l, t, r, b, s: INTEGER; col: Color)

NEW, ABSTRACT

Draws an ellipse (parallel to the coordinate axes) of width *r - l* and of height *b - t*. All coordinates are in pixels. If *s < 0*, the ellipse is filled with color *col*. Otherwise, the ellipse is drawn as an outline of thickness *s*. The outline is drawn *inside* of the rectangle. If *s = 0*, a very thin outline (hairline) is used.

Pre

(l <= r) & (t <= b)    20

(s >= 0) OR (s = fill)    21

PROCEDURE (rd: Rider) **DrawLine** (x0, y0, x1, y1, s: INTEGER; col: Color)

NEW, ABSTRACT

Draws a line from the point *(x0, y0)* to the point *(x1, y1)* of thickness *s* in color *col*: All coordinates are in pixels.

Note that if you need to draw strictly horizontal or vertical lines, you could use *DrawRect* with *fill* instead of *DrawLine*. The advantage of *DrawRect* is that it is clearer which pixels are really drawn, it's the pixels that are strictly inside the bounding rectangle.

Figure 3:  Line

If *s = 0*, a very thin outline (hairline) is used.

Pre

s >= 0    20

PROCEDURE (rd: Rider) **DrawPath** (IN p: ARRAY OF Point; n, s: INTEGER; col: Color;

                                                                path: INTEGER)

NEW, ABSTRACT

Draws the path consisting of points *p[0] .. p[n - 1]* in color *col*. The nature of the path is given by parameter *path*. It can either be a polyline, a polygon, an open Bezier curve, or a closed Bezier curve. The polyline is the same that a sequence of *DrawLine* operations would generate. For a polygon, the *n* points define the mathematical region which will be outlined or filled. An open path with *n* points results in *n - 1* path pieces, a closed path with *n* points results in *n* path pieces. All coordinates in the point array are in pixels.

Pre

n >= 0    20

n <= LEN(p)    21

(s = fill)  =>  (path = closedPoly) OR (path = closedBezier)    22

(s >= 0) OR (s = fill)    23

path IN {closedPoly, openPoly, closedBezier, openBezier}    25

path = openPoly

    n >= 2    20

path = closedPoly

    n >= 2    20

path = openBezier

    n >= 4    20

    n MOD 3 = 1    24

path = closedBezier

    n >= 3    20

    n MOD 3 = 0    24

PROCEDURE (rd: Rider) **MarkRect** (l, t, r, b, s: INTEGER; mode: INTEGER; show: BOOLEAN)

NEW, ABSTRACT

Marks a rectangle (parallel to the coordinate axes) of width *r - l* and of height *b - t*. All coordinates are in pixels. If *s < 0*, the rectangle is filled in some way dependent on *mode*. Otherwise, the rectangle is drawn as an outline of thickness *s*. The outline is drawn *inside* of the rectangle. If *s = 0*, a very thin outline (hairline) is used.

The meaning of *mode* is implementation-dependent, but it must change the marked area in a visible way. *show* indicates whether the mark should be drawn or removed. Calling *MarkRect* with *show* and then directly afterwards with *hide* (otherwise with the same parameters) should re-establish exactly the state before the first call.

Pre

(l <= r) & (l <= t)    20

s >= 0    21

mode IN {invert, hilite, dim25, dim50, dim75}    22

PROCEDURE (rd: Rider) **Scroll** (dx, dy: INTEGER)

NEW, ABSTRACT

Shifts the rider's contents by vector *(dx, dy)*. The translation vector is given in pixels. Shifting occurs completely *within* the rider's rectangle, ie., pixels outside of it are neither written nor read. The part of the rectangle that becomes newly exposed is undefined.

The purpose of *Scroll* is to speed up scrolling operations by reusing existing pixel data instead of making the application redraw everything.

However, under special circumstances, this procedure may not actually copy pixel data, but cause the application to restore part of the rectangle instead anyway.

Warning: this operation may only be used on interactive ports, in order to update the screen display after a user manipulation.

Figure 4.  Effect of Scroll Operation

PROCEDURE (rd: Rider) **SetCursor** (cursor: INTEGER)

NEW, ABSTRACT

Sets the cursor to the given value.

Pre

cursor IN {arrowCursor..refCursor}    20

PROCEDURE (rd: Rider) **Input** (OUT x, y: INTEGER; OUT modifiers: SET; OUT isDown: BOOLEAN)

NEW, ABSTRACT

Polls the current mouse location and tells whether the mouse button is currently pressed. All coordinates are in pixels. In *modifiers*, the currently pressed modifier keys are returned, like *Controllers.doubleClick*, *Controllers.extend*, *Controllers.modify*, and possibly additional platform-specific modifiers.

PROCEDURE (rd: Rider) **DrawString** (x, y: INTEGER; col: Color; IN s: ARRAY OF CHAR;

                                                                    font: Fonts.Font)

NEW, ABSTRACT

Draws string *s* in color *col* and font *font* with the base line at *y*. All coordinates are in pixels.

Pre

font # NIL    20

PROCEDURE (rd: Rider) **DrawSString** (x, y: INTEGER; col: Color; IN s: ARRAY OF SHORTCHAR;

                                                                    font: Fonts.Font)

NEW, ABSTRACT

Draws short string *s* in color *col* and font *font* with the base line at *y*. All coordinates are in pixels.

Pre

font # NIL    20

PROCEDURE (rd: Rider) **CharIndex** (x, pos: INTEGER; IN s: ARRAY OF CHAR;

                                                                    font: Fonts.Font): INTEGER

NEW, ABSTRACT

Given string *s* at position *x*, *CharIndex* determines the index of the character which lies at position *pos*. All coordinates are in pixels. *Result = 0* means *pos* is at or left of the first character in *s*, *result = n - 1*, where *n* is the number of characters in string *s*, means *po*s is right of the last character in *s*.

Pre

font # NIL    20

PROCEDURE (rd: Rider) **LCharIndex** (x, pos: INTEGER; IN s: ARRAY OF SHORTCHAR;

                                                                    font: Fonts.Font): INTEGER

NEW, ABSTRACT

Given string *s* at position *x*, *CharIndex* determines the index of the character which lies at position *pos*. All coordinates are in pixels. *Result = 0* means *pos* is at or left of the first character in *s*, *result = n - 1*, where *n* is the number of characters in string *s*, means *po*s is right of the last character in *s*.

Pre

font # NIL    20

PROCEDURE (rd: Rider) **CharPos** (x, index: INTEGER; IN s: ARRAY OF CHAR;

                                                                    font: Fonts.Font): INTEGER

NEW, ABSTRACT

Given string *s* at position *x*, *CharPos* determines the position of character *index* in *s*. All coordinates are in pixels. The position of the left margin of the character is returned.

Pre

font # NIL    20

PROCEDURE (rd: Rider) **LCharPos** (x, index: INTEGER; IN s: ARRAY OF SHORTCHAR;

                                                                    font: Fonts.Font): INTEGER

NEW, ABSTRACT

Given string *s* at position *x*, *CharPos* determines the position of character *index* in *s*. All coordinates are in pixels. The position of the left margin of the character is returned.

Pre

font # NIL    20

TYPE **Frame**

ABSTRACT

A Frame is a mapper for a port. Every frame has its own coordinate system. All coordinates used for a frame are measured in universal units. Most frame operations forward to the frame's rider, i.e., they call the frame rider's corresponding procedure, and perform the necessary coordinate transformations (scaling between universal units and pixels, and a translation by the frame's origin).

A frame *f* translates from local universal coordinates to global pixel coordinates using the following transformation:

        x := (f.gx + x) DIV f.unit; y := (f.gy + y) DIV f.unit;    (* frame -> rider coordinates *)

The opposit transformation is:

        x := x * f.unit - f.gx; y := y * f.unit - f.gy;    (* rider -> frame coordinates *)

The rider's clipping rectangle is always set up such that drawing cannot occur outside of the frame, i.e., outside of the drawing view. Be careful if you change the rider's clipping rectangle (using the rider's *SetRect* method), since this introduces mutable state that you have to manage.

Frames are allocated by views.

Frames are used by views, for drawing and for mouse polling.

Frames are extended internally (*Views.Frame*).

**unit**-: INTEGER    unit > 0

The size of a pixel in universal units.

**dot-**: INTEGER    dot = point - point MOD unit

This value can be used as an approximation of *point*, rounded to a pixel. By using *dot* instead of *point*, ugly rounding errors can be avoided. For example, if you used *point* as the thickness of a line, and a pixel were slightly larger than *point*, the line might disappear altogether. Moreover, you may want to use a very thin line on the screen (about one pixel) as a hairline, but not have it become too small on a laser printer (where the frame's *unit* is much smaller than on screen). In these cases, *dot* comes handy.

**rider**-: Ports.Rider

Rider which links the frame to a port.

**gx-, gy-**: INTEGER    [units]

The frame's origin in global coordinates (i.e., relative to the port's upper-left corner), but in units instead of pixels. This is an exception from the rule that riders use pixels, while frames use units.

PROCEDURE (f: Frame) **ConnectTo** (p: Port)

NEW, EXTENSIBLE

Connects the frame to a port. All other frame procedures require a connected frame, i.e., *rider # NIL*. This precondition is not checked explicitly.

*ConnectTo* is used internally.

Post

p = NIL

    f.unit = 0

    f.rider = NIL

p # NIL

    f.unit = p.unit

    f.rider # NIL  &  f.rider.Base() = p

    f.dot = point - point MOD p.unit

PROCEDURE (f: Frame) **SetOffset** (gx, gy: INTEGER)

NEW, EXTENSIBLE

Sets the frame's origin, in global coordinates (i.e., relative to the port's upper-left corner), but in units instead of pixels. All local coordinates are relative to this origin. This method is only for internal use in the framework.

*SetOffset* is used internally.

Pre

f.rider # NIL    20

Post

f.gx = gx  &  f.gy = gy

PROCEDURE (f: Frame) **SaveRect** (l, t, r, b: INTEGER; OUT res: INTEGER)

NEW

Saves a rectangle (parallel to the coordinate axes) of width *r - l* and of height *b - t* in a background buffer, from where it can be restored later using *RestoreRect*. *SaveRect* must be balanced by *RestoreRect(l, t, r, b, disposeBuffer)*.

Calls to *SaveRect* may not be nested, and they may not occur during restoration of a view (-> *Views.View.Restore*). The purpose of *SaveRect/RestoreRect* is to act as temporary buffering mechanism during mouse tracking (->*Views.View.HandleCtrlMsg*).

 *res = 0* means that the call was successful, otherwise *RestoreRect* must not be called.

Pre

(l <= r) & (t <= b)    20

PROCEDURE (f: Frame) **RestoreRect** (l, t, r, b: INTEGER; dispose: BOOLEAN)

NEW

After a successful call to *SaveRect*, the same rectangle, i.e., its pixelmap contents as it was upon saving, can be restored with *RestoreRect*. *RestoreRect* can be called several times in succession; the last time with *dispose = disposeBuffer* and all other times with *dispose = keepBuffer*.

Pre

(l <= r) & (t <= b)    20

PROCEDURE (f: Frame) **DrawRect** (l, t, r, b, s: INTEGER; col: Color)

NEW

Draws a rectangle (parallel to the coordinate axes) of width *r - l* and of height *b - t*. If *s < 0*, the rectangle is filled with color *col*. Otherwise, the rectangle is drawn as an outline of thickness *s*. The outline is drawn *inside* of the rectangle. If *s = 0*, a very thin outline (hairline) is used.

Pre

(l <= r) & (t <= b)    20

(s >= 0) OR (s = fill)    21

PROCEDURE (f: Frame) **DrawOval** (l, t, r, b, s: INTEGER; col: Color)

NEW

Draws an ellipse (parallel to the coordinate axes) of width *r - l* and of height *b - t*. If *s < 0*, the ellipse is filled with color *col*. Otherwise, the ellipse is drawn as an outline of thickness *s*. The outline is drawn *inside* of the rectangle. If *s = 0*, a very thin outline (hairline) is used.

Pre

(l <= r) & (t <= b)    20

(s >= 0) OR (s = fill)    21

PROCEDURE (f: Frame) **DrawLine** (x0, y0, x1, y1, s: INTEGER; col: Color)

NEW

Draws a line from the point *(x0, y0)* to the point *(x1, y1)* of thickness *s* in color *col*:

Figure 5.  Line

If *s = 0*, a very thin outline (hairline) is used.

Pre

s >= 0    20

PROCEDURE (f: Frame) **DrawPath** (IN p: ARRAY OF Point; n, s: INTEGER; col: Color;

                                                                path: INTEGER)

NEW

Draws the path consisting of points *p[0] .. p[n - 1]* in color *col*. The nature of the path is given by parameter *path*. It can either be a polyline, a polygon, an open Bezier curve, or a closed Bezier curve. The polyline is the same that a sequence of *DrawLine* operations would generate. For a polygon, the n points define the mathematical region which will be outlined or filled. An open path with *n* points results in *n - 1* path pieces, a closed path with *n* points results in *n* path pieces.

Pre

n >= 0    20

n <= LEN(p)    21

(s = fill)  =>  (path = closedPoly) OR (path = closedBezier)    22

(s >= 0) OR (s = fill)    23

path IN {closedPoly, openPoly, closedBezier, openBezier}    25

path = openPoly

    n >= 2    20

path = closedPoly

    n >= 2    20

path = openBezier

    n >= 4    20

    n MOD 3 = 1    24

path = closedBezier

    n >= 3    20

    n MOD 3 = 0    24

PROCEDURE (f: Frame) **MarkRect** (l, t, r, b, s: INTEGER; mode: INTEGER; show: BOOLEAN)

NEW

Marks a rectangle (parallel to the coordinate axes) of width *r - l* and of height *b - t*. If *s < 0*, the rectangle is filled in some way, dependent on *mode*. Otherwise, the rectangle is drawn as an outline of thickness *s*. If *s = 0*, a very thin outline (hairline) is used. The outline is drawn *inside* of the rectangle.

The meaning of *mode* is implementation-dependent, but it must change the marked area in a visible way.* show* indicates whether the mark should be drawn or removed. Calling *MarkRect* with *show* and then directly afterwards with *hide* (otherwise with the same parameters) should re-establish exactly the state before the first call.

Pre

(l <= r) & (t <= b)    20

s >= 0    21

mode IN {invert, hilite, dim25, dim50, dim75}    22

PROCEDURE (f: Frame) **Scroll** (dx, dy: INTEGER)

NEW

Shifts the frame's area by vector *(dx, dy)*. Shifting occurs completely *within* the frame's rectangle, i.e., pixels outside of it are neither written nor read. The part of the rectangle which becomes newly exposed should be considered as undefined.

The purpose of *Scroll* is to speed up scrolling and editing operations by reusing existing pixel data instead of making the application redraw everything.

However, under special circumstances, this procedure may not actually copy pixel data, but cause the application to restore part of the rectangle instead anyway.

Warning: this operation may only be used on interactive ports, in order to update the screen display after a user manipulation.

Figure 6.  Effect of Scroll Operation

PROCEDURE (f: Frame) **SetCursor** (cursor: INTEGER)

NEW

Sets the cursor to the given value.

*SetCursor* is used in polling loops during mouse tracking.

Pre

cursor IN {arrowCursor..refCursor}    20

PROCEDURE (f: Frame) **Input** (OUT x, y: INTEGER; OUT modifiers: SET; OUT isDown: BOOLEAN)

NEW

Polls the current mouse location and tells whether the mouse button is currently pressed.

*Input* is used in polling loops during mouse tracking. In *modifiers*, the currently pressed modifier keys are returned, like *Controllers.doubleClick*, *Controllers.extend*, *Controllers.modify*, and possibly additional platform-specific modifiers.

PROCEDURE (f: Frame) **DrawString** (x, y: INTEGER; col: Color; IN s: ARRAY OF CHAR;

                                                                    font: Fonts.Font)

NEW

Draws string *s* in color *col* and font *font* with the base line at *y*.

Pre

font # NIL    20

PROCEDURE (f: Frame) **DrawSString** (x, y: INTEGER; col: Color; IN s: ARRAY OF SHORTCHAR;

                                                                    font: Fonts.Font)

NEW

Draws short string *s* in color *col* and font *font* with the base line at *y*.

Pre

font # NIL    20

PROCEDURE (f: Frame) **CharIndex** (x, pos: INTEGER; IN s: ARRAY OF CHAR;

                                                                    font: Fonts.Font): INTEGER

NEW

Given string *s* at position *x*, *CharIndex* determines the index of the character which lies at position *pos*. *Result = 0* means *pos* is at or left of the first character in *s*, *result = n - 1*, where *n* is the number of characters in string *s*, means *po*s is right of the last character in *s*.

Pre

font # NIL    20

PROCEDURE (f: Frame) **SCharIndex** (x, pos: INTEGER; IN s: ARRAY OF SHORTCHAR;

                                                                    font: Fonts.Font): INTEGER

Given short string *s* at position *x*, *CharIndex* determines the index of the character which lies at position *pos*. *Result = 0* means *pos* is at or left of the first character in *s*, *result = n - 1*, where *n* is the number of characters in string *s*, means *po*s is right of the last character in *s*.

Pre

font # NIL    20

PROCEDURE (f: Frame) **CharPos** (x, index: INTEGER; IN s: ARRAY OF CHAR;

                                                                    font: Fonts.Font): INTEGER

Given string *s* at position *x*, *CharPos* determines the position of character *index* in *s*. The position of the left margin of the character is returned.

Pre

font # NIL    20

PROCEDURE (f: Frame) **SCharPos** (x, index: INTEGER; IN s: ARRAY OF SHORTCHAR;

                                                                    font: Fonts.Font): INTEGER

Given short string *s* at position *x*, *CharPos* determines the position of character *index* in *s*. The position of the left margin of the character is returned.

Pre

font # NIL    20

VAR **background**: Color    background >= 0

This variable denotes the color which is used for the background of a window.

VAR **dialogBackground**: Color    dialogBackground >= 0

This variable denotes the color which is used for the background of a dialog.

PROCEDURE **IsPrinterPort** (p: Port): BOOLEAN

Determines whether a port represents a printer.

PROCEDURE **RGBColor** (red, green, blue: INTEGER): Color

Constructs a *Color* out of the red, green, and blue components.

Pre

0 <= red < 256    20

0 <= green < 256    21

0 <= blue < 256    22

Post

result = blue * 65536  +  green * 256  + red

