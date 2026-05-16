MODULE Fonts;

(* BlackBox-faithful Fonts module.

   Defines abstract Font and Directory types plus a small set of
   constants every text-aware module shares. The concrete
   implementation lives in HostFonts (which extends these types and
   talks to the host through HostFontsSys → iGui).

   This file is intentionally a near-verbatim port of System/Mod/Fonts
   from the BlackBox corpus: zero imports, same constants, same
   exports — recognizable to anyone who knows BlackBox. NewCP requires
   the `XxxDesc` + `Xxx = POINTER TO XxxDesc` split and method
   receivers on the record type, so the surface differs from BlackBox
   only in that mechanical respect.

   The layering is:

     Fonts.cp           (this file — abstract types, BlackBox-verbatim)
       ▲
       │  (HostFonts privately extends Fonts.FontDesc / DirectoryDesc)
     HostFonts.cp       (concrete, calls HostFontsSys for measurement)
       │
       ▼
     HostFontsSys.cp    (CP-shaped wrapper over iGui font shims)
       │
       ▼
     iGui (native font_metrics service) *)

CONST
  (* universal units: 1mm = 36000 sub-mm units, 1 point = 1/72 inch *)
  mm*    = 36000;
  point* = 12700;

  (* style elements — bits in Font.style: SET *)
  italic*    = 0;
  underline* = 1;
  strikeout* = 2;

  (* canonical weight values *)
  normal* = 400;
  bold*   = 700;

  (* sentinel typeface that means "the default for this directory" *)
  default* = "*";

TYPE
  Typeface* = ARRAY 64 OF CHAR;

  FontDesc* = ABSTRACT RECORD
    typeface-: Typeface;
    size-:     INTEGER;
    style-:    SET;
    weight-:   INTEGER
  END;
  Font* = POINTER TO FontDesc;

  TypefaceInfo* = POINTER TO RECORD
    next*:     TypefaceInfo;
    typeface*: Typeface
  END;

  DirectoryDesc* = ABSTRACT RECORD END;
  Directory*     = POINTER TO DirectoryDesc;

  StdFontDesc* = RECORD (FontDesc) END;
  StdFont*     = POINTER TO StdFontDesc;

  StdDirectoryDesc* = RECORD (DirectoryDesc) END;
  StdDirectory*     = POINTER TO StdDirectoryDesc;

VAR
  (* Read-only public. HostFonts.Init registers the concrete
     directory by calling SetDir below. *)
  dir-:    Directory;
  stdDir-: StdDirectory;

(* Set the immutable fields on a Font. Called once during construction
   from a Directory.This / Default factory. *)
PROCEDURE (f: FontDesc) Init*
  (typeface: Typeface; size: INTEGER;
   style: SET; weight: INTEGER), NEW;
BEGIN
  ASSERT(f.size = 0, 20);
  ASSERT(size # 0, 21);
  f.typeface := typeface;
  f.size := size;
  f.style := style;
  f.weight := weight
END Init;

PROCEDURE (f: FontDesc) GetBounds*
  (OUT asc, dsc, w: INTEGER), NEW, ABSTRACT;

PROCEDURE (f: FontDesc) StringWidth*
  (IN s: ARRAY OF CHAR): INTEGER, NEW, ABSTRACT;

PROCEDURE (f: FontDesc) SStringWidth*
  (IN s: ARRAY OF SHORTCHAR): INTEGER, NEW, ABSTRACT;

PROCEDURE (f: FontDesc) IsAlien* (): BOOLEAN, NEW, ABSTRACT;

PROCEDURE (d: DirectoryDesc) This*
  (typeface: Typeface; size: INTEGER;
   style: SET; weight: INTEGER): Font, NEW, ABSTRACT;

PROCEDURE (d: DirectoryDesc) Default* (): Font, NEW, ABSTRACT;

PROCEDURE (d: DirectoryDesc) TypefaceList* (): TypefaceInfo, NEW, ABSTRACT;

(* --- StdFont concrete implementation --- *)

PROCEDURE (f: StdFontDesc) GetBounds*
  (OUT asc, dsc, w: INTEGER);
BEGIN
  (* Stub: report a plausible cap-height / descender for 12 pt.
     Real metrics come from HostFonts once it registers its directory. *)
  asc := f.size * 4 DIV 5;
  dsc := f.size     DIV 5;
  w   := f.size * 3 DIV 5
END GetBounds;

PROCEDURE (f: StdFontDesc) StringWidth*
  (IN s: ARRAY OF CHAR): INTEGER;
  VAR i, w: INTEGER;
BEGIN
  w := 0; i := 0;
  WHILE (i < LEN(s)) & (s[i] # 0X) DO
    INC(w, f.size * 3 DIV 5);
    INC(i)
  END;
  RETURN w
END StringWidth;

PROCEDURE (f: StdFontDesc) SStringWidth*
  (IN s: ARRAY OF SHORTCHAR): INTEGER;
  VAR i, w: INTEGER;
BEGIN
  w := 0; i := 0;
  WHILE (i < LEN(s)) & (s[i] # 0SX) DO
    INC(w, f.size * 3 DIV 5);
    INC(i)
  END;
  RETURN w
END SStringWidth;

PROCEDURE (f: StdFontDesc) IsAlien* (): BOOLEAN;
BEGIN
  RETURN FALSE
END IsAlien;

(* --- StdDirectory concrete implementation --- *)

PROCEDURE (d: StdDirectoryDesc) This*
  (typeface: Typeface; size: INTEGER;
   style: SET; weight: INTEGER): Font;
  VAR f: StdFont;
BEGIN
  NEW(f);
  f.Init(typeface, size, style, weight);
  RETURN f
END This;

PROCEDURE (d: StdDirectoryDesc) Default* (): Font;
  VAR tf: Typeface;
BEGIN
  tf := "Segoe UI";
  RETURN d.This(tf, 12 * point, {}, normal)
END Default;

PROCEDURE (d: StdDirectoryDesc) TypefaceList* (): TypefaceInfo;
BEGIN
  RETURN NIL
END TypefaceList;

(* Install the host-supplied font directory. HostFonts.Init calls
   this exactly once during startup. *)
PROCEDURE SetDir* (d: Directory);
BEGIN
  ASSERT(d # NIL, 20);
  dir := d
END SetDir;

BEGIN
  NEW(stdDir);
  dir := stdDir

END Fonts.
