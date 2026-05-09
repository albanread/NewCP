MODULE AbstractDispatch;
(* Pointer-aliased OOP: ABSTRACT base + concrete subclasses + virtual
   dispatch through the abstract pointer. The dominant CP idiom for
   `Files.Reader / .File / .Directory` and every other Host* port. *)

TYPE
    ShapeDesc*  = ABSTRACT RECORD END;
    Shape*      = POINTER TO ShapeDesc;

    SquareDesc* = RECORD (ShapeDesc) side*: INTEGER END;
    Square*     = POINTER TO SquareDesc;

    CircleDesc* = RECORD (ShapeDesc) r*: INTEGER END;
    Circle*     = POINTER TO CircleDesc;

(* Abstract method: must be overridden by concrete subclasses. *)
PROCEDURE (s: ShapeDesc) Area*(): INTEGER, NEW, ABSTRACT;

(* Concrete override on Square. *)
PROCEDURE (s: SquareDesc) Area*(): INTEGER;
BEGIN RETURN s.side * s.side END Area;

(* Concrete override on Circle (uses 3 as a stand-in for pi). *)
PROCEDURE (c: CircleDesc) Area*(): INTEGER;
BEGIN RETURN 3 * c.r * c.r END Area;

(* Caller takes the abstract pointer base. Should virtual-dispatch to
   the concrete override based on the dynamic type of `s`. *)
PROCEDURE AreaOf(s: Shape): INTEGER;
BEGIN RETURN s.Area() END AreaOf;

PROCEDURE TestSquare*(): INTEGER;
    VAR sq: Square; sh: Shape;
BEGIN
    NEW(sq); sq.side := 5;
    sh := sq;
    RETURN AreaOf(sh)         (* expect 25 *)
END TestSquare;

PROCEDURE TestCircle*(): INTEGER;
    VAR ci: Circle; sh: Shape;
BEGIN
    NEW(ci); ci.r := 4;
    sh := ci;
    RETURN AreaOf(sh)         (* expect 48 *)
END TestCircle;

END AbstractDispatch.
