MODULE PropertiesExtBase;
(*
   Cross-module workout for the `Properties` slice.

   Builds a concrete subclass of `Properties.Property`, overrides
   the ABSTRACT `IntersectWith`, populates the inline-record
   `style` field of a `StdProp`, and reads it back through the
   property's pointer alias.

   Exercises:
   - extending `Properties.PropertyDesc` (ABSTRACT) with a leaf;
   - overriding `IntersectWith` (ABSTRACT) and confirming the
     override fires through the vtable;
   - reading/writing through the inline `style: RECORD val,
     mask: SET END` field on `StdPropDesc` — pins the new
     `__anon_inline_` Named-type plumbing.
*)

    IMPORT Properties, Fonts;

    TYPE
        (** Leaf property — carries one extra knob and overrides
            IntersectWith to a no-op so the abstract base is
            satisfied. *)
        MyPropDesc* = RECORD (Properties.PropertyDesc)
            tag*: INTEGER
        END;
        MyProp* = POINTER TO MyPropDesc;


    PROCEDURE (p: MyPropDesc) IntersectWith* (q: Properties.Property; OUT equal: BOOLEAN);
    BEGIN
        equal := TRUE;
        p.tag := p.tag + 1
    END IntersectWith;


    PROCEDURE Run* (): INTEGER;
        VAR my: MyProp; base: Properties.Property;
            std: Properties.StdProp;
            eq: BOOLEAN;
            r: INTEGER;
    BEGIN
        (* Stage 1: leaf MyProp dispatches IntersectWith via the
           Properties.Property base pointer. *)
        NEW(my);
        my.tag := 0;
        my.known := {0, 2};
        my.valid := {0};
        base := my;
        base.IntersectWith(base, eq);          (* virtual dispatch -> MyPropDesc.IntersectWith *)
        IF ~eq THEN RETURN -1 END;
        IF my.tag # 1 THEN RETURN -2 END;

        (* Stage 2: StdProp with the inline-record `style` field.
           Round-trips val + mask SETs through the inner record. *)
        NEW(std);
        std.size := 12;
        std.weight := 700;
        std.style.val  := {0, 3};
        std.style.mask := {1, 2, 3};
        std.typeface := "Helvetica";
        ASSERT(0 IN std.style.val,  20);
        ASSERT(3 IN std.style.val,  21);
        ASSERT(1 IN std.style.mask, 22);

        (* Pack a value that proves each stage fired:
             my.tag                              =     1     ->  1000000
             std.size                            =    12     ->     1200
             std.weight                          =   700     ->      700
             0 IN std.style.val (TRUE)           ->     +10
             3 IN std.style.val (TRUE)           ->      +1
             encode: 1*1000000 + 12*100 + 7 + 10 + 1
                   = 1000000 + 1200 + 700 + 10 + 1
                   = 1001911 *)
        r := my.tag * 1000000
           + std.size * 100
           + std.weight;
        IF 0 IN std.style.val  THEN r := r + 10 END;
        IF 3 IN std.style.val  THEN r := r + 1 END;
        RETURN r
    END Run;

END PropertiesExtBase.
