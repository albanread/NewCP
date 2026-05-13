MODULE TextMappersExtBase;
(*
   Workout for the `TextMappers` slice.

   The slice itself depends on the BB-faithful
   `TextModels.Reader` / `TextModels.Writer` / `TextModels.Model`
   abstract surface.  No concrete implementation of those
   exists in this slice (concrete StdModel reading lives in
   the wire-format `StdModelDesc` ladder that's headed for a
   future overhaul).

   So this test fixture is necessarily minimal: it builds a
   tiny concrete Reader/Writer/Model trio with hard-coded
   behaviour, runs Scanner and Formatter against them, and
   verifies that the `ConnectTo` / `SetPos` / `SetOpts` /
   `Pos` paths thread the cursor through correctly.

   What we DON'T test here — Scanner.Skip's whitespace loop
   and the at-EOT length computation — because those touch
   the read-loop body which calls `rider.ReadChar()`.  Our
   concrete leaf reader supplies just enough for ConnectTo /
   SetPos / Pos to round-trip; the full scan logic waits on
   a real text-buffer-backed reader.

   Returns a packed int proving each stage round-trips.
*)

    IMPORT TextModels, TextMappers, Containers, Models, Stores, Views;


    TYPE
        (** Minimum-viable concrete model — knows how many
            chars it has, hands out our fake Reader/Writer. *)
        MyModelDesc* = RECORD (TextModels.ModelDesc)
            simulatedLength*: INTEGER
        END;
        MyModel* = POINTER TO MyModelDesc;

        (** Concrete Reader — tracks a position, never EOT. *)
        MyReaderDesc* = RECORD (TextModels.ReaderDesc)
            myModel*: MyModel;
            pos*:     INTEGER
        END;
        MyReader* = POINTER TO MyReaderDesc;

        (** Concrete Writer — tracks an append cursor. *)
        MyWriterDesc* = RECORD (TextModels.WriterDesc)
            myModel*:  MyModel;
            writePos*: INTEGER
        END;
        MyWriter* = POINTER TO MyWriterDesc;


    (* -- Reader concrete overrides ------------------------------------- *)

    PROCEDURE (rd: MyReaderDesc) ReadChar* ();
    BEGIN
        rd.pos := rd.pos + 1
    END ReadChar;

    PROCEDURE (rd: MyReaderDesc) SetPos* (pos: INTEGER);
    BEGIN
        rd.pos := pos;
        rd.eot := FALSE;
        rd.char := "A"
    END SetPos;

    PROCEDURE (rd: MyReaderDesc) Pos* (): INTEGER;
    BEGIN
        RETURN rd.pos
    END Pos;

    PROCEDURE (rd: MyReaderDesc) Base* (): TextModels.Model;
    BEGIN
        RETURN rd.myModel
    END Base;


    (* -- Writer concrete overrides ------------------------------------- *)

    PROCEDURE (wr: MyWriterDesc) WriteChar* (ch: CHAR);
    BEGIN
        wr.writePos := wr.writePos + 1
    END WriteChar;

    PROCEDURE (wr: MyWriterDesc) WriteString* (IN s: ARRAY OF CHAR);
        VAR i: INTEGER;
    BEGIN
        i := 0;
        WHILE (i < LEN(s)) & (s[i] # 0X) DO
            wr.writePos := wr.writePos + 1;
            INC(i)
        END
    END WriteString;

    PROCEDURE (wr: MyWriterDesc) SetPos* (pos: INTEGER);
    BEGIN
        wr.writePos := pos
    END SetPos;

    PROCEDURE (wr: MyWriterDesc) Pos* (): INTEGER;
    BEGIN
        RETURN wr.writePos
    END Pos;

    PROCEDURE (wr: MyWriterDesc) SetAttr* (attr: TextModels.Attributes);
    BEGIN
    END SetAttr;

    PROCEDURE (wr: MyWriterDesc) Base* (): TextModels.Model;
    BEGIN
        RETURN wr.myModel
    END Base;


    (* -- Model concrete overrides -------------------------------------- *)

    PROCEDURE (m: MyModelDesc) NewReader* (old: TextModels.Reader): TextModels.Reader;
        VAR r: MyReader;
    BEGIN
        NEW(r);
        r.myModel := m(MyModel);    (* needs WITH-style narrow; this might trip on
                                       self-receiver assignment - test will tell *)
        r.pos := 0;
        r.eot := FALSE;
        r.char := "A";
        RETURN r
    END NewReader;

    PROCEDURE (m: MyModelDesc) NewWriter* (old: TextModels.Writer): TextModels.Writer;
        VAR w: MyWriter;
    BEGIN
        NEW(w);
        w.myModel := m(MyModel);
        w.writePos := 0;
        RETURN w
    END NewWriter;

    PROCEDURE (m: MyModelDesc) Length* (): INTEGER;
    BEGIN
        RETURN m.simulatedLength
    END Length;

    (* Containers.Model ABSTRACTs we must override. *)
    PROCEDURE (m: MyModelDesc) GetEmbeddingLimits* (OUT minW, maxW, minH, maxH: INTEGER);
    BEGIN
        minW := 0; maxW := 1000; minH := 0; maxH := 1000
    END GetEmbeddingLimits;

    PROCEDURE (m: MyModelDesc) ReplaceView* (old, new: Views.View);
    BEGIN
    END ReplaceView;


    (* -- Driver --------------------------------------------------------- *)

    PROCEDURE Run* (): INTEGER;
        VAR m: MyModel; s: TextMappers.Scanner; f: TextMappers.Formatter;
            posAfterSeek, posAfterWrite: INTEGER;
            packed: INTEGER;
    BEGIN
        NEW(m);
        m.simulatedLength := 42;

        (* Stage 1: Scanner.ConnectTo binds the rider via
           cross-module vtable dispatch into our concrete
           NewReader. *)
        s.ConnectTo(m);
        IF s.rider = NIL THEN RETURN -1 END;

        (* Stage 2: SetPos pokes the abstract Reader.SetPos
           which dispatches into our concrete one. *)
        s.SetPos(17);
        posAfterSeek := s.Pos();
        IF posAfterSeek # 17 THEN RETURN -100 - posAfterSeek END;

        (* Stage 3: Formatter binds + writes through cross-
           module vtable. *)
        f.ConnectTo(m);
        IF f.rider = NIL THEN RETURN -2 END;
        f.SetPos(0);
        f.WriteChar("X");
        f.WriteChar("Y");
        f.WriteChar("Z");
        posAfterWrite := f.Pos();
        IF posAfterWrite # 3 THEN RETURN -200 - posAfterWrite END;

        (* Pack:
             posAfterSeek (17) * 1000 + posAfterWrite (3) * 100
             + m.simulatedLength (42)
             = 17000 + 300 + 42
             = 17342 *)
        packed := posAfterSeek * 1000 + posAfterWrite * 100 + m.simulatedLength;
        RETURN packed       (* expected 17342 *)
    END Run;

END TextMappersExtBase.
