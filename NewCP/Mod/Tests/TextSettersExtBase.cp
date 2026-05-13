MODULE TextSettersExtBase;
(*
   Workout for the `TextSetters` slice.

   Exercises:
   - extending ABSTRACT `Directory` and overriding `New` to
     hand out a leaf `Setter`;
   - extending ABSTRACT `Setter` with the (large) set of
     ABSTRACT method overrides;
   - extending ABSTRACT `Reader` with the four ABSTRACT
     method overrides;
   - building a LineBox, reading back its fields including
     the inline `tabW: ARRAY TextRulers.maxTabs OF INTEGER`
     array — verifies the inline-array-of-INTEGER field
     access path through a record that lives in a cross-
     module chain (TextSetters → Stores).

   Heavy logic is deferred; the test fixture stubs every
   override and just confirms the type chain stitches
   together at runtime.
*)

    IMPORT Stores, Views, Properties, TextModels, TextRulers, TextSetters;


    TYPE
        MyReaderDesc* = RECORD (TextSetters.ReaderDesc) END;
        MyReader* = POINTER TO MyReaderDesc;

        MySetterDesc* = RECORD (TextSetters.SetterDesc) END;
        MySetter* = POINTER TO MySetterDesc;

        MyDirectoryDesc* = RECORD (TextSetters.DirectoryDesc) END;
        MyDirectory* = POINTER TO MyDirectoryDesc;


    (* Reader ABSTRACT stubs. *)
    PROCEDURE (rd: MyReaderDesc) Set*
        (old: TextModels.Reader;
         setter: TextSetters.Setter;
         pos: INTEGER;
         ruler: TextRulers.Ruler;
         rpos: INTEGER);
    BEGIN
    END Set;

    PROCEDURE (rd: MyReaderDesc) Read*;
    BEGIN
    END Read;

    PROCEDURE (rd: MyReaderDesc) AdjustWidth*
        (start, pos: INTEGER; IN box: TextSetters.LineBox; VAR w: INTEGER);
    BEGIN
    END AdjustWidth;

    PROCEDURE (rd: MyReaderDesc) SplitWidth* (w: INTEGER): INTEGER;
    BEGIN
        RETURN w
    END SplitWidth;


    (* Setter ABSTRACT stubs. *)
    PROCEDURE (s: MySetterDesc) ThisPage* (pageH: INTEGER; pageNo: INTEGER): INTEGER;
    BEGIN RETURN 0 END ThisPage;

    PROCEDURE (s: MySetterDesc) NextPage* (pageH: INTEGER; start: INTEGER): INTEGER;
    BEGIN RETURN 0 END NextPage;

    PROCEDURE (s: MySetterDesc) ThisSequence* (pos: INTEGER): INTEGER;
    BEGIN RETURN pos END ThisSequence;

    PROCEDURE (s: MySetterDesc) NextSequence* (start: INTEGER): INTEGER;
    BEGIN RETURN start + 1 END NextSequence;

    PROCEDURE (s: MySetterDesc) PreviousSequence* (start: INTEGER): INTEGER;
    BEGIN RETURN start - 1 END PreviousSequence;

    PROCEDURE (s: MySetterDesc) ThisLine* (pos: INTEGER): INTEGER;
    BEGIN RETURN pos END ThisLine;

    PROCEDURE (s: MySetterDesc) NextLine* (start: INTEGER): INTEGER;
    BEGIN RETURN start + 1 END NextLine;

    PROCEDURE (s: MySetterDesc) PreviousLine* (start: INTEGER): INTEGER;
    BEGIN RETURN start - 1 END PreviousLine;

    PROCEDURE (s: MySetterDesc) GetWord* (pos: INTEGER; OUT beg, end: INTEGER);
    BEGIN beg := pos; end := pos END GetWord;

    PROCEDURE (s: MySetterDesc) GetLine* (start: INTEGER; OUT box: TextSetters.LineBox);
    BEGIN
        box.len := 0
    END GetLine;

    PROCEDURE (s: MySetterDesc) GetBox*
        (start, end, maxW, maxH: INTEGER; OUT box: TextSetters.LineBox);
    BEGIN
        box.len := 0
    END GetBox;

    PROCEDURE (s: MySetterDesc) NewReader* (old: TextSetters.Reader): TextSetters.Reader;
        VAR rd: MyReader;
    BEGIN
        NEW(rd);
        RETURN rd
    END NewReader;

    PROCEDURE (s: MySetterDesc) GridOffset*
        (dsc: INTEGER; IN box: TextSetters.LineBox): INTEGER;
    BEGIN RETURN dsc END GridOffset;


    (* Directory's NEW factory. *)
    PROCEDURE (d: MyDirectoryDesc) New* (): TextSetters.Setter;
        VAR s: MySetter;
    BEGIN
        NEW(s);
        RETURN s
    END New;


    PROCEDURE Run* (): INTEGER;
        VAR d: MyDirectory; s: TextSetters.Setter;
            box: TextSetters.LineBox; r: TextSetters.Reader;
            seqStart, prevSeq, packed: INTEGER;
    BEGIN
        NEW(d);

        (* Stage 1: Directory.New cross-module-dispatches to
           the MyDirectoryDesc.New override. *)
        s := d.New();
        IF s = NIL THEN RETURN -1 END;

        (* Stage 2: Setter.NewReader factory dispatches
           through MySetter via vtable. *)
        r := s.NewReader(NIL);
        IF r = NIL THEN RETURN -2 END;

        (* Stage 3: drive a couple of arithmetic-based
           ABSTRACT overrides to confirm vtable dispatch
           lands in our concrete leaves. *)
        seqStart := s.NextSequence(10);
        prevSeq  := s.PreviousSequence(10);
        IF seqStart # 11 THEN RETURN -3 END;
        IF prevSeq # 9 THEN RETURN -4 END;

        (* Stage 4: directly populate a LineBox and
           round-trip its inline-array-of-INTEGER tabW
           field — tests the cross-module field +
           array-index access pattern that #34 fixed. *)
        box.len     := 42;
        box.left    := 100;
        box.right   := 500;
        box.asc     := 12;
        box.dsc     := 4;
        box.tabW[0] := -3;
        box.tabW[1] := -7;
        box.tabW[5] := -11;
        IF box.tabW[0] # -3 THEN RETURN -10 END;
        IF box.tabW[1] # -7 THEN RETURN -11 END;
        IF box.tabW[5] # -11 THEN RETURN -12 END;

        (* Pack:
             seqStart (11) * 10000 + prevSeq (9) * 1000 +
             box.len (42) * 10 + box.right/100 (5)
             = 110000 + 9000 + 420 + 5 = 119425 *)
        packed := seqStart * 10000 + prevSeq * 1000
                + box.len * 10 + box.right DIV 100;
        RETURN packed     (* expected 119425 *)
    END Run;

END TextSettersExtBase.
