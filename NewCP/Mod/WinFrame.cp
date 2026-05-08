DEFINITION MODULE WinFrame;

(* Interface only. The actual implementations are Rust-hosted exports
   registered by winframe_module_artifact() in newcp-runtime. Do not
   convert this back to a regular MODULE — the loader would JIT-compile
   the empty bodies and shadow the real Rust shims. *)

CONST
  BufFrame* = 0;
  BufPersistent* = 1;

PROCEDURE FrameIndex*(): INTEGER;

PROCEDURE ElapsedMs*(): INTEGER;

PROCEDURE DeltaMs*(): INTEGER;

PROCEDURE ResolvePaneId*(nodeId: ARRAY OF SHORTCHAR; VAR paneId: INTEGER): INTSHORT;

PROCEDURE PaneLayout*(paneId: INTEGER;
                      VAR x, y, width, height: INTSHORT): INTSHORT;

PROCEDURE RequestPresent*;

PROCEDURE PostPaneMsg*(paneId: INTEGER;
                       kind, detail: ARRAY OF SHORTCHAR): INTSHORT;

PROCEDURE PollPaneMsg*(paneId: INTEGER;
                       VAR kind: ARRAY OF SHORTCHAR;
                       VAR detail: ARRAY OF SHORTCHAR): INTSHORT;

END WinFrame.
