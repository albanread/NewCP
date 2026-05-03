**Math**

DEFINITION Math;

    PROCEDURE Pi (): REAL;

    PROCEDURE Eps (): REAL;

    PROCEDURE Sqrt (x: REAL): REAL;

    PROCEDURE Exp (x: REAL): REAL;

    PROCEDURE Ln (x: REAL): REAL;

    PROCEDURE Log (x: REAL): REAL;

    PROCEDURE Power (x, y: REAL): REAL;

    PROCEDURE IntPower (x: REAL; n: INTEGER): REAL;

    PROCEDURE Sin (x: REAL): REAL;

    PROCEDURE Cos (x: REAL): REAL;

    PROCEDURE Tan (x: REAL): REAL;

    PROCEDURE ArcSin (x: REAL): REAL;

    PROCEDURE ArcCos (x: REAL): REAL;

    PROCEDURE ArcTan (x: REAL): REAL;

    PROCEDURE ArcTan2 (y, x: REAL): REAL;

    PROCEDURE Sinh (x: REAL): REAL;

    PROCEDURE Cosh (x: REAL): REAL;

    PROCEDURE Tanh (x: REAL): REAL;

    PROCEDURE ArcSinh (x: REAL): REAL;

    PROCEDURE ArcCosh (x: REAL): REAL;

    PROCEDURE ArcTanh (x: REAL): REAL;

    PROCEDURE Sign (x: REAL): REAL;

    PROCEDURE Floor (x: REAL): REAL;

    PROCEDURE Ceiling (x: REAL): REAL;

    PROCEDURE Trunc (x: REAL): REAL;

    PROCEDURE Frac (x: REAL): REAL;

    PROCEDURE Round (x: REAL): REAL;

    PROCEDURE Mantissa (x: REAL): REAL;

    PROCEDURE Exponent (x: REAL): INTEGER;

    PROCEDURE Real (m: REAL; e: INTEGER): REAL;



END Math.

Module *Math* is a basic library for numerical computations. It offers the most frequently used functions and constants. For some additional functions, a transformation in terms of functions available in module *Math* is given below.

**Constants**

PROCEDURE **Pi** (): REAL

Returns an approximation of the value of *pi*.

Post

result = 3.141592...

PROCEDURE **Eps** (): REAL

Returns the *machine epsilon* for *REAL*. The machine epsilon *eps* is the smallest floating point number such that the sum 1.0 + *eps *can be represented *exactly* in a *REAL* variable. Usually, the machine epsilon is *2-m* where *m* is the number of digits used to represent the mantissa of a floating point number. Note, that *Eps()* is not necessarily the smallest floating point number with the property that its sum with 1.0 is greater than 1.0, and note also, that most floating point processors offer a higher internal precision than what can be represented within *REAL* variables.

**Powers and logarithms**

PROCEDURE **Sqrt** (x: REAL): REAL

Returns the square root of *x*.

Pre

x >= 0.0    20

Post

result >= 0.0

x = INF

    result = INF

PROCEDURE **Exp** (x: REAL): REAL

Returns *ex*.

Post

result > 0.0

x = INF

    result = INF

x = -INF

    result = 0.0

PROCEDURE **Ln** (x: REAL): REAL

Returns the natural logarithm of *x*.

Pre

x >= 0.0    20

Post

x = 0.0

    result = -INF

x = INF

    result = INF

PROCEDURE **Log** (x: REAL): REAL

Returns the logarithm to the basis 10 of *x*.

Pre

x >= 0.0

Post

x = 0.0

    result = -INF

x = INF

    result = INF

PROCEDURE **Power** (x, y: REAL): REAL

Returns *xy*.

Pre

x >= 0.0    20

x # 0.0  OR  y # 0.0    21

x # INF  OR  y # 0.0    22

x # 1.0  OR  ABS(y) # INF    23

Post

x = 0.0  &  y < 0.0

    result = INF

x = 0.0  &  y > 0.0

    result = 0.0

x = INF  &  y > 0.0

    result = INF

x = INF  &  y < 0.0

    result = 0.0

x > 1.0  &  y = INF

    result = INF

x > 1.0  &  y = -INF

    result = 0.0

x < 1.0  &  y = INF

    result = 0.0

x < 1.0  &  y = -INF

    result = INF

PROCEDURE **IntPower** (x: REAL; n: INTEGER): REAL

Returns *xn*. The procedure is optimized for integer values of *n*. *IntPower(0, 0)* yields *1*. If the result is too large, *INF* is returned.

**Trigonometric and hyperbolic functions**

The arguments for all trigonometric and hyperbolic functions must be given in radians, and the inverse trigonometric and hyperbolic functions are calculated in radians (1 radian = 180/pi degrees). At the end of this section, a transformation table for additional trigonometric and hyperbolic functions is given which do not belong to the interface of the *Math* module, but which can easily be written in terms of the exported functions.

PROCEDURE **Sin** (x: REAL): REAL

Returns the sine of *x*.

Pre

ABS(x) # INF    20

Post

-1.0 <= result <= 1.0

PROCEDURE **Cos** (x: REAL): REAL

Returns the cosine of *x*.

Pre

ABS(x) # INF    20

Post

-1.0 <= result <= 1.0

PROCEDURE **Tan** (x: REAL): REAL

Returns the tangent of *x*. The *Tan* can be computed for all possible *REAL* arguments except *±INF*.

Pre

ABS(x) # INF    20

PROCEDURE **ArcSin** (x: REAL): REAL

Returns the arcus sine of *x*.

Pre

-1.0 <= x <= 1.0    20

Post

-pi/2.0 <= result <= pi/2.0

PROCEDURE **ArcCos** (x: REAL): REAL

Returns the arcus cosine of *x*.

Pre

-1.0 <= x <= 1.0    20

Post

0.0 <= result <= pi

PROCEDURE **ArcTan** (x: REAL): REAL

Returns the arcus tangent of *x*.

Post

-pi/2.0 <= result <= pi/2.0

x = INF

    result = pi/2.0

x = -INF

    result = -pi/2.0

PROCEDURE **ArcTan2** (y, x: REAL): REAL

Returns the quadrant-correct principal value of the argument of the complex number *x + iy* in the range *(-pi, pi]*.

Pre

y # 0  OR  x # 0    20

ABS(y) # INF  OR  ABS(x)  # INF    21

Post

-pi < result <= pi

ABS(y) # INF  &  x = INF

    result = 0

y = INF  &  ABS(x) # INF

    result = pi/2.0

ABS(y) # INF  &  x = -INF

    result = pi

y = -INF  &  ABS(x) # INF

    result = -pi/2.0

PROCEDURE **Sinh** (x: REAL): REAL

Returns the hyperbolic sine of *x*.

Post

x = INF

    result = INF

x = -INF

    result = -INF

PROCEDURE **Cosh** (x: REAL): REAL

Returns the hyperbolic cosine of *x*.

Post

1.0 <= result

x = INF

    result = INF

x = -INF

    result = INF

PROCEDURE **Tanh** (x: REAL): REAL

Returns the hyperbolic tangent of *x*.

Post

-1.0 <= result <= 1.0

x = INF

    result = 1.0

x = -INF

    result = -1.0

PROCEDURE **ArcSinh** (x: REAL): REAL;

Returns the inverse hyperbolic sine of *x*.

Post

x = INF

    result = INF

x = -INF

    result = -INF

PROCEDURE **ArcCosh** (x: REAL): REAL;

Returns the inverse hyperbolic cosine of *x*.

Pre

1.0 <= x    20

Post

0.0 <= result

x = INF

    result = INF

PROCEDURE **ArcTanh** (x: REAL): REAL;

Returns the inverse hyperbolic tangent of *x*.

Pre

-1.0 <= x <= 1.0    20

Post

x = 1.0

    result = INF

x = -1.0

    result = -INF

Below you find the definition of additional trigonometric and hyperbolic functions in terms of the exported functions. Their pre and post conditions can be induced from the pre and post conditions of the involved functions.

    **    Cot **(x) = 1.0 / Math.Tan(x)

        **Csc** (x) = 1.0 / Math.Sin(x)

        **Sec** (x) = 1.0 / Math.Cos(x)

        **ArcCot** (x) = Math.Pi() / 2.0 - Math.ArcTan(x)

        **ArcCsc** (x) = Math.ArcSin(1.0 / x)

        **ArcSec** (x) = Math.ArcCos(1.0 / x)

        **Coth **(x) = 1.0 / Math.Tanh(x)

        **Csch** (x) = 1.0 / Math.Sinh(x)

        **Sech** (x) = 1.0 / Math.Cosh(x)

        **ArcCoth** (x) = Math.ArcTanh(1.0 / x)

        **ArcCsch** (x) = Math.ArcSinh(1.0 / x)

        **ArcSech** (x) = Math.ArcCosh(1.0 / x)

**Miscellaneous functions**

You could easily implement the functions *Sign*, *Floor*, *Ceiling*, *Trunc*, *Frac*, and *Round* yourself, using the *ENTIER* standard function. They are provided here for convenience, and in versions which return *REAL* typed results, so that no conversions between reals and integers become necessary if they are used in real expressions.

PROCEDURE **Sign** (x: REAL): REAL

Returns the sign of *x*, that is *1.0* if *x > 0.0*, *-1.0* if *x < 0.0* and *0.0* if *x = 0.0*.

Post

result IN {-1.0, 0.0, 1.0}

PROCEDURE **Floor** (x: REAL): REAL

Returns the greatest integer less than or equal to *x. *This function is identical to *ENTIER*, except that it returns a *REAL* type value.

Post

x = INF

    result = INF

x = -INF

    result = -INF

PROCEDURE **Ceiling** (x: REAL): REAL

Returns the smallest integer greater than or equal to *x*.

Post

x = INF

    result = INF

x = -INF

    result = -INF

PROCEDURE **Trunc** (x: REAL): REAL

*Trunc* truncates its argument to the next nearest integer towards zero.

Post

x = INF

    result = INF

x = -INF

    result = -INF

PROCEDURE **Frac** (x: REAL): REAL

*Frac* is the fractional part of the argument. The following equation holds: *x = Trunc(x) + Frac(x)*.

Pre

x # INF  &  x # -INF    20

PROCEDURE **Round** (x: REAL): REAL

Same as *Floor(x + 0.5)*.

PROCEDURE **Mantissa** (x: REAL): REAL

Returns the mantissa of *x*.

Post

1.0 <= ABS(result) < 2.0  OR  result = 0.0

x = INF

    result = 1.0

x = -INF

    result = -1.0

x = not-a-number

    ABS(result) > 1.0

PROCEDURE **Exponent** (x: REAL): INTEGER

Returns the exponent of *x* such that *x = Mantissa(x) ** 2*Exponent(x)*. If *x* represents *±INF* or if *x* is *not-a-number*, then *MAX(INTEGER)* is returned.

PROCEDURE **Real** (m: REAL; e: INTEGER): REAL

Returns *m * 2e*. If the argument *e* is *MAX(INTEGER)*, then *INF* or *not-a-number* is returned where *±INF* is returned if *m = ± 1.0 *and *not-a-number* otherwise. Thus for any real *x* the equation

*    x = Real(Mantissa(x), Exponent(x))*

holds.

Note: the normal arithmetic operations of the language Component Pascal, and the other operations implemented in this module, never produce the IEEE *not-a-number* value.

Note: *Real(0, 0) = 0*.

Pre

(1.0 <= ABS(m) < 2.0)  OR  (m = 0.0)    20

Post

m = 0.0

    result = 0.0

