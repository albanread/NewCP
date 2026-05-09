MODULE HostFiles;
(*
   Concrete Files implementation backed by HostFileSys (Rust std::fs).

   This is the analogue of BlackBox Host/Mod/Files.odc but trimmed to
   the subset needed to make Files.cp do real I/O end-to-end:

     StdLocator   <: Files.LocatorDesc    — holds an absolute path
     StdFile      <: Files.FileDesc       — wraps a HostFileSys handle
     StdReader    <: Files.ReaderDesc     — sequential read view
     StdWriter    <: Files.WriterDesc     — sequential write view
     StdDir       <: Files.DirectoryDesc  — the singleton filesystem root

   What's intentionally simplified relative to BlackBox:
   - Locator just carries a path string. No "current dir" / "stationery
     folder" semantics — pass an absolute path to `dir.This(path)`.
   - Multiple readers/writers per file are NOT shared via a single
     handle yet. Each NewReader/NewWriter opens its own handle.
   - File.Register is a no-op (BlackBox uses it to commit a temp file
     to its real name; here Open writes to the target path directly).
   - FileList / LocList / SameFile / GetFileName return NIL / FALSE
     stubs.
*)

IMPORT Files, HostFileSys;

TYPE
    StdLocatorDesc* = RECORD (Files.LocatorDesc)
        path*: Files.Name
    END;
    StdLocator* = POINTER TO StdLocatorDesc;

    StdFileDesc* = RECORD (Files.FileDesc)
        path*:   Files.Name;
        handle*: INTEGER;     (* HostFileSys handle, 0 if closed *)
        mode*:   INTEGER      (* HostFileSys.modeRead/Write/ReadWrite *)
    END;
    StdFile* = POINTER TO StdFileDesc;

    StdReaderDesc* = RECORD (Files.ReaderDesc)
        file*: StdFile;
        pos*:  INTEGER
    END;
    StdReader* = POINTER TO StdReaderDesc;

    StdWriterDesc* = RECORD (Files.WriterDesc)
        file*: StdFile;
        pos*:  INTEGER
    END;
    StdWriter* = POINTER TO StdWriterDesc;

    StdDirDesc* = RECORD (Files.DirectoryDesc)
        marker-: INTEGER     (* placeholder so the record is non-empty *)
    END;
    StdDir* = POINTER TO StdDirDesc;

VAR
    theDir-: StdDir;


(* -- Helpers ----------------------------------------------------------- *)

PROCEDURE CopyName (IN src: ARRAY OF CHAR; OUT dst: ARRAY OF CHAR);
    VAR i: INTEGER;
BEGIN
    i := 0;
    WHILE (i < LEN(src) - 1) & (i < LEN(dst) - 1) & (src[i] # 0X) DO
        dst[i] := src[i];
        INC(i)
    END;
    dst[i] := 0X
END CopyName;


(* -- StdLocator -------------------------------------------------------- *)

PROCEDURE (l: StdLocatorDesc) This* (IN path: ARRAY OF CHAR): Files.Locator;
    VAR new: StdLocator;
BEGIN
    NEW(new);
    CopyName(path, new.path);
    new.res := 0;
    RETURN new
END This;


(* -- StdFile ----------------------------------------------------------- *)

PROCEDURE (f: StdFileDesc) Length* (): INTEGER;
BEGIN
    RETURN HostFileSys.Length(f.handle)
END Length;

PROCEDURE (f: StdFileDesc) Closed* (): BOOLEAN;
BEGIN
    RETURN f.handle = 0
END Closed;

PROCEDURE (f: StdFileDesc) Shared* (): BOOLEAN;
BEGIN
    (* This minimal impl always opens exclusively. *)
    RETURN FALSE
END Shared;

PROCEDURE (f: StdFileDesc) Flush* ();
    VAR ignore: INTEGER;
BEGIN
    ignore := HostFileSys.Flush(f.handle)
END Flush;

PROCEDURE (f: StdFileDesc) Close* ();
BEGIN
    IF f.handle # 0 THEN
        HostFileSys.Close(f.handle);
        f.handle := 0
    END
END Close;

PROCEDURE (f: StdFileDesc) NewReader* (old: Files.Reader): Files.Reader;
    VAR r: StdReader;
BEGIN
    NEW(r);
    r.file := f(StdFile);
    r.pos  := 0;
    r.eof  := FALSE;
    RETURN r
END NewReader;

PROCEDURE (f: StdFileDesc) NewWriter* (old: Files.Writer): Files.Writer;
    VAR w: StdWriter;
BEGIN
    NEW(w);
    w.file := f(StdFile);
    w.pos  := HostFileSys.Length(f.handle);
    RETURN w
END NewWriter;

PROCEDURE (f: StdFileDesc) Register* (name: Files.Name; type: Files.Type;
                                      ask: BOOLEAN; OUT res: INTEGER);
BEGIN
    (* In this minimal impl, files are written directly to their final
       path at New() time, so Register is a no-op. *)
    res := 0
END Register;


(* -- StdReader --------------------------------------------------------- *)

PROCEDURE (r: StdReaderDesc) Base* (): Files.File;
BEGIN
    RETURN r.file
END Base;

PROCEDURE (r: StdReaderDesc) Pos* (): INTEGER;
BEGIN
    RETURN r.pos
END Pos;

PROCEDURE (r: StdReaderDesc) SetPos* (pos: INTEGER);
    VAR ignore: INTEGER;
BEGIN
    ignore := HostFileSys.SetPos(r.file.handle, pos);
    r.pos := pos;
    r.eof := FALSE
END SetPos;

PROCEDURE (r: StdReaderDesc) ReadByte* (OUT x: BYTE);
    VAR b, ignore: INTEGER;
BEGIN
    ignore := HostFileSys.SetPos(r.file.handle, r.pos);
    b := HostFileSys.ReadByte(r.file.handle);
    IF b < 0 THEN
        r.eof := TRUE;
        x := 0
    ELSE
        r.pos := r.pos + 1;
        (* INTEGER (i64) -> BYTE (u8): three SHORT() steps in NewCP's
           rank chain (Integer -> IntShort -> ShortInt -> Byte). *)
        x := SHORT(SHORT(SHORT(b)))
    END
END ReadByte;

PROCEDURE (r: StdReaderDesc) ReadBytes* (VAR x: ARRAY OF BYTE; beg, len: INTEGER);
    VAR i, b: INTEGER; one: BYTE;
BEGIN
    i := 0;
    WHILE (i < len) & ~r.eof DO
        r.ReadByte(one);
        IF ~r.eof THEN
            x[beg + i] := one;
            INC(i)
        END
    END
END ReadBytes;


(* -- StdWriter --------------------------------------------------------- *)

PROCEDURE (w: StdWriterDesc) Base* (): Files.File;
BEGIN
    RETURN w.file
END Base;

PROCEDURE (w: StdWriterDesc) Pos* (): INTEGER;
BEGIN
    RETURN w.pos
END Pos;

PROCEDURE (w: StdWriterDesc) SetPos* (pos: INTEGER);
    VAR ignore: INTEGER;
BEGIN
    ignore := HostFileSys.SetPos(w.file.handle, pos);
    w.pos := pos
END SetPos;

PROCEDURE (w: StdWriterDesc) WriteByte* (x: BYTE);
    VAR ignore: INTEGER;
BEGIN
    ignore := HostFileSys.SetPos(w.file.handle, w.pos);
    ignore := HostFileSys.WriteByte(w.file.handle, x);
    w.pos := w.pos + 1
END WriteByte;

PROCEDURE (w: StdWriterDesc) WriteBytes* (IN x: ARRAY OF BYTE; beg, len: INTEGER);
    VAR i: INTEGER;
BEGIN
    i := 0;
    WHILE i < len DO
        w.WriteByte(x[beg + i]);
        INC(i)
    END
END WriteBytes;


(* -- StdDir ------------------------------------------------------------ *)

PROCEDURE (d: StdDirDesc) This* (IN path: ARRAY OF CHAR): Files.Locator;
    VAR loc: StdLocator;
BEGIN
    NEW(loc);
    CopyName(path, loc.path);
    loc.res := 0;
    RETURN loc
END This;

PROCEDURE (d: StdDirDesc) New* (loc: Files.Locator; ask: BOOLEAN): Files.File;
    VAR f: StdFile; sl: StdLocator; h: INTEGER;
BEGIN
    sl := loc(StdLocator);
    h := HostFileSys.Open(sl.path, HostFileSys.modeReadWrite);
    IF h = 0 THEN RETURN NIL END;
    NEW(f);
    CopyName(sl.path, f.path);
    f.handle := h;
    f.mode   := HostFileSys.modeReadWrite;
    RETURN f
END New;

PROCEDURE (d: StdDirDesc) Old* (loc: Files.Locator; name: Files.Name;
                                shared: BOOLEAN): Files.File;
    VAR f: StdFile; sl: StdLocator; h: INTEGER;
BEGIN
    sl := loc(StdLocator);
    h := HostFileSys.Open(sl.path, HostFileSys.modeRead);
    IF h = 0 THEN RETURN NIL END;
    NEW(f);
    CopyName(sl.path, f.path);
    f.handle := h;
    f.mode   := HostFileSys.modeRead;
    RETURN f
END Old;

PROCEDURE (d: StdDirDesc) Temp* (): Files.File;
BEGIN
    (* Stub: caller should pass an absolute path through New(). *)
    RETURN NIL
END Temp;

PROCEDURE (d: StdDirDesc) Delete* (loc: Files.Locator; name: Files.Name);
    VAR sl: StdLocator; ignore: INTEGER;
BEGIN
    sl := loc(StdLocator);
    ignore := HostFileSys.Delete(sl.path)
END Delete;

PROCEDURE (d: StdDirDesc) Rename* (loc: Files.Locator;
                                   old, new: Files.Name; ask: BOOLEAN);
    VAR sl: StdLocator; ignore: INTEGER;
BEGIN
    sl := loc(StdLocator);
    ignore := HostFileSys.Rename(old, new)
END Rename;

PROCEDURE (d: StdDirDesc) SameFile* (loc0: Files.Locator; name0: Files.Name;
                                     loc1: Files.Locator; name1: Files.Name): BOOLEAN;
BEGIN
    RETURN FALSE
END SameFile;

PROCEDURE (d: StdDirDesc) FileList* (loc: Files.Locator): Files.FileInfo;
BEGIN RETURN NIL END FileList;

PROCEDURE (d: StdDirDesc) LocList* (loc: Files.Locator): Files.LocInfo;
BEGIN RETURN NIL END LocList;

PROCEDURE (d: StdDirDesc) GetFileName* (name: Files.Name; type: Files.Type;
                                        OUT filename: Files.Name);
BEGIN
    CopyName(name, filename)
END GetFileName;


(* -- Module init ------------------------------------------------------- *)

BEGIN
    NEW(theDir);
    Files.SetDir(theDir)
END HostFiles.
