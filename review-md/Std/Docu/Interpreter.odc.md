**StdInterpreter**

DEFINITION StdInterpreter;

END StdInterpreter.

Module *StdInterpreter* implements a plug-in service for BlackBox: an interpreter for some forms of Component Pascal procedure calls. The text string to be interpreted must conform to the following syntax:

Command = Call { ";" Call }.

Call = ModuleName "." ProcedureName [ "(" Parameter { "," Parameter } ")" ].

Parameter = Integer | String.

Integer = Decimal | Hex.

Decimal = [ "-" ] Digit { Digit }.

Hex = [ "-" ] HexDigit { HexDigit } "H".

String = " ' " { Char } " ' ".

The calls are executed in the given sequence. Parameters corresponding to integers must be of type INTEGER. Parameters corresponding to strings must be value or IN parameters of type ARRAY OF CHAR. The called procedures must not return a value.

For example, the following string can be used to call the three given procedures:

     "TurtleDraw.GotoPos(35, 587); TurtleDraw.ShowPen; TurtleDraw.WriteString('Hello')".



    PROCEDURE GotoPos (x, y: INTEGER);

    PROCEDURE ShowPen;

    PROCEDURE WriteText (IN text: ARRAY OF CHAR);

Such statement sequences are used mainly in menu command configurations and in control properties.

Module *StdInterpreter* is installed during startup of BlackBox. Its service is made available through the procedure *Dialog.Call*.

