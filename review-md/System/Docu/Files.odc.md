**Files**

DEFINITION Files;

    CONST

        exclusive = FALSE; shared = TRUE;

        dontAsk = FALSE; ask = TRUE;

        readOnly = 0; hidden = 1; system = 2; archive = 3; stationery = 4;

    TYPE

        Name = ARRAY 256 OF CHAR;

        Type = ARRAY 16 OF CHAR;

        FileInfo = POINTER TO RECORD

            next: FileInfo;

            name: Name;

            length: INTEGER;

            type: Type

            modified: RECORD

                year, month, day, hour, minute, second: INTEGER

            END;

            attr: SET

        END;

        LocInfo = POINTER TO RECORD

            next: LocInfo;

            name: Name;

            attr: SET

        END;

        Locator = POINTER TO ABSTRACT RECORD

            res: INTEGER;

            (l: Locator) This (IN path: ARRAY OF CHAR): Locator, NEW, ABSTRACT

        END;

        File = POINTER TO ABSTRACT RECORD

            type-: Type;

            (f: File) Length (): INTEGER, NEW, ABSTRACT;

            (f: File) NewReader (old: Reader): Reader, NEW, ABSTRACT;

            (f: File) NewWriter (old: Writer): Writer, NEW, ABSTRACT;

            (f: File) Flush, NEW, ABSTRACT;

            (f: File) Register (name: Name; type: Type; ask: BOOLEAN; OUT res: INTEGER), NEW, ABSTRACT;

            (f: File) Close, NEW, ABSTRACT;

            (f: File) InitType (type: Type), NEW

        END;

        Reader = POINTER TO ABSTRACT RECORD

            eof: BOOLEAN;

            (r: Reader) Base (): File, NEW, ABSTRACT;

            (r: Reader) Pos (): INTEGER, NEW, ABSTRACT;

            (r: Reader) SetPos (pos: INTEGER), NEW, ABSTRACT;

            (r: Reader) ReadByte (OUT x: BYTE), NEW, ABSTRACT;

            (r: Reader) ReadBytes (VAR x: ARRAY OF BYTE; beg, len: INTEGER), NEW, ABSTRACT

        END;

        Writer = POINTER TO ABSTRACT RECORD

            (w: Writer) Base (): File, NEW, ABSTRACT;

            (w: Writer) Pos (): INTEGER, NEW, ABSTRACT;

            (w: Writer) SetPos (pos: INTEGER), NEW, ABSTRACT;

            (w: Writer) WriteByte (x: BYTE), NEW, ABSTRACT;

            (w: Writer) WriteBytes (IN x: ARRAY OF BYTE; beg, len: INTEGER), NEW, ABSTRACT

        END;

        Directory = POINTER TO ABSTRACT RECORD

            (d: Directory) This (IN path: ARRAY OF CHAR): Locator, NEW, ABSTRACT;

            (d: Directory) Temp (): File, NEW, ABSTRACT;

            (d: Directory) New (loc: Locator; ask: BOOLEAN): File, NEW, ABSTRACT;

            (d: Directory) Old (loc: Locator; name: Name; shared: BOOLEAN): File, NEW, ABSTRACT;

            (d: Directory) Delete (loc: Locator; name: Name), NEW, ABSTRACT;

            (d: Directory) Rename (loc: Locator; old, new: Name; ask: BOOLEAN), NEW, ABSTRACT;

            (d: Directory) SameFile (loc0: Locator; name0: Name;

                                                loc1: Locator; name1: Name): BOOLEAN, NEW, ABSTRACT;

            (d: Directory) FileList (loc: Locator): FileInfo, NEW, ABSTRACT;

            (d: Directory) LocList (loc: Locator): LocInfo, NEW, ABSTRACT;

            (d: Directory) GetFileName (name: Name; type: Type; OUT filename: Name), NEW, ABSTRACT

        END;

    VAR

        dir-, stdDir-: Directory;

        docType-, objType-, symType-: Type;

    PROCEDURE SetDir (d: Directory);

END Files.

*Most programmers never need to deal with files directly, instead they can use readers and writers (of module -> Stores) which are set up already by BlackBox!*

Module *Files* provides the abstractions necessary to handle most aspects of a hierarchical file system. A *file* is a sequence of bytes. Several access paths can be open simultaneously on the same file, possibly at different positions.

A file and its access paths are modeled as separate data structures, namely as *File* and *Reader/Writer*. Where a statement applies both to readers and writers, the term "rider" will be used. An open rider is never closed explicitly, and an application can create as many riders as it needs.

*Figure 1: File with three riders*

Each file resides at some location in the file hierarchy (i.e., in a subdirectory). In BlackBox, a location is described by a *locator* object. A *directory* object provides a procedure which creates a locator, given a path name in the host platform's file name syntax. Most other directory operations take a locator as parameter, to find the specific subdirectory where the operation should be performed.

For *temporary files*, a system-specific (implicit) location is used. Temporary files are used as scratch files, and cannot be registered in the file directory. Module *Dialog* provides further sources for file locators, via standard file dialogs.

A file itself is specified by a location and a name. The name is the file's local name at the given location, i.e., it cannot be a path name.

A directory object provides three procedures to access a file: *New*, *Temp*, and *Old*. *New* creates a new file. This file already has a particular location, but is anonymous, i.e., it has no name (yet). When the file's contents are written, the file can be registered under a given name, possibly replacing an already existing file which in turn becomes anonymous itself. File registration is an atomic action, which reduces the danger that a file is replaced by a new, but incomplete or corrupted, file.

Anonymous files for which no more riders exist are automatically deleted by the garbage collector, at an appropriate time.

*Temp* creates a temporary file. Such a file is never registered, and thus remains anonymous.

*Old* looks up and opens an existing file, given its name and location. The file may either be opened in *shared* or in *exclusive* mode. "shared" means that it may be looked up and opened by several programs simultaneously, but that none may alter it (immutable file). Even if a file has been opened, its entry in the file directory is replaced when a new file is registered at the same location and under the same name. In this case, the old file remains accessible through the existing file readers. However, looking up this file with procedure *Old* yields the most recently registered file version. When no more riders on an older file version exist, the disk space occupied by the file is reclaimed by the garbage collector eventually.

Opening a file in shared mode is the rule in BlackBox; opening a file in exclusive mode is an infrequent exception. "exclusive" means that at most one program may open a file. As long as the file is not closed again, other programs remain locked out, i.e., *Old* on the same file fails. An exclusively opened file may be modified (mutable file), which is useful for simple data base applications. Registering a new file under the same name as an exclusively opened file has the same effect as for shared files, i.e., the existing file becomes anonymous, and is garbage collected eventually.

A file can be opened in exclusive mode, closed, and then be opened again in shared mode, for example. However, it can never be open in exclusive and in shared mode simultaneously.

Open files for which no more riders exist are automatically closed by the garbage collector at an appropriate time. For files opened in exclusive mode, it is recommended that they be closed explicitly, in order to make them accessible again to other programs as early as possible.

A directory object represents *all* accessible files (not just one subdirectory), independent of their location in the file hierarchy. There is exactly one file hierarchy. However, every BlackBox service may implement its own file directory object. Such an object represents exactly the same file hierarchy, but may provide different ways to look up files, e.g., by applying default search paths, or it may define a current directory relative to which path names are evaluated, etc.

Files are typed. This means that each file has a *type* attribute which is a string, typically of length 3 (Windows) or 4 (Mac OS). On some platforms, the host file system knows about file types (Mac OS), while on others file types are simulated by using file suffixes as extensions (Windows). File types are useful to tell the system which operations are permissible on files and which aren't. For example, it is possible to install file converters (-> Converters) in BlackBox which translate between file and memory data structures.

Example: [<u>ObxAscii  docu</u>](../../Obx/Docu/Ascii.odc.md)

CONST **exclusive, shared**

Values which can be passed to the *Directory.Old.shared* parameter, to determine whether a file should be opened in shared or in exclusive mode.

CONST **ask, dontAsk**

Values which can be passed to the *Directory.New*, *Directory.Rename*, and *File.Register* methods.

CONST **readOnly**

Possible value for *FileInfo.attr*. Indicates that the file can be accessed only for reading.

CONST **hidden**

Possible value for *FileInfo.attr*. Indicates that the file is not displayed when the user lists the available file.

CONST **system**

Possible value for *FileInfo.attr*. Indicates that the file belongs to the operating system.

CONST **archive**

Possible value for *FileInfo.attr*. Indicates that the file is an archive.

CONST **stationery**

Possible value for *FileInfo.attr*. Indicates that the file is a stationery (i.e., a template).

TYPE **Name**

String type for file names.

TYPE **Type**

String type for file type names. Under Windows, file type names correspond to the three-character file name extensions, e.g., file *XYZ.txt* has type *txt*. On Mac OS, the appropriate four-character file type name is used, e.g. an ASCII file *xyz* has file type *TEXT*.

TYPE **Locator**

ABSTRACT

A file locator identifies a location in the file system.

File locators are used internally, and sometimes in commands which operate on non-BlackBox files.

File locators are extended internally.

**res**: INTEGER

Directory operations return their results in the locator's *res* field.

The following result codes are predefined:

res = 0    no error

res = 1    invalid parameter (name or locator)

res = 2    location or file not found

res = 3    file already exists

res = 4    write-protection

res = 5    io error

res = 6    access denied

res = 7    illegal file type

res = 8    cancelled

res = 80    not enough memory

res = 81    not enough system resources (disk space, file handles, etc.)

A particular BlackBox implementation may return additional, platform-specific, error codes. These error codes always have negative values.

PROCEDURE (l: Locator) **This** (IN path: ARRAY OF CHAR): Locator

NEW, ABSTRACT

*This* evaluates a relative path, starting from the location specified by *l*.

Post

result # NIL

    l.res = 0    no error

result = NIL

    l.res = 1    invalid name

    l.res = 5    io error

TYPE **FileInfo**

This record represents information about a file.

**next**: FileInfo

Next entry in the list of file descriptors. No particular ordering is defined.

**name**: Name    name # ""

The file's name.

**length**: INTEGER    length >= 0

The file's length in bytes.

**type**: Type    type # ""

The file's type.

**modified**: RECORD year, month, day, hour, minute, second: INTEGER END

Date and time of most recent modification of the file.

**attr**: SET

Indicates various optional attributes of a file (*readOnly, hidden, system, archive, stationery*).

TYPE **LocInfo**

This record represents information about a location.

**next**: LocInfo

Next entry in the list of location descriptors. No particular ordering is defined.

**name**: Name    name # ""

The file's name.

**attr**: SET

Indicates various optional attributes of a location (*readOnly, hidden, system, archive*).

TYPE **File**

ABSTRACT

A file is a carrier for a linear sequence of bytes, which typically resides on a hard disk or similar device.

Files are allocated by file directories.

Files are used by commands which operate on non-BlackBox files.

Files are extended internally.

**type**-: Type    type # ""

This file's file type.

PROCEDURE (f: File) **Length** (): INTEGER

NEW, ABSTRACT

Returns the current length of the file in bytes.

Post

result >= 0

PROCEDURE (f: File) **NewReader** (old: Reader): Reader

NEW, ABSTRACT

Returns a reader which has the appropriate type (for this file type). If *old = NIL*, then a new reader is allocated. If *old # NIL* and *old* has the appropriate type, *old* is returned. Otherwise, a new reader is allocated. The returned reader is connected to *f*, its *eof* field is set to *FALSE*, and its position is somewhere on the file. If an old reader is passed as parameter, the old position will be retained if possible.

If an old reader is passed as parameter, it is the application's responsibility to guarantee that it is not in use anymore. Passing an unused old reader is recommended because it avoids unnecessary allocations.

Post

result # NIL

~result.eof

old # NIL & old.Base() = f

    result.Pos() = old.Pos()

old = NIL OR old.Base() # f

    result.Pos() = 0

PROCEDURE (f: File) **NewWriter** (old: Writer): Writer

NEW, ABSTRACT

Returns a writer which has the appropriate type (for this file type). If *old = NIL*, then a new writer is allocated. If *old # NIL* and *old* has the appropriate type, *old* is returned. Otherwise, a new writer is allocated. The returned writer is connected to *f*, and its position is somewhere on the file. If an old writer is passed as parameter, the old position will be retained if possible.

If an old writer is passed as parameter, it is the application's responsibility to guarantee that it is not in use anymore. Passing an unused old writer is recommended because it avoids unnecessary allocations.

Read-only files allow no writers at all. In such cases, *NewWriter* returns *NIL*.

Post

result # NIL

    old # NIL & old.Base() = f

        result.Pos() = old.Pos()

    old = NIL OR old.Base() # f

        result.Pos() = f.Length()

result = NIL

    read-only file

PROCEDURE (f: File) **Flush**

NEW, ABSTRACT

To guarantee consistency of the file, *Flush* should be called after the last writer operation. Superfluous calls of *Flush* have no effect.

*Close* may call *Flush* internally.

PROCEDURE (f: File) **Register** (name: Name; type: Type; ask: BOOLEAN; OUT res: INTEGER)

NEW, ABSTRACT

*Register* makes an anonymous file permanently available. If a file with the same name at the same location already exists, it is deleted first. If the deletion does not work, i.e. when the file is write protected, the parameter *ask* determines whether a platform specific error message is displayed or not. Pass either the constant *ask* or *dontAsk*.

*Register* can be considered as an atomic action.

Only files opened with procedure *New* may be registered. Trying to register a file opened with *Old* results in a precondition violation error.

If an already existing file is deleted during *Register*, only its entry in the file directory is removed. The file's contents are still available to existing file riders. The space occupied by a file is reclaimed at an unspecified time after no more riders on it exist anymore.

The file *f* and the riders operating on file *f* are not valid anymore after registering *f*, i.e., no more file or rider operations may be performed on it. This also implies that *Register* may only be executed once. However, the registered file can be retrieved by procedure *Old* again.

*Register* may call *Flush* internally, and closes the file.

Each registered file has a file type, which is passed to *Register* in the *type* parameter.

Pre

f is anonymous and not temporary    20

name # ""    21

name is a file name    22

Post

res = 0    no error

res = 1    invalid parameter (name or locator)

res = 2    location or file not found

res = 3    file already exists

res = 4    write-protection

res = 5    io error

res = 6    access denied

res = 7    illegal file type

res = 8    cancelled

res = 80    not enough memory

res = 81    not enough system resources (disk space, file handles, etc.)

A particular BlackBox implementation may return additional, platform-specific, error codes. These error codes always have negative values.

PROCEDURE (f: File) **Close**

NEW, ABSTRACT

Closes an open file. Close does nothing if the file is not open. If a call to *New* or *Old* is not balanced by a call to *Close*, the *Close* is later performed automatically, at an unspecified time. If it is known that a file won't be used again, it is recommended to call its *Close* procedure.

The file *f* and the riders operating on file *f* are not valid anymore after closing *f*, i.e., no more file or rider operations may be performed on it. However, the closed file can be retrieved and opened again by procedure *Old*.

*Close* may call *Flush* internally.

*Close* should (but need not necessarily) be called explicitly after a file is not needed anymore.

PROCEDURE (f: File) **InitType** (type: Type)

Initializes the file's *type* field.

Pre

type # ""    20

f.type = "" OR f.type = type    21

TYPE **Reader**

ABSTRACT

Reading access path to a file carrier.

Readers are allocated by their base files.

Readers are used by commands which read non-BlackBox files and operate at the byte level.

Readers are extended internally.

**eof**: BOOLEAN

Set when it has been attempted to read the byte after the end of the file (by *ReadByte* or *ReadBytes*). Reset when the reader is generated or positioned.

PROCEDURE (r: Reader) **Base** (): File

NEW, ABSTRACT

Returns the file to which the reader is currently connected.

Post

result # NIL

PROCEDURE (r: Reader) **Pos** (): INTEGER

NEW, ABSTRACT

Returns the reader's current position.

Post

0 <= result <= r.Base().Length()

PROCEDURE (r: Reader) **SetPos** (pos: INTEGER)

NEW, ABSTRACT

Sets the reader's current position to *pos* and clears the *eof* flag.

Pre

pos >= 0    20

pos <= r.Base().Length()    21

Post

r.Pos() = pos

~r.eof

PROCEDURE (r: Reader) **ReadByte** (OUT x: BYTE)

NEW, ABSTRACT

Attempts to read the byte after the current position. If successful, it increments the position by one. If the current position (before reading) is at the end of the available data, i.e., *Pos* equals the carrier data's length, then *r.eof* is set.

*ReadByte* internally may call *SetPos*.

Post

r.Pos()' < r.Base().Length()

    r.Pos() = r.Pos()' + 1

    ~r.eof

    x = byte after r.Pos()'

r.Pos()' = r.Base().Length()

    r.Pos() = r.Base().Length()

    r.eof

    x = 0H

PROCEDURE (r: Reader) **ReadBytes** (VAR x: ARRAY OF BYTE; beg, len: INTEGER)

NEW, ABSTRACT

Attempts to read *len* bytes after the current position. It increments the position by the number of bytes which have been read successfully. If reading is continued beyond the file's length, then *r.eof* is set. The data are transferred to the array *x* starting at element *beg*.

*ReadBytes* internally may call *SetPos*.

Pre

beg >= 0    20

len >= 0    21

beg + len <= LEN(x)    22

Post

r.Pos()' <= r.Base().Length() - len

    r.Pos() = r.Pos()' + len

    ~r.eof

    len bytes read after r.Pos()' and transferred into x

r.Pos()' > r.Base().Length() - len

    r.Pos() = r.Base().Length()

    r.eof

    r.Base().Length() - r.Pos()' bytes read after r.Pos()' and transferred into x

TYPE **Writer**

ABSTRACT

Writing access path to a file carrier.

Writers are allocated by their base files.

Writers are used by commands which write non-BlackBox files and operate at the byte level.

Writers are extended internally.

PROCEDURE (w: Writer) **Base** (): File

NEW, ABSTRACT

Returns the file to which the writer is currently connected.

Post

result # NIL

PROCEDURE (w: Writer) **Pos** (): INTEGER

NEW, ABSTRACT

Returns the writer's current position.

Post

0 <= result <= w.Base().Length()

PROCEDURE (w: Writer) **SetPos** (pos: INTEGER)

NEW, ABSTRACT

Sets the writer's current position to *pos*.

Pre

pos >= 0    20

pos <= w.Base().Length()    21

Post

w.Pos() = pos

PROCEDURE (w: Writer) **WriteByte** (x: BYTE)

NEW, ABSTRACT

Writes a byte after the current position, then increments the current position. If the current position is at the end of the carrier data, the writer's length is incremented also.

*WriteByte* internally may call *SetPos*.

Post

x written at w.Pos()'

w.Pos() = w.Pos()' + 1

w.Pos()' < w.Base().Length()'

    w.Base().Length() = w.Base().Length()'

    x has overwritten old value after w.Pos()'

w.Pos()' = w.Base().Length()'

    w.Base().Length() = w.Base().Length()' + 1

    x was appended

PROCEDURE (w: Writer) **WriteBytes** (IN x: ARRAY OF BYTE; beg, len: INTEGER)

NEW, ABSTRACT

Writes *len* bytes after the current position and increments the position accordingly. If necessary the stream's length is increased. The data are transferred from array *x* starting with element *beg*.

*WriteBytes* internally may call *SetPos*.

Pre

beg >= 0    20

len >= 0    21

beg + len <= LEN(x)    22

Post

len bytes transferred from variable x to carrier

w.Pos() = w.Pos()' + len

w.Pos()' + len <= w.Base().Length()'

    w.Base().Length() = w.Base().Length()'

w.Pos()' + len > w.Base().Length()'

    w.Base().Length() = w.Pos()' + len

TYPE **Directory**

ABSTRACT

Directory for the lookup in and manipulation of file directories.

File directories are allocated by BlackBox.

File directories are used by commands which operate on non-BlackBox files.

File directories are extended internally.

PROCEDURE (d: Directory) **This** (IN path: ARRAY OF CHAR): Locator

NEW, ABSTRACT

Returns a locator, given a path name in the host platform's syntax.

*This* may perform some validity checks, e.g., whether the syntax of the name is correct. Passing the empty string yields a default location (typically the BlackBox directory itself).

Post

result # NIL

result.res = 0

    legal locator

result.res # 0

    illegal locator

PROCEDURE (d: Directory) **Temp** (): File

NEW, ABSTRACT

Returns a temporary file. This file is anonymous, i.e., not registered in a directory. (In host file systems where anonymous files are not directly supported, they may appear under temporary names in a suitable subdirectory.) Registration is not possible on a temporary file.

A temporary file always has both read and write capabilities (mutable file).

Post

result # NIL

PROCEDURE (d: Directory) **New** (loc: Locator; ask: BOOLEAN): File

NEW, ABSTRACT

Returns a new file object (or *NIL* if this is not possible). This file is anonymous, i.e., not yet registered in the directory. (In host file systems where anonymous files are not directly supported, they may appear under temporary names in subdirectory *loc.*) If the file is registered later, it will appear in the subdirectory specified by *loc*.

If *loc* indicates a location that does not yet exist, the necessary location(s) (i.e., directories) must be created first. Parameter *ask* determines whether the user is asked for the permission to do so. Pass either the constant *ask* or *dontAsk*.

A new file always has both read and write capabilities (mutable file).

If location *loc* does not exist, the user may be asked whether the location should be created (*loc.res = 0*) or not (*loc.res = 8*).

Pre

loc # NIL    20

Post

result # NIL

    loc.res = 0    no error

result = NIL

    loc.res = 1    invalid name

    loc.res = 2    location not found

    loc.res = 4    write-protection

    loc.res = 5    io error

    loc.res = 8    cancelled

PROCEDURE (d: Directory) **Old** (loc: Locator; name: Name; shared: BOOLEAN): File

NEW, ABSTRACT

Looks up and opens a file with name *name* at location *loc*. It returns this file (or *NIL* if this is not possible). Parameter *shared* determines whether the returned file is in shared or in exclusive mode. A shared file provides read-only access. This means that several applications may read the file simultaneously, but it may not be modified. An exclusively opened file provides exclusive read and write access. This means that both read and write access are denied to any other application. Note however, that the application may pass on the file pointer to wherever it likes. The point is, another application cannot gain access to the file solely via the file directory, without cooperation of the application which currently has access. Moreover, "exclusive" access does not imply that only one rider may be active on the file.

A file is usually opened in shared mode. To change its contents, a new file is generated and then registered under the old name. If only a small part of the data is actually changed, it may be more appropriate to use the exclusive mode instead, e.g. when implementing simple data bases. In this case, the file should be closed explicitly as soon as it isn't needed anymore.

Pre

loc # NIL    20

name # ""    21

Post

result # NIL

    loc.res = 0    no error

result = NIL

    loc.res = 1    invalid name

    loc.res = 2    location or file not found

    loc.res = 6    access denied

PROCEDURE (d: Directory) **Delete** (loc: Locator; name: Name)

NEW, ABSTRACT

Deletes the file specified by *loc* and *name*.

Pre

loc # NIL    20

Post

loc.res = 0    no error

loc.res = 1    invalid parameter (name or locator)

loc.res = 2    location or file not found

loc.res = 4    write-protection

loc.res = 5    io error

PROCEDURE (d: Directory) **Rename** (loc: Locator; old, new: Name; ask: BOOLEAN)

NEW, ABSTRACT

Rename the file specified by *loc* and *new* to the local name *new*. If a file with name *new* already exists, it must be deleted first. Parameter *ask* determines whether the user is asked for the permission to do so. Pass either the constant *ask* or *dontAsk*.

Pre

loc # NIL    20

Post

loc.res = 0    no error

loc.res = 1    invalid parameter (locator or name)

loc.res = 2    location or file not found

loc.res = 3    file already exists

loc.res = 4    write-protection

loc.res = 5    io error

PROCEDURE (d: Directory) **SameFile** (loc0: Locator; name0: Name;

                                                            loc1: Locator; name1: Name): BOOLEAN;

NEW, ABSTRACT

Determines whether two *(locator, name)* pairs denote the same file.

Pre

loc0 # NIL    20

name0 # ""    21

loc1 # NIL    22

name1 # ""    23

PROCEDURE (d: Directory) **FileList** (loc: Locator): FileInfo

NEW, ABSTRACT

Returns information about the files at a given location. The result is a linear list of file descriptions, in no particular order. The procedure may alter *loc.res*.

Pre

loc # NIL    20

PROCEDURE (d: Directory) **LocList** (loc: Locator): LocInfo

NEW, ABSTRACT

Returns information about subdirectories at a given location. The result is a linear list of location (subdirectory) descriptions, in no particular order. The procedure may alter *loc.res*.

Pre

loc # NIL    20

PROCEDURE (d: Directory) **GetFileName** (name: Name; type: Type; OUT filename: Name)

NEW, ABSTRACT

Make a file name out of a file and its type.

Windows: filename = name + "." + type

Mac OS: filename = name

VAR **dir-, stdDir-**: Directory    (dir # NIL) & (stdDir # NIL)

Directories for the lookup of files.

PROCEDURE **SetDir** (d: Directory)

Assigns directory.

*SetDir* is used in configuration routines.

Pre

d # NIL    20

Post

stdDir' = NIL

    stdDir = d

stdDir' # NIL

    stdDir = stdDir'

dir = d

VAR **docType**-, **objType**-, **symType**-: Type    (docType # NIL) & (objType # NIL) & (symType # NIL)

File types of BlackBox documents (*docType*), of BlackBox code files (*objType*), and of BlackBox symbol files (*symType*).

