MODULE Files;
(*
   Faithful port of BlackBox System/Mod/Files.odc.

   This is an *abstract* interface module — it declares the OOP surface
   (Locator, File, Reader, Writer, Directory) and a few helpers, but
   contains no I/O implementation. The actual filesystem code lives in
   HostFiles.cp (which extends each abstract type with a concrete
   subclass backed by HostFileSys).

   Differences from BlackBox:
   - The original imports Kernel for `objType`, `symType`, `docType`
     (the .ocf / .osf / .odc filename suffixes used by the loader and
     the IDE). Until the NewCP Kernel surface reaches that point the
     module body initialises them to literal strings; consumers that
     compare against these constants will still see the right values.
   - `dontAsk` and `ask` are commented in BlackBox as the boolean values
     consumed by `File.Register`, `Directory.New`, and `Directory.Rename`.
   - `attr: SET` on `FileInfo` / `LocInfo` is omitted in this slice
     (NewCP's SET runtime support is not yet wired up). The fields
     are scheduled to come back when SET lands.
*)

TYPE
    Name* = ARRAY 256 OF CHAR;
    Type* = ARRAY 16 OF CHAR;

    FileInfo* = POINTER TO RECORD
        next*: FileInfo;
        name*: Name;
        length*: INTEGER;
        type*: Type;
        modified*: RECORD
            year*, month*, day*, hour*, minute*, second*: INTEGER
        END
        (* attr*: SET — restored when SET runtime support lands *)
    END;

    LocInfo* = POINTER TO RECORD
        next*: LocInfo;
        name*: Name
        (* attr*: SET — restored when SET runtime support lands *)
    END;

    LocatorDesc* = ABSTRACT RECORD
        res*: INTEGER
    END;
    Locator* = POINTER TO LocatorDesc;

    FileDesc* = ABSTRACT RECORD
        type-: Type;
        init-: BOOLEAN
    END;
    File* = POINTER TO FileDesc;

    ReaderDesc* = ABSTRACT RECORD
        eof*: BOOLEAN
    END;
    Reader* = POINTER TO ReaderDesc;

    WriterDesc* = ABSTRACT RECORD END;
    Writer* = POINTER TO WriterDesc;

    DirectoryDesc* = ABSTRACT RECORD END;
    Directory* = POINTER TO DirectoryDesc;

CONST
    (* mode flags consumed by File.Register, Directory.New, Directory.Rename *)
    shared*    = TRUE;   exclusive* = FALSE;
    dontAsk*   = FALSE;  ask*       = TRUE;

    (* file attribute bit positions (matches BlackBox set bit numbers) *)
    readOnly*   = 0;
    hidden*     = 1;
    system*     = 2;
    archive*    = 3;
    stationery* = 4;

VAR
    dir-,    stdDir-:                Directory;   (* the active directory *)
    objType-, symType-, docType-:    Type;        (* well-known file types *)


(* -- Abstract methods --------------------------------------------------- *)

PROCEDURE (l: LocatorDesc) This* (IN path: ARRAY OF CHAR): Locator, NEW, ABSTRACT;

PROCEDURE (f: FileDesc) InitType* (type: Type), NEW;
BEGIN
    ASSERT(~f.init, 20);
    f.type := type;
    f.init := TRUE
END InitType;

PROCEDURE (f: FileDesc) Length*    (): INTEGER, NEW, ABSTRACT;
PROCEDURE (f: FileDesc) NewReader* (old: Reader): Reader, NEW, ABSTRACT;
PROCEDURE (f: FileDesc) NewWriter* (old: Writer): Writer, NEW, ABSTRACT;
PROCEDURE (f: FileDesc) Flush*     (), NEW, ABSTRACT;
PROCEDURE (f: FileDesc) Register*  (name: Name; type: Type; ask: BOOLEAN;
                                    OUT res: INTEGER), NEW, ABSTRACT;
PROCEDURE (f: FileDesc) Close*     (), NEW, ABSTRACT;
PROCEDURE (f: FileDesc) Closed*    (): BOOLEAN, NEW, ABSTRACT;
PROCEDURE (f: FileDesc) Shared*    (): BOOLEAN, NEW, ABSTRACT;

PROCEDURE (r: ReaderDesc) Base*      (): File, NEW, ABSTRACT;
PROCEDURE (r: ReaderDesc) Pos*       (): INTEGER, NEW, ABSTRACT;
PROCEDURE (r: ReaderDesc) SetPos*    (pos: INTEGER), NEW, ABSTRACT;
PROCEDURE (r: ReaderDesc) ReadByte*  (OUT x: BYTE), NEW, ABSTRACT;
PROCEDURE (r: ReaderDesc) ReadBytes* (VAR x: ARRAY OF BYTE; beg, len: INTEGER), NEW, ABSTRACT;

PROCEDURE (w: WriterDesc) Base*       (): File, NEW, ABSTRACT;
PROCEDURE (w: WriterDesc) Pos*        (): INTEGER, NEW, ABSTRACT;
PROCEDURE (w: WriterDesc) SetPos*     (pos: INTEGER), NEW, ABSTRACT;
PROCEDURE (w: WriterDesc) WriteByte*  (x: BYTE), NEW, ABSTRACT;
PROCEDURE (w: WriterDesc) WriteBytes* (IN x: ARRAY OF BYTE; beg, len: INTEGER), NEW, ABSTRACT;

PROCEDURE (d: DirectoryDesc) This*    (IN path: ARRAY OF CHAR): Locator, NEW, ABSTRACT;
PROCEDURE (d: DirectoryDesc) New*     (loc: Locator; ask: BOOLEAN): File, NEW, ABSTRACT;
PROCEDURE (d: DirectoryDesc) Old*     (loc: Locator; name: Name; shared: BOOLEAN): File,
                                       NEW, ABSTRACT;
PROCEDURE (d: DirectoryDesc) Temp*    (): File, NEW, ABSTRACT;
PROCEDURE (d: DirectoryDesc) Delete*  (loc: Locator; name: Name), NEW, ABSTRACT;
PROCEDURE (d: DirectoryDesc) Rename*  (loc: Locator; old, new: Name; ask: BOOLEAN),
                                       NEW, ABSTRACT;
PROCEDURE (d: DirectoryDesc) SameFile* (loc0: Locator; name0: Name;
                                        loc1: Locator; name1: Name): BOOLEAN, NEW, ABSTRACT;
PROCEDURE (d: DirectoryDesc) FileList* (loc: Locator): FileInfo, NEW, ABSTRACT;
PROCEDURE (d: DirectoryDesc) LocList*  (loc: Locator): LocInfo, NEW, ABSTRACT;
PROCEDURE (d: DirectoryDesc) GetFileName* (name: Name; type: Type;
                                           OUT filename: Name), NEW, ABSTRACT;


PROCEDURE SetDir* (d: Directory);
BEGIN
    ASSERT(d # NIL, 20);
    dir := d;
    IF stdDir = NIL THEN stdDir := d END
END SetDir;


BEGIN
    (* Suffix conventions inherited from BlackBox / Component Pascal:
         .ocf = compiled object file
         .osf = symbol file
         .odc = document container *)
    objType := "ocf";
    symType := "osf";
    docType := "odc"
END Files.
