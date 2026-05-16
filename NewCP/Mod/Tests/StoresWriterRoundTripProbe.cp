MODULE StoresWriterRoundTripProbe;
(* Smoke test for the in-memory Writer + buffer-sourced Reader.
   Writes a known sequence of primitive fields through the typed
   `Writer` methods, opens a Reader on the resulting buffer, reads
   the same sequence back through the typed `Reader` methods, and
   returns a packed value that's only correct if every field
   round-tripped intact.

   This is the foundation `Stores.CopyOf` will sit on: Externalize
   appends bytes via a Writer, then a Reader anchored at those
   bytes drains them via Internalize.  No node tree needed —
   the whole carrier is one anonymous byte buffer.
*)

    IMPORT Stores;

    PROCEDURE Run* (): INTEGER;
        VAR
            wr: Stores.Writer;
            rd: Stores.Reader;
            i, l: INTEGER;
            b: BOOLEAN;
            byte: BYTE;
    BEGIN
        wr.handle := Stores.NewWriter();
        ASSERT(wr.handle # 0, 20);

        (* Serialise: 1 byte (0x2A = 42), 2-byte int 1234 (BB i16 LE),
           4-byte long 999999999 (BB i32 LE), BOOLEAN TRUE.
           Total = 1 + 2 + 4 + 1 = 8 bytes. *)
        wr.WriteByte(2AX);
        wr.WriteInt(1234);
        wr.WriteLong(999999999);
        wr.WriteBool(TRUE);
        ASSERT(Stores.WriterPos(wr.handle) = 8, 21);

        (* Hand the buffer over to a Reader and drain it back. *)
        rd.handle := Stores.OpenReaderFromWriter(wr.handle);
        ASSERT(rd.handle # 0, 22);
        Stores.CloseWriter(wr.handle);

        rd.eof := FALSE;
        rd.ReadByte(byte);
        rd.ReadInt(i);
        rd.ReadLong(l);
        rd.ReadBool(b);

        Stores.CloseReader(rd.handle);

        (* Assemble a packed result that's only the expected value
           if every field round-tripped:
               byte (42) * 100_000_000
             + i    (1234) * 100
             + (l - 999999900) * 10    (= 99 * 10 = 990)
             + (IF b THEN 7 ELSE 0)
             = 4_200_000_000 + 123_400 + 990 + 7
             = 4_200_124_397 *)
        IF ~b THEN RETURN -1 END;
        RETURN (byte * 100000000) + (i * 100)
             + (l - 999999900) * 10 + 7
    END Run;

END StoresWriterRoundTripProbe.
