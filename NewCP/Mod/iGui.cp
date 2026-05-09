DEFINITION MODULE iGui;

(* Integrated GUI bridge. Phase 2 surface: typed event mailbox.

   Field semantics for NextEvent's VAR outputs depend on `kind`. See the
   Rust source [`newcp-runtime/src/igui/cp_exports.rs`] for the full
   table; the most important ones for Phase 2 are:

     EvKey:   p1=vkey, p2=scancode, p3=mods, p4=down(1)/up(0) | repeat<<16
     EvChar:  p1=codepoint, p2=mods
     EvMouse: p1=x, p2=y, p3=mods | button<<8 | op<<16,
              p4=wheel_delta | wheel_lines<<16
     EvResize: p1=width, p2=height
     EvDpiChange: p1=dpi_x*100, p2=dpi_y*100
     EvFrameClose: all zero — terminate the loop.
*)

CONST
  (* event kinds *)
  EvNone*        = 0;
  EvKey*         = 1;
  EvChar*        = 2;
  EvMouse*       = 3;
  EvFocus*       = 4;
  EvResize*      = 5;
  EvPaint*       = 6;
  EvClose*       = 7;
  EvFrameClose*  = 8;
  EvMenu*        = 9;
  EvThemeChange* = 10;
  EvDpiChange*   = 11;
  EvSurfaceReply* = 12;
  EvTick*        = 13;

  (* mouse op sub-kinds packed into p3 high byte *)
  MouseMove*       = 0;
  MouseLeftDown*   = 1;
  MouseLeftUp*     = 2;
  MouseRightDown*  = 3;
  MouseRightUp*    = 4;
  MouseMiddleDown* = 5;
  MouseMiddleUp*   = 6;
  MouseWheel*      = 7;

  (* modifier bits *)
  ModShift*   = 1;
  ModControl* = 2;
  ModAlt*     = 4;
  ModWin*     = 8;
  ModCaps*    = 16;

  (* cursor kinds for SetCursor *)
  CrArrow*       = 0;
  CrIBeam*       = 1;
  CrCrosshair*   = 2;
  CrHand*        = 3;
  CrWait*        = 4;
  CrResizeNS*    = 5;
  CrResizeEW*    = 6;
  CrResizeNESW*  = 7;
  CrResizeNWSE*  = 8;
  CrSizeAll*     = 9;
  CrNotAllowed*  = 10;
  CrHelp*        = 11;
  CrAppStarting* = 12;

  (* font style for text descriptors *)
  FsNormal*  = 0;
  FsItalic*  = 1;
  FsOblique* = 2;

  (* font stretch *)
  FwUltraCondensed* = 1;
  FwExtraCondensed* = 2;
  FwCondensed*      = 3;
  FwSemiCondensed*  = 4;
  FwNormal*         = 5;
  FwSemiExpanded*   = 6;
  FwExpanded*       = 7;
  FwExtraExpanded*  = 8;
  FwUltraExpanded*  = 9;

  (* text alignment *)
  AlignLeading*   = 0;
  AlignTrailing*  = 1;
  AlignCenter*    = 2;
  AlignJustified* = 3;

  (* text trimming *)
  TrimNone*         = 0;
  TrimEllipsisChar* = 1;
  TrimEllipsisWord* = 2;

  (* MarkRect modes *)
  MarkHighlight* = 0;
  MarkInvert*    = 1;
  MarkDim25*     = 2;
  MarkDim50*     = 3;
  MarkDim75*     = 4;

  (* Path stroke caps *)
  CapFlat*   = 0;
  CapRound*  = 1;
  CapSquare* = 2;

  (* Path stroke joins *)
  JoinMiter* = 0;
  JoinRound* = 1;
  JoinBevel* = 2;

  (* System color kinds *)
  ScWindowBg*    = 0;
  ScWindowFg*    = 1;
  ScControlBg*   = 2;
  ScControlFg*   = 3;
  ScSelectionBg* = 4;
  ScSelectionFg* = 5;
  ScHighlightBg* = 6;
  ScHighlightFg* = 7;
  ScDisabledFg*  = 8;
  ScCaret*       = 9;
  ScDialogBg*    = 10;
  ScDialogFg*    = 11;

(* Block on the event mailbox. Returns 1 if an event was delivered, 0 on
   timeout. timeoutMs < 0 blocks indefinitely. *)
PROCEDURE NextEvent*(VAR kind, childId, timeMs, p1, p2, p3, p4: INTEGER;
                     timeoutMs: INTEGER): INTSHORT;

(* Post WM_CLOSE to the iGui frame. The frame tears down on its own. *)
PROCEDURE Quit*;

(* Open an MDI child window with the given title. The new child has its
   own swap chain and Direct2D bitmap target. Returns 1 on success, 0
   on failure (typically: frame not running, or MDI client not yet
   created). *)
PROCEDURE OpenChild*(title: ARRAY OF SHORTCHAR;
                     VAR childId: INTEGER): INTSHORT;

(* Close an MDI child by id. Returns 1 if the child existed, 0 if the
   id was unknown. *)
PROCEDURE CloseChild*(childId: INTEGER): INTSHORT;

(* Update the title bar of an MDI child. *)
PROCEDURE SetTitle*(childId: INTEGER; title: ARRAY OF SHORTCHAR);

(* ── Phase 3b: surface batch builder ──────────────────────────────
   Build a batch by calling BeginBatch(childId), then any number of
   Emit* procedures, then SubmitBatch(). A new batch supersedes the
   previous one for the same child; whatever is in the latest batch
   defines what the child paints on every WM_PAINT. *)

PROCEDURE BeginBatch*(childId: INTEGER);
PROCEDURE SubmitBatch*(): INTSHORT;

(* lifecycle *)
PROCEDURE EmitClear*(r, g, b, a: REAL);
PROCEDURE EmitPresentHint*;

(* geometry *)
PROCEDURE EmitFillRect*(x0, y0, x1, y1, cornerRadius,
                        r, g, b, a: REAL);
PROCEDURE EmitStrokeRect*(x0, y0, x1, y1, cornerRadius, halfThickness,
                          r, g, b, a: REAL);
PROCEDURE EmitDrawLine*(x0, y0, x1, y1, halfThickness,
                        r, g, b, a: REAL);

(* Phase 3c geometry primitives *)
PROCEDURE EmitFillOval*(x0, y0, x1, y1,
                        r, g, b, a: REAL);
PROCEDURE EmitFillCircle*(cx, cy, radius,
                          r, g, b, a: REAL);
PROCEDURE EmitStrokeOval*(x0, y0, x1, y1, halfThickness,
                          r, g, b, a: REAL);
PROCEDURE EmitStrokeCircle*(cx, cy, radius, halfThickness,
                            r, g, b, a: REAL);
PROCEDURE EmitDrawArc*(cx, cy, radius,
                       rotationRad, halfApertureRad, halfThickness,
                       r, g, b, a: REAL);

(* ── Phase 3c: per-child cursor + DPI ─────────────────────────────
   GetDpi reads the child's effective DPI. Returns 1 on success, 0 if
   the child id is unknown. dpiX = dpiY = 96.0 at 100% scaling.
   SetCursor sets the cursor shape applied while the pointer is over
   the child's render area. Picks one of the Cr* constants above. *)

PROCEDURE GetDpi*(childId: INTEGER; VAR dpiX, dpiY: REAL): INTSHORT;
PROCEDURE SetCursor*(childId: INTEGER; kind: INTSHORT);

(* ── Phase 4: text via DirectWrite ────────────────────────────────
   DrawTextRun is async — added to the current batch and rendered on
   the next paint. Measure/CharIndexAtPoint/PointAtCharIndex submit
   their own one-command batch and block on the GUI thread reply
   channel for up to 5 seconds. All four resolve against the same
   DirectWrite layout for the given (text, family, size, weight,
   style, stretch, locale) tuple, so draw and hit-test geometry
   agree.

   weight is 100..900 (DWRITE_FONT_WEIGHT). style/stretch/alignment/
   trimming pick from the Fs* / Fw* / Align* / Trim* constants
   above. maxWidth < 0 disables wrap. family/locale empty strings
   default to "Segoe UI" / "en-us". *)

PROCEDURE EmitDrawTextRun*(text: ARRAY OF SHORTCHAR;
                           x, y, fontSize: REAL;
                           family: ARRAY OF SHORTCHAR;
                           weight, style, stretch: INTSHORT;
                           locale: ARRAY OF SHORTCHAR;
                           maxWidth: REAL;
                           alignment, trimming: INTSHORT;
                           r, g, b, a: REAL);

PROCEDURE MeasureTextRun*(childId: INTEGER;
                          text: ARRAY OF SHORTCHAR;
                          fontSize: REAL;
                          family: ARRAY OF SHORTCHAR;
                          weight, style, stretch: INTSHORT;
                          locale: ARRAY OF SHORTCHAR;
                          maxWidth: REAL;
                          alignment, trimming: INTSHORT;
                          VAR width, height, ascent: REAL;
                          VAR lineCount: INTEGER): INTSHORT;

PROCEDURE CharIndexAtPoint*(childId: INTEGER;
                            text: ARRAY OF SHORTCHAR;
                            fontSize: REAL;
                            family: ARRAY OF SHORTCHAR;
                            weight, style, stretch: INTSHORT;
                            locale: ARRAY OF SHORTCHAR;
                            x, y: REAL;
                            VAR charIndex: INTEGER;
                            VAR isInside, isTrailing: INTSHORT): INTSHORT;

PROCEDURE PointAtCharIndex*(childId: INTEGER;
                            text: ARRAY OF SHORTCHAR;
                            fontSize: REAL;
                            family: ARRAY OF SHORTCHAR;
                            weight, style: INTSHORT;
                            charIndex: INTEGER;
                            VAR x, y, height: REAL): INTSHORT;

(* ── Phase 5: composition + overlays + paths + system colors ─────
   Composition state is per-batch: clip and offset stacks reset to
   identity at BeginBatch. Mismatched push/pop or unknown SaveRect
   slots produce a runtime warning; the renderer recovers but the
   results may be visually wrong. *)

PROCEDURE EmitPushClipRect*(x0, y0, x1, y1: REAL);
PROCEDURE EmitPopClipRect*;
PROCEDURE EmitPushOffset*(dx, dy: REAL);
PROCEDURE EmitPopOffset*;
PROCEDURE EmitScrollRect*(x0, y0, x1, y1, dx, dy: REAL);
PROCEDURE EmitSaveRect*(slot: INTSHORT; x0, y0, x1, y1: REAL);
PROCEDURE EmitRestoreRect*(slot: INTSHORT);
PROCEDURE EmitInstallChildViewBounds*(childViewId: INTSHORT;
                                       x0, y0, x1, y1: REAL);

(* ── Phase 5: overlays ────────────────────────────────────────────
   MarkRect picks one of MarkHighlight / MarkInvert / MarkDimNN and
   resolves the actual color from the system palette at draw time.
   Caret and SelectionRange take an explicit RGBA so the caller can
   match its own theme. FocusRing strokes a rounded rect; same shape
   as StrokeRect but provided as its own command so the platform can
   later swap in a system focus visual. *)

PROCEDURE EmitMarkRect*(x0, y0, x1, y1: REAL; mode: INTSHORT);
PROCEDURE EmitCaret*(x0, y0, x1, y1, r, g, b, a: REAL);
PROCEDURE EmitSelectionRange*(x0, y0, x1, y1, r, g, b, a: REAL);
PROCEDURE EmitFocusRing*(x0, y0, x1, y1, cornerRadius, halfThickness,
                         r, g, b, a: REAL);

(* ── Phase 5: path builder ────────────────────────────────────────
   Build a path with PathBegin / PathMoveTo / PathLineTo / PathQuadTo
   / PathCubicTo / PathArcTo / PathClose, then call EmitPath to push
   a DrawPath into the active batch. fillMode and strokeMode are 0/1
   flags. strokeDashLen != 0 enables a default 4-on/4-off dash. *)

PROCEDURE PathBegin*;
PROCEDURE PathMoveTo*(x, y: REAL);
PROCEDURE PathLineTo*(x, y: REAL);
PROCEDURE PathQuadTo*(cx, cy, ex, ey: REAL);
PROCEDURE PathCubicTo*(c1x, c1y, c2x, c2y, ex, ey: REAL);
PROCEDURE PathArcTo*(rx, ry, rotationRad: REAL;
                     largeArc, sweepClockwise: INTSHORT;
                     ex, ey: REAL);
PROCEDURE PathClose*;
PROCEDURE EmitPath*(fillMode: INTSHORT;
                    fillR, fillG, fillB, fillA: REAL;
                    strokeMode: INTSHORT;
                    strokeHalfThickness: REAL;
                    strokeCap, strokeJoin: INTSHORT;
                    strokeMiter: REAL;
                    strokeDashLen: INTSHORT;
                    strokeR, strokeG, strokeB, strokeA: REAL): INTSHORT;

(* ── Phase 5: system colors ───────────────────────────────────────
   Read the current theme palette. Returns 1 always (defaults to
   sensible mid-light values until the first WM_SYSCOLORCHANGE
   refresh). EvThemeChange events fire when the OS theme changes. *)

PROCEDURE SystemColor*(kind: INTSHORT;
                       VAR r, g, b, a: REAL): INTSHORT;

(* ── Phase 6: menu + MDI verbs ────────────────────────────────────
   SetMenu installs a menu bar on the iGui frame from a compact
   line-oriented spec:

     MENU File
     ITEM 1001 New
     ITEM 1002 Open
     SEP
     ITEM 1099 Exit
     MENU Window
     MDI cascade
     MDI tile-h
     MDI tile-v
     MDI close-all
     MDI arrange-icons

   ITEM ids must fit in 16 bits and should sit in 0x1000..=0x1FFF
   (the MDI verb range 0x2000..=0x2010 is reserved for the auto-
   allocated MDI items). User-item clicks arrive as EvMenu events
   with item_id = the chosen id; MDI verb clicks are dispatched
   directly inside iGui — no event reaches the language thread.

   Standard MDI verbs can also be invoked directly from CP code
   regardless of any menu being installed. *)

PROCEDURE SetMenu*(spec: ARRAY OF SHORTCHAR): INTSHORT;
PROCEDURE MdiCascade*;
PROCEDURE MdiTileH*;
PROCEDURE MdiTileV*;
PROCEDURE MdiCloseAll*;
PROCEDURE MdiArrangeIcons*;

(* ── Animation tick ───────────────────────────────────────────────
   SetRedrawRate schedules an EvTick event for childId every
   intervalMs milliseconds. intervalMs <= 0 disables the timer.
   Win32 auto-coalesces queued WM_TIMERs, so a backed-up language
   thread sees at most one tick per child per drain cycle.

   Returns 1 on success, 0 if childId is unknown. *)

PROCEDURE SetRedrawRate*(childId: INTEGER; intervalMs: INTEGER): INTSHORT;

(* ── Diagnostics ──────────────────────────────────────────────────
   LayoutCacheStats reads the DirectWrite layout cache counters.
   `hits`/`misses` are monotonic since process start; `size` is the
   current entry count (capped at 256, LRU-evicted). Useful for
   tests and demos that want to confirm the cache is actually
   serving repeats. *)

PROCEDURE LayoutCacheStats*(VAR hits, misses, size: INTEGER): INTSHORT;

(* Failover log (Rust-owned ring buffer).

   LogAppend writes one line to the always-on log buffer maintained
   inside iGui itself. Identical adjacent lines coalesce — the log
   view shows them as `(xN) message` instead of producing duplicates,
   which is what you want when something starts spinning out the same
   panic message. The log view is opened from `Tools > Log` or
   Ctrl+Shift+L, sits on the left of the MDI client by default, and
   survives a language-thread fault: the buffer lives on the UI side,
   so it stays readable even when CP code has stopped running. *)

PROCEDURE LogAppend*(s: ARRAY OF SHORTCHAR);

(* Standalone font measurement service.

   Synchronous, child-id-less, batch-less DirectWrite measurement.
   The caller passes a typeface name + DIP size + weight + italic
   flag; the service builds a one-shot DirectWrite text-format and
   layout, reads metrics, and returns. Safe to call from any thread
   because IDWriteFactory2 is free-threaded.

   These primitives are the foundation of the CP-side `Fonts` /
   `HostFontsSys` / `HostFonts` stack: the Sys layer wraps these to
   expose CP-shaped helpers, and `HostFonts` converts BlackBox
   sub-millimeter units to/from DIPs at the boundary.

   Both procedures return 1 on success and 0 on failure (typically
   "DirectWrite couldn't create a format for that typeface" — caller
   should retry with a fallback family). *)

PROCEDURE MeasureFont*(
    family: ARRAY OF SHORTCHAR;
    size: REAL;             (* DIPs *)
    weight: INTEGER;        (* 400=normal, 700=bold *)
    italic: INTSHORT;       (* 0 = upright, 1 = italic *)
    OUT ascent, descent, lineHeight, advanceM: REAL): INTSHORT;

PROCEDURE MeasureString*(
    s: ARRAY OF SHORTCHAR;
    family: ARRAY OF SHORTCHAR;
    size: REAL;
    weight: INTEGER;
    italic: INTSHORT;
    OUT width: REAL): INTSHORT;

END iGui.
