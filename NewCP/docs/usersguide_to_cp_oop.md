# User's guide to OOP in Component Pascal (NewCP)

This guide is for programmers writing Component Pascal in NewCP. It covers
the object-oriented features the compiler currently supports end-to-end:
record types, pointer aliases, bound procedures (methods), inheritance,
virtual dispatch, abstract methods, and runtime type tests.

If you've used Java, C#, or Go: most of the same intuitions apply, but
the *syntax* is older. CP's OOP idiom predates the modern C-style
notation — methods are declared as procedures with a *receiver*
parameter rather than nested inside a class.

---

## 1. Records and pointer aliases

The fundamental object type in CP is the **record**. A record is a
struct: a fixed set of named fields.

```cp
TYPE
    BoxDesc* = RECORD
        value*: INTEGER
    END;
```

The `*` after a name **exports** it (makes it visible to other modules).

To use a record by reference — which is what you want for any object
with identity, inheritance, or shared state — you wrap it in a
**pointer alias**:

```cp
TYPE
    BoxDesc* = RECORD value*: INTEGER END;
    Box*     = POINTER TO BoxDesc;
```

This is the **dominant CP idiom**. The convention — followed by the
entire BlackBox standard library — is to name the record type
`<Name>Desc` and the pointer type `<Name>`. So users almost always
work with `Box`, not `BoxDesc`.

```cp
VAR b: Box;
NEW(b);              (* allocate on the heap *)
b.value := 42;       (* automatic dereference *)
```

You don't write `b^.value` — CP automatically dereferences a pointer
when you select a field on it.

---

## 2. Methods (bound procedures)

A method is just a procedure with an extra **receiver parameter** in
parentheses before the name.

```cp
PROCEDURE (b: BoxDesc) Get*(): INTEGER, NEW;
BEGIN RETURN b.value END Get;

PROCEDURE (b: BoxDesc) Set*(v: INTEGER), NEW;
BEGIN b.value := v END Set;
```

Read this as: "a procedure named `Get` bound to `BoxDesc`, where inside
the body `b` refers to the receiver (the `self` / `this`)."

The trailing `NEW` keyword declares this as a brand-new method (not an
override). You'll see why this matters in §4.

To call a method, use dotted notation on a pointer or record:

```cp
VAR b: Box;
NEW(b);
b.Set(42);              (* method call *)
RETURN b.Get()          (* returns 42 *)
```

The receiver in the *declaration* can be either a value-typed record
(`b: BoxDesc`) or a pointer (`b: Box`). Both forms work. In practice
the value-typed form is more common in BlackBox code, because the
caller goes through the pointer alias anyway.

---

## 3. Constructing objects with `NEW`

`NEW(p)` allocates a record on the heap and stores its pointer in `p`.

```cp
VAR b: Box;
NEW(b);
```

After `NEW(b)`:
- All numeric fields are zero, booleans are `FALSE`, pointer fields are
  `NIL`.
- The block carries a hidden type tag (TypeDesc pointer) so the runtime
  can do virtual dispatch and garbage collection.
- The garbage collector will reclaim the block when no live pointer
  refers to it. There is no explicit `DISPOSE` — CP is a managed
  language.

You don't initialize fields in `NEW`; assign them afterwards. The
common pattern is a constructor procedure:

```cp
PROCEDURE NewBox*(v: INTEGER): Box;
    VAR b: Box;
BEGIN
    NEW(b); b.value := v; RETURN b
END NewBox;
```

---

## 4. Inheritance: extending records

A record can **extend** another by naming the base in parentheses:

```cp
TYPE
    Animal* = RECORD
        legs*: INTEGER
    END;

    Bird* = RECORD (Animal)
        canFly*: BOOLEAN
    END;
```

`Bird` now has both `legs` (inherited) and `canFly`. A `Bird` value
fits everywhere an `Animal` value is expected, and a `POINTER TO Bird`
fits everywhere a `POINTER TO Animal` is expected.

### EXTENSIBLE vs final

By default, a record cannot be extended. To allow subclassing, mark
the base `EXTENSIBLE`:

```cp
TYPE Shape* = EXTENSIBLE RECORD x*, y*: INTEGER END;
```

Or use `ABSTRACT` (see §6) for a base that can never be instantiated
directly.

> **Tip:** in modern CP-style code that uses pointer aliases, the
> *record* is what's marked `EXTENSIBLE` or `ABSTRACT`, not the
> pointer type. The pointer is just a reference; extensibility is a
> property of the underlying record.

### Method overriding

In a subtype, declare the same-named method **without** `NEW`. The
compiler treats it as an override and the runtime dispatches to it
when the dynamic type is the subtype:

```cp
TYPE
    Shape*  = EXTENSIBLE RECORD x*, y*: INTEGER END;
    Circle* = RECORD (Shape) r*: INTEGER END;

PROCEDURE (s: Shape) GetX*(): INTEGER, NEW, EXTENSIBLE;
BEGIN RETURN s.x END GetX;

(* Override of Shape.GetX — same name, same signature, no NEW *)
PROCEDURE (c: Circle) GetX*(): INTEGER;
BEGIN RETURN c.x + c.r END GetX;
```

Method modifiers:

| Modifier      | Meaning                                                     |
|---------------|-------------------------------------------------------------|
| `NEW`         | Introduces a new method slot.                               |
| `EXTENSIBLE`  | This method may be overridden in subtypes.                  |
| `ABSTRACT`    | No body; subtypes **must** override.                        |
| (no modifier) | Override of a same-named method on the base.                |

A method that's `NEW` but not `EXTENSIBLE` is final — subtypes can't
override it.

---

## 5. Virtual dispatch through a base pointer

This is the payoff: a procedure that takes a base-class pointer can be
called with any subtype, and it dispatches to the right override.

```cp
TYPE
    ShapeDesc*  = ABSTRACT RECORD END;
    Shape*      = POINTER TO ShapeDesc;

    SquareDesc* = RECORD (ShapeDesc) side*: INTEGER END;
    Square*     = POINTER TO SquareDesc;

PROCEDURE (s: ShapeDesc) Area*(): INTEGER, NEW, ABSTRACT;

PROCEDURE (s: SquareDesc) Area*(): INTEGER;
BEGIN RETURN s.side * s.side END Area;

PROCEDURE AreaOf(s: Shape): INTEGER;
BEGIN RETURN s.Area() END AreaOf;     (* virtual dispatch *)

PROCEDURE Demo*(): INTEGER;
    VAR sq: Square; sh: Shape;
BEGIN
    NEW(sq); sq.side := 5;
    sh := sq;                          (* Square <: Shape *)
    RETURN AreaOf(sh)                  (* returns 25 *)
END Demo;
```

A `POINTER TO SquareDesc` is automatically assignable to `Shape`
(`POINTER TO ShapeDesc`) because `SquareDesc` extends `ShapeDesc`.
Inside `AreaOf`, `s.Area()` looks up the actual method through the
hidden type tag and calls `SquareDesc.Area`, not the abstract one.

---

## 6. Abstract records and abstract methods

An `ABSTRACT RECORD` cannot be instantiated with `NEW`. It exists only
to be extended. Abstract methods (declared with `, NEW, ABSTRACT`)
have no body and *must* be overridden by every concrete subclass.

```cp
TYPE
    ReaderDesc* = ABSTRACT RECORD END;
    Reader*     = POINTER TO ReaderDesc;

PROCEDURE (r: ReaderDesc) ReadByte*(): INTEGER, NEW, ABSTRACT;
PROCEDURE (r: ReaderDesc) Close*(), NEW, ABSTRACT;
```

Abstract pointers (`Reader`) are the standard CP equivalent of a Java
interface or a Go interface. The whole BlackBox `Files`, `Stores`,
`Views` and `Models` family is built this way: an abstract base
defines the contract, concrete subclasses implement it.

---

## 7. Runtime type tests: `IS` and `WITH`

When you have a base-typed pointer or value and need to know its
actual dynamic type, use the type-test operator `IS`:

```cp
IF p IS Bird THEN
    (* p is dynamically a Bird (or a subtype of Bird) *)
END
```

To safely *use* the variable as the more-specific type within a block,
use a **type guard** with `WITH`:

```cp
PROCEDURE Describe*(VAR a: Animal): INTEGER;
    VAR result: INTEGER;
BEGIN
    WITH a: Bird DO
        IF a.canFly THEN result := 10 ELSE result := 11 END
    | a: Fish DO
        result := 20 + a.fins
    ELSE
        result := a.legs
    END;
    RETURN result
END Describe;
```

Inside each `WITH` arm, `a` is treated as the narrowed type, so you
can read `a.canFly` (a `Bird` field) without a cast.

---

## 8. Putting it together: the canonical pattern

The CP idiom for any "interface plus implementations" design:

```cp
MODULE Animals;

TYPE
    (* Abstract base: defines the contract. *)
    AnimalDesc* = ABSTRACT RECORD
        legs*: INTEGER
    END;
    Animal*     = POINTER TO AnimalDesc;

PROCEDURE (a: AnimalDesc) Sound*(): INTEGER, NEW, ABSTRACT;

TYPE
    (* Concrete subclass: provides the implementation. *)
    DogDesc* = RECORD (AnimalDesc) END;
    Dog*     = POINTER TO DogDesc;

PROCEDURE (d: DogDesc) Sound*(): INTEGER;
BEGIN RETURN 1 (* "woof" *) END Sound;

(* Constructor *)
PROCEDURE NewDog*(): Dog;
    VAR d: Dog;
BEGIN NEW(d); d.legs := 4; RETURN d END NewDog;

END Animals.
```

A client module:

```cp
MODULE PetShop;
IMPORT Animals;

PROCEDURE Greet*(a: Animals.Animal): INTEGER;
BEGIN RETURN a.Sound() END Greet;        (* virtual dispatch *)

PROCEDURE Demo*(): INTEGER;
    VAR d: Animals.Dog;
BEGIN
    d := Animals.NewDog();
    RETURN Greet(d)                       (* returns 1 *)
END Demo;

END PetShop.
```

---

## 9. What to remember

| Question                                | Answer                                                                       |
|-----------------------------------------|------------------------------------------------------------------------------|
| How do I make a class?                  | A record + a pointer alias: `Foo* = POINTER TO FooDesc;`                     |
| How do I make a method?                 | A procedure with a receiver parameter, marked `NEW` if it's a new slot.      |
| How do I subclass?                      | `RECORD (BaseDesc) <new fields> END`                                         |
| How do I declare a method overridable?  | Add `, EXTENSIBLE` to its declaration.                                       |
| How do I override?                      | Declare a same-named method on the subtype, **without** `NEW`.               |
| How do I make an interface?             | Use an `ABSTRACT RECORD` with `NEW, ABSTRACT` methods.                       |
| How do I allocate?                      | `NEW(p)` — always through the pointer alias.                                 |
| How do I free?                          | You don't — the GC reclaims unreachable blocks.                              |
| How do I check the dynamic type?        | `IF p IS SubType THEN ... END`                                               |
| How do I narrow the type for a block?   | `WITH p: SubType DO ... END` — `p` is treated as `SubType` inside.           |

---

## 10. Where this is implemented

If you're curious about the runtime mechanics:

- Allocation goes through `__newcp_new_rec(typedesc)`, which writes a
  `BlockHeader` containing the type tag at `obj_ptr - 16`.
- A method call expands to:
  `obj → BlockHeader.tag → TypeDesc → vtable[slot] → indirect call`.
- See [docs/oop_runtime_status.md](oop_runtime_status.md) for the JIT
  details (how vtables are populated post-JIT) and
  [docs/garbage-collection.md](garbage-collection.md) for the GC
  contract.

For working examples, browse [Mod/Tests/](../Mod/Tests):
`PtrMethod.cp`, `AbstractDispatch.cp`, `Methods.cp`, `TypeExt.cp`,
`CaseWith.cp`.
