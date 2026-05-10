MODULE WithProbe;
(* Verify the `WITH … DO … ELSE … END` type-guard statement dispatches
   correctly at runtime.  Two record types extend a common abstract
   base; a procedure receives the base by value-pointer and uses WITH
   to narrow to one of the extensions, calling the kind-specific
   accessor. *)

    TYPE
        AnimalDesc* = ABSTRACT RECORD
            id*: INTEGER
        END;
        Animal* = POINTER TO AnimalDesc;

        DogDesc* = RECORD (AnimalDesc)
            barkVolume*: INTEGER
        END;
        Dog* = POINTER TO DogDesc;

        CatDesc* = RECORD (AnimalDesc)
            purrPitch*: INTEGER
        END;
        Cat* = POINTER TO CatDesc;


    (** Accept a base Animal pointer; if it's actually a Dog, return
        its barkVolume; if it's a Cat, return -purrPitch; else 0. *)
    PROCEDURE Identify* (a: Animal): INTEGER;
        VAR result: INTEGER;
    BEGIN
        result := 0;
        WITH a: Dog DO
            result := a.barkVolume
        | a: Cat DO
            result := -a.purrPitch
        ELSE
            result := 0
        END;
        RETURN result
    END Identify;

    (** Allocate one Dog (barkVolume = 88) and one Cat (purrPitch = 7),
        run Identify on each plus a NIL animal, and return a packed
        verifier:
          (dogResult * 1000) + ((-catResult) * 10) + nilResult
        Expect 88_070 = (88 × 1000) + (7 × 10) + 0. *)
    PROCEDURE Run* (): INTEGER;
        VAR
            d: Dog;
            c: Cat;
            n: Animal;
            dogResult, catResult, nilResult: INTEGER;
    BEGIN
        NEW(d); d.id := 1; d.barkVolume := 88;
        NEW(c); c.id := 2; c.purrPitch := 7;
        n := NIL;

        dogResult := Identify(d);
        catResult := Identify(c);
        nilResult := Identify(n);

        RETURN (dogResult * 1000) + ((-catResult) * 10) + nilResult
    END Run;

END WithProbe.
