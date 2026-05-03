MODULE CaseWith;
(* Exercises CASE statements and WITH type-guard statements. *)

IMPORT TypeExt;

TYPE
    Shape* = RECORD
        kind*: INTEGER
    END;

    (* kind codes *)
CONST
    Circle*   = 1;
    Square*   = 2;
    Triangle* = 3;

(* CASE on integer: return number of sides. *)
PROCEDURE Sides*(kind: INTEGER): INTEGER;
    VAR n: INTEGER;
BEGIN
    CASE kind OF
      Circle:   n := 0
    | Square:   n := 4
    | Triangle: n := 3
    ELSE        n := -1
    END;
    RETURN n
END Sides;

(* CASE on CHAR: classify ASCII character. *)
PROCEDURE CharClass*(ch: CHAR): INTEGER;
    VAR cls: INTEGER;
BEGIN
    CASE ch OF
      "a".."z": cls := 1     (* lower *)
    | "A".."Z": cls := 2     (* upper *)
    | "0".."9": cls := 3     (* digit *)
    ELSE        cls := 0     (* other *)
    END;
    RETURN cls
END CharClass;

(* WITH: dispatch on dynamic type of a TypeExt.Animal variable. *)
PROCEDURE Describe*(VAR a: TypeExt.Animal): INTEGER;
    VAR result: INTEGER;
BEGIN
    WITH a: TypeExt.Bird DO
        IF a.canFly THEN result := 10 ELSE result := 11 END
    | a: TypeExt.Fish DO
        result := 20 + a.fins
    ELSE
        result := a.legs
    END;
    RETURN result
END Describe;

BEGIN
END CaseWith.
