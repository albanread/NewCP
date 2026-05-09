MODULE HostFilesRoundTrip;
(* End-to-end test for Files / HostFiles / HostFileSys via the OOP path:
     dir.This(absolute_path) -> Locator
     dir.New(loc, FALSE)     -> File (open for read+write)
     f.NewWriter(NIL)        -> Writer
     w.WriteByte / WriteBytes
     f.NewReader(NIL)        -> Reader
     r.SetPos(0); r.ReadByte / ReadBytes
   Each step exercises virtual dispatch through Files.* abstract pointer
   types to the concrete HostFiles.Std* subclasses.

   We deliberately drive the call chain through `HostFiles.theDir`
   rather than `Files.dir`, because `Files.dir` is initialised in the
   HostFiles module body and the optimizer does not model that
   inter-module init order — calling through Files.dir folds to
   `unreachable` since the static initial value of Files.dir is NIL. *)

IMPORT Files, HostFiles, HostFileSys;

(* Per-procedure path to keep tests independent. *)
PROCEDURE FillPath (VAR path: ARRAY OF CHAR; IN src: ARRAY OF CHAR);
    VAR i: INTEGER;
BEGIN
    i := 0;
    WHILE (i < LEN(src) - 1) & (i < LEN(path) - 1) & (src[i] # 0X) DO
        path[i] := src[i];
        INC(i)
    END;
    path[i] := 0X
END FillPath;


PROCEDURE DiagThis* (): INTEGER;
    VAR loc: Files.Locator; path: ARRAY 64 OF CHAR;
BEGIN
    FillPath(path, "diag.bin");
    loc := HostFiles.theDir.This(path);
    IF loc = NIL THEN RETURN -1 END;
    RETURN 1
END DiagThis;

PROCEDURE DiagOpen* (): INTEGER;
    VAR
        loc: Files.Locator;
        sl:  HostFiles.StdLocator;
        h:   INTEGER;
        path: ARRAY 64 OF CHAR;
BEGIN
    FillPath(path, "diag2.bin");
    loc := HostFiles.theDir.This(path);
    IF loc = NIL THEN RETURN -1 END;
    sl := loc(HostFiles.StdLocator);
    IF sl = NIL THEN RETURN -2 END;
    h := HostFileSys.Open(sl.path, HostFileSys.modeReadWrite);
    IF h = 0 THEN RETURN -3 END;
    HostFileSys.Close(h);
    RETURN 1
END DiagOpen;

(* DiagOpen variant that uses the original path buffer directly,
   avoiding both the type-guard AND the StdLocator.path read. *)
PROCEDURE DiagOpenDirect* (): INTEGER;
    VAR
        loc: Files.Locator;
        h:   INTEGER;
        path: ARRAY 64 OF CHAR;
BEGIN
    FillPath(path, "diag2.bin");
    loc := HostFiles.theDir.This(path);
    IF loc = NIL THEN RETURN -1 END;
    h := HostFileSys.Open(path, HostFileSys.modeReadWrite);
    IF h = 0 THEN RETURN -3 END;
    HostFileSys.Close(h);
    RETURN 1
END DiagOpenDirect;

(* Bypass the OOP layer entirely; just call the flat HostFileSys API. *)
PROCEDURE DiagFlatOpen* (): INTEGER;
    VAR h: INTEGER; path: ARRAY 64 OF CHAR;
BEGIN
    FillPath(path, "diag3.bin");
    h := HostFileSys.Open(path, HostFileSys.modeReadWrite);
    IF h = 0 THEN RETURN -1 END;
    HostFileSys.Close(h);
    RETURN 1
END DiagFlatOpen;

PROCEDURE WriteThenReadByte* (): INTEGER;
    VAR
        loc:   Files.Locator;
        f:     Files.File;
        r:     Files.Reader;
        w:     Files.Writer;
        b:     BYTE;
        path:  ARRAY 64 OF CHAR;
BEGIN
    FillPath(path, "newcp_hostfiles_byte.bin");

    loc := HostFiles.theDir.This(path);
    IF loc = NIL THEN RETURN -1 END;

    f := HostFiles.theDir.New(loc, FALSE);
    IF f = NIL THEN RETURN -2 END;

    w := f.NewWriter(NIL);
    IF w = NIL THEN RETURN -3 END;
    w.WriteByte(0AAX);

    r := f.NewReader(NIL);
    IF r = NIL THEN RETURN -4 END;
    r.SetPos(0);
    r.ReadByte(b);
    IF r.eof THEN RETURN -5 END;

    f.Close();
    HostFiles.theDir.Delete(loc, path);
    RETURN b              (* expect 170 = 0xAA *)
END WriteThenReadByte;


PROCEDURE WriteThenReadBytes* (): INTEGER;
    VAR
        loc:   Files.Locator;
        f:     Files.File;
        r:     Files.Reader;
        w:     Files.Writer;
        path:  ARRAY 64 OF CHAR;
        i, sum: INTEGER;
        out:   ARRAY 8 OF BYTE;
        in:    ARRAY 8 OF BYTE;
BEGIN
    FillPath(path, "newcp_hostfiles_bytes.bin");

    out[0] := 1; out[1] := 2; out[2] := 3; out[3] := 4;
    out[4] := 5; out[5] := 6; out[6] := 7; out[7] := 8;

    loc := HostFiles.theDir.This(path);
    f := HostFiles.theDir.New(loc, FALSE);
    IF f = NIL THEN RETURN -1 END;

    w := f.NewWriter(NIL);
    w.WriteBytes(out, 0, 8);

    r := f.NewReader(NIL);
    r.SetPos(0);
    r.ReadBytes(in, 0, 8);

    f.Close();
    HostFiles.theDir.Delete(loc, path);

    sum := 0;
    i := 0;
    WHILE i < 8 DO
        IF in[i] # out[i] THEN RETURN -100 - i END;
        sum := sum + in[i];
        INC(i)
    END;
    RETURN sum     (* expect 1+2+...+8 = 36 *)
END WriteThenReadBytes;


PROCEDURE LengthAfterWrite* (): INTEGER;
    VAR
        loc:   Files.Locator;
        f:     Files.File;
        w:     Files.Writer;
        path:  ARRAY 64 OF CHAR;
        n:     INTEGER;
BEGIN
    FillPath(path, "newcp_hostfiles_len.bin");

    loc := HostFiles.theDir.This(path);
    f := HostFiles.theDir.New(loc, FALSE);
    IF f = NIL THEN RETURN -1 END;

    w := f.NewWriter(NIL);
    w.WriteByte(1X);
    w.WriteByte(2X);
    w.WriteByte(3X);

    n := f.Length();

    f.Close();
    HostFiles.theDir.Delete(loc, path);
    RETURN n     (* expect 3 *)
END LengthAfterWrite;

END HostFilesRoundTrip.
