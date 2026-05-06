MODULE Arrays;

(* Test module for array indexing (Step 7). *)

TYPE
    Point* = RECORD
        x*, y*: INTEGER
    END;

VAR
    data: ARRAY 5 OF INTEGER;
    grid: ARRAY 3 OF ARRAY 4 OF INTEGER;
    points: ARRAY 4 OF Point;

PROCEDURE SetElem*(idx: INTEGER; val: INTEGER);
BEGIN
    data[idx] := val
END SetElem;

PROCEDURE GetElem*(idx: INTEGER): INTEGER;
BEGIN
    RETURN data[idx]
END GetElem;

PROCEDURE SumAll*(): INTEGER;
VAR i, s: INTEGER;
BEGIN
    s := 0;
    i := 0;
    WHILE i < 5 DO
        s := s + data[i];
        INC(i)
    END;
    RETURN s
END SumAll;

PROCEDURE SetPoint*(idx, x, y: INTEGER);
BEGIN
    points[idx].x := x;
    points[idx].y := y
END SetPoint;

PROCEDURE GetX*(idx: INTEGER): INTEGER;
BEGIN
    RETURN points[idx].x
END GetX;

PROCEDURE GetY*(idx: INTEGER): INTEGER;
BEGIN
    RETURN points[idx].y
END GetY;

END Arrays.
