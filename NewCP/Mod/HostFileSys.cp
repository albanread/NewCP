DEFINITION MODULE HostFileSys;
(**
   Flat C-ABI file I/O facade backed by Rust's std::fs (see
   src/newcp-runtime/src/host_file_sys.rs). This is the host-side
   primitive that HostFiles.cp wraps in OOP-style Files.File / Reader /
   Writer subclasses.

   Direct CP clients should normally use Files / HostFiles instead;
   HostFileSys is the low-level layer.

   Conventions:
   - Paths are CP `ARRAY OF CHAR` (UTF-32, null-terminated).
   - Handles are opaque INTEGER values; 0 means "invalid".
   - Mode flags: 0 = Read, 1 = Write (create + truncate), 2 = ReadWrite.
   - Read/Write return the actual byte count, or -1 on error.
   - Length / Pos / SetPos / Flush return -1 on error (1 on success
     for SetPos / Flush).
   - Exists / Delete / Rename return 1 for true / success, 0 otherwise.
*)

CONST
    modeRead*      = 0;
    modeWrite*     = 1;
    modeReadWrite* = 2;

PROCEDURE Open* (IN path: ARRAY OF CHAR; mode: INTEGER): INTEGER;
PROCEDURE Close* (handle: INTEGER);

PROCEDURE ReadBytes*  (handle: INTEGER; VAR buf: ARRAY OF BYTE; len: INTEGER): INTEGER;
PROCEDURE WriteBytes* (handle: INTEGER; IN buf: ARRAY OF BYTE; len: INTEGER): INTEGER;

PROCEDURE ReadByte*  (handle: INTEGER): INTEGER;
PROCEDURE WriteByte* (handle: INTEGER; byte: INTEGER): INTEGER;

PROCEDURE Length* (handle: INTEGER): INTEGER;
PROCEDURE Pos*    (handle: INTEGER): INTEGER;
PROCEDURE SetPos* (handle: INTEGER; pos: INTEGER): INTEGER;
PROCEDURE Flush*  (handle: INTEGER): INTEGER;

PROCEDURE Exists* (IN path: ARRAY OF CHAR): INTEGER;
PROCEDURE Delete* (IN path: ARRAY OF CHAR): INTEGER;
PROCEDURE Rename* (IN old, new: ARRAY OF CHAR): INTEGER;

END HostFileSys.
