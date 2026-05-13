MODULE ArrayOfXModRecordProbe;
(* Repro for deferred_fixes #34: field access on an
   array-element of a cross-module Named record.

   `bag.items` is `ARRAY 8 OF ArrayOfXModRecordBase.Entry`,
   where `Entry` is declared in the imported module.
   The chain `bag.items[i].stop` should:

     1. gep into `bag` for field `items` (Array of Entry).
     2. IndexGep into the array at `i` (gives ref<Entry>).
     3. gep into Entry for field `stop`.

   Currently the IR skips step 2 and tries to resolve
   `stop` directly on the array type, falling into the
   `opaque:field:stop` unresolved-field stub.
*)

IMPORT ArrayOfXModRecordBase;

PROCEDURE Run* (): INTEGER;
    VAR bag: ArrayOfXModRecordBase.Bag;
BEGIN
    bag.len := 2;
    bag.items[0].stop := 11;
    bag.items[0].kind := {0, 2};
    bag.items[1].stop := 22;
    bag.items[1].kind := {1};
    RETURN bag.items[0].stop * 100 + bag.items[1].stop  (* 1122 *)
END Run;

END ArrayOfXModRecordProbe.
