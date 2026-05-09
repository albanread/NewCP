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

VAR
  (* Read-only public. HostFonts.Init registers the concrete
     directory by calling SetDir below. *)
  dir-: Directory;

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

(* Install the host-supplied font directory. HostFonts.Init calls
   this exactly once during startup. *)
PROCEDURE SetDir* (d: Directory);
BEGIN
  ASSERT(d # NIL, 20);
  dir := d
END SetDir;

END Fonts.
