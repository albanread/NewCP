**Oberon by Example: ObxRatCalc**

This example implements a simplifier for rational numbers of arbitrary precision. Expressions to be simplified are built up from integer numbers, the operators "+", "-", "*", "/", "^", and parentheses. Exponents ("^") must be integer numbers and their absolute value must not exceed MAX(INTEGER).

The result is either an integer or a rational number (command *ObxRatCalc.Simplify*), or a floating point number (*ObxRatCalc.Approximate*).

The implementation of *ObxRatCalc* uses the [*<u>Integers </u>*<u>module</u>](../../System/Docu/Integers.odc.md), which provides a data type for arbitrary precision integers and operations on them. The calculator works on arithmetic expressions, such as the following:

Source Expression        ObxRatCalc.Simplify        ObxRatCalc.Approximate

((100 + 27 - 2) / (-5 * (2 + 3))) ^ (-3)    =    -1 / 125    =    -8*10^-3

999999999 * 999999999 / 81    =    12345678987654321

1234567890 / 987654321    =    137174210 /  109739369    = 1.2499999886093750001423828...

30.2 + 60.5    =    907 / 10    =    90.7

Note that the undo/redo mechanism can be used after an evaluation.

Syntax of input:

    expression := ["-"] {term addop} term.

    term := {factor mulop} factor.

    factor := ("(" expression ")" | integer) ["^" factor].

    integer := digit {digit} [ "." digit {digit} ].

    addop := "+".

    mulop := "*" | "/".

Menu entries in "Obx" menu:

    "Simplify"    ""    "ObxRatCalc.Simplify"    "TextCmds.SelectionGuard"

    "Approximate"    ""    "ObxRatCalc.Approximate"    "TextCmds.SelectionGuard"

[<u>ObxRatCalc  sources</u>](../Mod/RatCalc.odc.md)

