MODULE Records;
(* Record types with exported fields; procedures that operate on records. *)

TYPE
    Point* = RECORD
        x*, y* : INTEGER
    END;

    Rect* = RECORD
        left*, top*, right*, bottom* : INTEGER
    END;

VAR instanceCount- : INTEGER;   (* read-only export: incremented internally *)

PROCEDURE SetPoint*(VAR p: Point; x, y: INTEGER);
BEGIN
    p.x := x;
    p.y := y
END SetPoint;

PROCEDURE Translate*(VAR p: Point; dx, dy: INTEGER);
BEGIN
    INC(p.x, dx);
    INC(p.y, dy)
END Translate;

PROCEDURE Width*(r: Rect): INTEGER;
BEGIN
    RETURN r.right - r.left
END Width;

PROCEDURE Height*(r: Rect): INTEGER;
BEGIN
    RETURN r.bottom - r.top
END Height;

PROCEDURE Contains*(r: Rect; p: Point): BOOLEAN;
BEGIN
    RETURN (p.x >= r.left) & (p.x < r.right)
         & (p.y >= r.top)  & (p.y < r.bottom)
END Contains;

BEGIN
    instanceCount := 0
END Records.
