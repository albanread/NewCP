MODULE HostFonts;

(* BlackBox-faithful concrete Fonts implementation backed by iGui.

   This module has no public API: its only responsibility is to
   register a concrete `Fonts.Directory` at startup. Everything else
   reaches the implementation through `Fonts.dir`. The internal type
   names are deliberately *not* `Font` / `Directory` to avoid
   shadowing the same-named exports from Fonts (NewCP sema currently
   recurses infinitely on cross-module name collisions, even when
   the local type extends the imported one).

   Cell-metrics and string-width methods convert between BlackBox
   sub-millimeter units (which Fonts.FontDesc.size stores) and DIPs
   (which HostFontsSys speaks).

   Unit conversion:
     Fonts.point = 12700 sub-mm units
     1 inch = 72 points = 96 DIPs    (a DIP is 1/96 inch)
     ⇒ 12700 BB units / point × 72 points / inch ÷ 96 DIPs / inch
        = 9525 BB units per DIP
   So `dip := bb / 9525.0` and `bb := ENTIER(dip * 9525.0 + 0.5)`. *)

IMPORT Fonts, HostFontsSys;

CONST
  bbPerDip          = 9525.0;
  defaultTypeface   = "Cascadia Mono";
  defaultSizePoints = 10;     (* 10pt body — same default as redit *)

TYPE
  fontImpl     = RECORD (Fonts.FontDesc) END;
  fontImplPtr  = POINTER TO fontImpl;

  dirImpl      = RECORD (Fonts.DirectoryDesc) END;
  dirImplPtr   = POINTER TO dirImpl;

VAR
  hostDir: dirImplPtr;   (* singleton; also stored in Fonts.dir *)

PROCEDURE BBToDip(bb: INTEGER): REAL;
BEGIN
  RETURN bb / bbPerDip
END BBToDip;

PROCEDURE DipToBB(dip: REAL): INTEGER;
BEGIN
  RETURN SHORT(ENTIER(dip * bbPerDip + 0.5))
END DipToBB;

(* Concrete Font method overrides. Each one converts the Font's stored
   sub-mm size to a DIP size, calls HostFontsSys, converts the
   returned DIP measurements back to sub-mm. *)

PROCEDURE (f: fontImpl) GetBounds*
  (OUT asc, dsc, w: INTEGER);
  VAR
    sizeDip, ascDip, dscDip, lhDip, advDip: REAL;
    ok: INTSHORT;
BEGIN
  sizeDip := BBToDip(f.size);
  ok := HostFontsSys.MeasureFont(
    f.typeface, sizeDip, f.weight, f.style,
    ascDip, dscDip, lhDip, advDip);
  IF ok = 0 THEN
    (* Fallback if iGui couldn't satisfy the typeface. Sane numbers
       based purely on size. *)
    asc := SHORT(ENTIER(0.8 * f.size + 0.5));
    dsc := f.size - asc;
    w   := SHORT(ENTIER(0.5 * f.size + 0.5))
  ELSE
    asc := DipToBB(ascDip);
    dsc := DipToBB(dscDip);
    w   := DipToBB(advDip)
  END
END GetBounds;

PROCEDURE (f: fontImpl) StringWidth*
  (IN s: ARRAY OF CHAR): INTEGER;
  VAR
    sizeDip, widthDip: REAL;
    ok: INTSHORT;
BEGIN
  sizeDip := BBToDip(f.size);
  ok := HostFontsSys.StringWidth(
    s, f.typeface, sizeDip, f.weight, f.style, widthDip);
  IF ok = 0 THEN
    RETURN 0
  END;
  RETURN DipToBB(widthDip)
END StringWidth;

PROCEDURE (f: fontImpl) SStringWidth*
  (IN s: ARRAY OF SHORTCHAR): INTEGER;
  VAR
    sizeDip, widthDip: REAL;
    ok: INTSHORT;
BEGIN
  sizeDip := BBToDip(f.size);
  ok := HostFontsSys.SStringWidth(
    s, f.typeface, sizeDip, f.weight, f.style, widthDip);
  IF ok = 0 THEN
    RETURN 0
  END;
  RETURN DipToBB(widthDip)
END SStringWidth;

PROCEDURE (f: fontImpl) IsAlien* (): BOOLEAN;
BEGIN
  (* DirectWrite always falls back to a similar typeface rather than
     failing, so we never report "alien". When we add the ability to
     check whether the requested family was actually resolved we can
     surface that here. *)
  RETURN FALSE
END IsAlien;

(* Build a Font with the given parameters. The caller's typeface name
   is copied as-is; sentinels like Fonts.default ("*") are interpreted
   here to mean "swap in the host default before constructing". *)
PROCEDURE (d: dirImpl) This*
  (typeface: Fonts.Typeface; size: INTEGER;
   style: SET; weight: INTEGER): Fonts.Font;
  VAR f: fontImplPtr;
BEGIN
  NEW(f);
  IF typeface = Fonts.default THEN
    typeface := defaultTypeface
  END;
  (* Initialize fields directly rather than calling the inherited
     Fonts.FontDesc.Init: the JIT can't currently patch a vtable slot
     that points across module boundaries (see
     docs/files_module_investigation.md, item 2). The asserts in the
     original Init are duplicated here to preserve the invariant. *)
  ASSERT(f.size = 0, 20);
  ASSERT(size # 0, 21);
  f.typeface := typeface;
  f.size := size;
  f.style := style;
  f.weight := weight;
  RETURN f
END This;

PROCEDURE (d: dirImpl) Default* (): Fonts.Font;
  VAR
    tf: Fonts.Typeface;
BEGIN
  tf := defaultTypeface;
  RETURN d.This(tf, defaultSizePoints * Fonts.point, {}, Fonts.normal)
END Default;

PROCEDURE (d: dirImpl) TypefaceList* (): Fonts.TypefaceInfo;
BEGIN
  (* TODO: enumerate the system font collection through a future
     iGui shim. For now we return NIL — callers that need the list
     should treat NIL as "directory doesn't enumerate". *)
  RETURN NIL
END TypefaceList;

BEGIN
  NEW(hostDir);
  Fonts.SetDir(hostDir)
END HostFonts.
