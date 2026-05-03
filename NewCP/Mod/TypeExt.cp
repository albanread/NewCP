MODULE TypeExt;
(* Type extension (record inheritance) and polymorphic procedures. *)

TYPE
    Animal* = RECORD
        legs* : INTEGER
    END;

    Bird* = RECORD (Animal)
        canFly* : BOOLEAN
    END;

    Fish* = RECORD (Animal)
        fins* : INTEGER
    END;

PROCEDURE Init*(VAR a: Animal; legs: INTEGER);
BEGIN
    a.legs := legs
END Init;

PROCEDURE IsQuadruped*(a: Animal): BOOLEAN;
BEGIN
    RETURN a.legs = 4
END IsQuadruped;

PROCEDURE MakeBird*(VAR b: Bird; canFly: BOOLEAN);
BEGIN
    b.legs   := 2;
    b.canFly := canFly
END MakeBird;

PROCEDURE MakeFish*(VAR f: Fish; fins: INTEGER);
BEGIN
    f.legs := 0;
    f.fins := fins
END MakeFish;

BEGIN
END TypeExt.
