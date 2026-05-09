MODULE HostFontsSys;

(* CP-shaped wrapper over the iGui font_metrics service.

   This is the Sys-layer between HostFonts (BlackBox-faithful concrete
   font impl) and iGui (the native runtime). Only this module imports
   iGui. HostFonts and Fonts above stay free of iGui specifics so the
   BlackBox-faithful surface is recognizable to anyone who knows
   BlackBox.

   Inputs are in DIPs (device-independent pixels). The unit conversion
   between BlackBox sub-millimeter coordinates and DIPs happens up in
   HostFonts at the boundary, not here.

   The Fonts.style SET (with the `italic`, `underline`, `strikeout`
   bits) is collapsed to a single italic flag here — DirectWrite
   handles underline/strikeout via separate render passes that the
   measurement primitives don't care about. Underline/strikeout will
   be re-introduced when the corresponding DrawString variant lands. *)

IMPORT iGui, Fonts;

PROCEDURE ItalicFlag(style: SET): INTSHORT;
BEGIN
  IF Fonts.italic IN style THEN
    RETURN SHORT(1)
  ELSE
    RETURN SHORT(0)
  END
END ItalicFlag;

(* Narrow a CHAR string into a SHORTCHAR scratch buffer.
   ASCII / Latin-1 round-trips losslessly; out-of-range codepoints
   become "?". `dst` should be at least one byte longer than the
   number of source chars to leave room for the NUL terminator. *)
PROCEDURE Narrow(IN src: ARRAY OF CHAR; VAR dst: ARRAY OF SHORTCHAR);
  VAR i, c: INTEGER;
BEGIN
  i := 0;
  WHILE (i < LEN(src) - 1) & (i < LEN(dst) - 1) & (src[i] # 0X) DO
    c := ORD(src[i]);
    IF c > 0FFH THEN c := ORD("?") END;
    dst[i] := SHORT(CHR(c));
    INC(i)
  END;
  dst[i] := 0X
END Narrow;

(* Get cell metrics for a typeface at a DIP size. Returns 1 on
   success, 0 if iGui couldn't satisfy the request (typically a
   typeface DirectWrite refused — the caller should retry with a
   fallback family). *)
PROCEDURE MeasureFont*
  (IN family: ARRAY OF CHAR;
   sizeDip: REAL;
   weight: INTEGER;
   style: SET;
   OUT ascentDip, descentDip, lineHeightDip, advanceMDip: REAL): INTSHORT;
  VAR sf: ARRAY 64 OF SHORTCHAR;
BEGIN
  Narrow(family, sf);
  RETURN iGui.MeasureFont(
    sf, sizeDip, weight, ItalicFlag(style),
    ascentDip, descentDip, lineHeightDip, advanceMDip)
END MeasureFont;

(* Measure the rendered width of a CHAR string. The native side
   converts to UTF-16 internally; for CHAR (32-bit codepoint in NewCP)
   we narrow to SHORTCHAR at this boundary. ASCII-clean source — the
   overwhelming majority of text in BlackBox views — round-trips
   losslessly; non-Latin-1 codepoints become "?". When DrawString
   needs full-fidelity CHAR support we'll widen the iGui shim to take
   ARRAY OF CHAR directly. *)
PROCEDURE StringWidth*
  (IN s: ARRAY OF CHAR;
   IN family: ARRAY OF CHAR;
   sizeDip: REAL;
   weight: INTEGER;
   style: SET;
   OUT widthDip: REAL): INTSHORT;
  VAR
    sf: ARRAY 64 OF SHORTCHAR;
    ss: ARRAY 4096 OF SHORTCHAR;
BEGIN
  Narrow(family, sf);
  Narrow(s, ss);
  RETURN iGui.MeasureString(ss, sf, sizeDip, weight,
                            ItalicFlag(style), widthDip)
END StringWidth;

(* SHORTCHAR string variant — only the family is narrowed. *)
PROCEDURE SStringWidth*
  (IN s: ARRAY OF SHORTCHAR;
   IN family: ARRAY OF CHAR;
   sizeDip: REAL;
   weight: INTEGER;
   style: SET;
   OUT widthDip: REAL): INTSHORT;
  VAR sf: ARRAY 64 OF SHORTCHAR;
BEGIN
  Narrow(family, sf);
  RETURN iGui.MeasureString(s, sf, sizeDip, weight,
                            ItalicFlag(style), widthDip)
END SStringWidth;

END HostFontsSys.
