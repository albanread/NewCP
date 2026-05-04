MODULE Methods;
(* Bound procedures (methods) and virtual dispatch.
   Shape is an extensible base record with two NEW extensible methods.
   Circle extends Shape and overrides both methods.
   GetArea is NEW on Circle only (no override).
   MakeCircle is a plain procedure that initialises a Circle.
*)

TYPE
    Shape* = EXTENSIBLE RECORD
        x*, y* : INTEGER
    END;

    Circle* = RECORD (Shape)
        r* : INTEGER
    END;

(* NEW extensible bound procedure on Shape — slot 0 *)
PROCEDURE (s: Shape) GetX*(): INTEGER, NEW, EXTENSIBLE;
BEGIN
    RETURN s.x
END GetX;

(* NEW extensible bound procedure on Shape — slot 1 *)
PROCEDURE (s: Shape) GetY*(): INTEGER, NEW, EXTENSIBLE;
BEGIN
    RETURN s.y
END GetY;

(* Override of Shape.GetX in Circle — reuses slot 0 *)
PROCEDURE (c: Circle) GetX*(): INTEGER;
BEGIN
    RETURN c.x + c.r
END GetX;

(* NEW bound procedure on Circle — slot 2 *)
PROCEDURE (c: Circle) GetR*(): INTEGER, NEW;
BEGIN
    RETURN c.r
END GetR;

PROCEDURE MakeCircle*(VAR c: Circle; x, y, r: INTEGER);
BEGIN
    c.x := x;
    c.y := y;
    c.r := r
END MakeCircle;

BEGIN
END Methods.
