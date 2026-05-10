MODULE ModelsCopyOfProbe;
(* Verify `Models.CopyOf` is no longer the identity stub: a
   concrete Model subclass with one INTEGER field round-trips
   via `Stores.CopyOf` (the real BlackBox path) and the result
   is a separate heap object that doesn't alias the source. *)

    IMPORT Models, Stores;

    TYPE
        TaggedModelDesc* = RECORD (Models.ModelDesc)
            tag*: INTEGER
        END;
        TaggedModel* = POINTER TO TaggedModelDesc;

    PROCEDURE (m: TaggedModel) Externalize* (VAR wr: Stores.Writer);
    BEGIN
        m.Externalize^(wr);
        wr.WriteLong(m.tag)
    END Externalize;

    PROCEDURE (m: TaggedModel) Internalize* (VAR rd: Stores.Reader);
    BEGIN
        m.Internalize^(rd);
        rd.ReadLong(m.tag)
    END Internalize;

    (** Run constructs a TaggedModel with tag=7, deep-clones via
        Models.CopyOf, mutates the source to 99, and returns
        (orig.tag * 100) + copy.tag = 9907 only if the copy is a
        true clone.  An aliased copy would yield 99 * 100 + 99 = 9999. *)
    PROCEDURE Run* (): INTEGER;
        VAR orig, copy: TaggedModel;
            cloned: Models.Model;
    BEGIN
        NEW(orig);
        orig.tag := 7;

        cloned := Models.CopyOf(orig);
        copy := cloned(TaggedModel);
        ASSERT(copy # orig, 20);
        ASSERT(copy.tag = 7, 21);

        orig.tag := 99;
        ASSERT(copy.tag = 7, 22);

        RETURN (orig.tag * 100) + copy.tag
    END Run;

END ModelsCopyOfProbe.
