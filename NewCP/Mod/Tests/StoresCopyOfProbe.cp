MODULE StoresCopyOfProbe;
(* End-to-end test for `Stores.CopyOf`: a concrete Store subclass
   that carries an INTEGER field overrides Externalize / Internalize
   so the round-trip preserves the field, then we mutate the
   original and check that the copy is untouched.  This is the
   contract every Cut/Copy/Paste / undo path will rely on once it
   sits on top of `Stores.CopyOf`. *)

    IMPORT Kernel, Stores;

    TYPE
        BoxDesc* = RECORD (Stores.StoreDesc)
            value*: INTEGER
        END;
        Box* = POINTER TO BoxDesc;

    (* Override the Stores.Store persistence hooks so the round-trip
       carries `value` through the writer/reader pair. *)

    PROCEDURE (b: Box) Externalize* (VAR wr: Stores.Writer);
    BEGIN
        wr.WriteLong(b.value)
    END Externalize;

    PROCEDURE (b: Box) Internalize* (VAR rd: Stores.Reader);
    BEGIN
        rd.ReadLong(b.value)
    END Internalize;

    (* The abstract `Domain` accessor — we don't have a concrete
       Stores.Domain yet, so just return NIL.  Sema requires this
       override because Stores.StoreDesc.Domain is ABSTRACT. *)
    PROCEDURE (b: Box) Domain* (): Stores.Domain;
    BEGIN
        RETURN NIL
    END Domain;


    (** Allocate a Box, set its value to 42, deep-clone via
        `Stores.CopyOf`, mutate the original to 999, then return
        a packed value that's only correct if the copy stayed at
        42 and is a different heap object from the original.

            (orig.value * 1000) + copy.value
                = 999 * 1000 + 42
                = 999_042

        If CopyOf were the old identity stub, the copy would
        alias the original and we'd see 999 * 1000 + 999 = 999_999. *)
    PROCEDURE Run* (): INTEGER;
        VAR orig: Box; copy: Stores.Store; copyBox: Box;
    BEGIN
        NEW(orig);
        orig.value := 42;

        copy := Stores.CopyOf(orig);
        ASSERT(copy # NIL, 20);
        copyBox := copy(Box);
        ASSERT(copyBox # orig, 21);          (* must be a different heap object *)
        ASSERT(copyBox.value = 42, 22);

        orig.value := 999;
        ASSERT(copyBox.value = 42, 23);      (* mutation didn't leak *)

        RETURN (orig.value * 1000) + copyBox.value
    END Run;

END StoresCopyOfProbe.
