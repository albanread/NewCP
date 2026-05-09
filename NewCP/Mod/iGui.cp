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

END iGui.
