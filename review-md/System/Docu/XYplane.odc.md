**XYplane**

DEFINITION XYplane;

    CONST erase = 0; draw = 1;

    VAR X, Y, W, H: INTEGER;

    PROCEDURE Open;

    PROCEDURE Dot (x, y, mode: INTEGER);

    PROCEDURE IsDot (x, y: INTEGER): BOOLEAN;

    PROCEDURE ReadKey (): CHAR;

    PROCEDURE Clear;

END XYplane.

This module is provided for compatibility with the book "Programming in Oberon" by Reiser/Wirth. It is useful when learning the language. It is not recommended for use in production programs.

CONST **erase**

This value can be passed to parameter *mode* in procedure *Dot*. It indicates that a white dot should be placed at the given coordinates.

CONST **draw**

This value can be passed to parameter *mode* in procedure *Dot*. It indicates that a black dot should be placed at the given coordinates.

VAR **X, Y, W, H**

These values define the rectangle in which drawing occurs. *(X, Y)* is the lower-left corner of the rectangle, *(W, H)* its size. In BlackBox, *(X, Y)* is always *(0, 0)*. Unlike the port model of BlackBox, *XYplane* has its origin at the lower-left corner of the drawing area, and positive Y values *above* the origin.

PROCEDURE **Open**

Opens a new window for drawing. The window's contents is cleared to white.

PROCEDURE **Dot** (x, y, mode: INTEGER)

Draws a white dot (*mode = erase*) or a black dot (*mode = draw*).

PROCEDURE **IsDot** (x, y: INTEGER): BOOLEAN

Returns whether the dot at *(x, y)* is white (*FALSE*) or black (*TRUE*).

PROCEDURE **ReadKey** (): CHAR

If a key has been pressed, it is returned as result. Otherwise, *0X* is returned.

PROCEDURE **Clear**

Erases the whole drawing area (setting it to white).

