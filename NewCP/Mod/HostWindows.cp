MODULE HostWindows;
(**
   HostWindows — Component Pascal interface to the wingui spec_bind host.

   These procedure stubs are backed by Rust shims in newcp-runtime/src/wingui_host.rs.
   The Rust side stores the WinguiSpecBindRuntime pointer in a process-wide OnceLock
   so CP code never sees a raw pointer.

   All events arrive via WaitNamedEvent.  The event name is a short string
   (e.g. "button_id", "__close_requested", "__host_stopping", "input_id").
   The payload is a JSON string with the event details.

   Well-known event names:
     "__close_requested"   User clicked the close button
     "__host_stopping"     Host is shutting down
*)

(**
   RequestPresent — ask the UI thread to flush the current frame to screen.
   Usually a no-op when auto_request_present is set; included for completeness.
*)
PROCEDURE RequestPresent*;
BEGIN
END RequestPresent;

(**
   RequestClose — ask the UI thread to close the application.
*)
PROCEDURE RequestClose*;
BEGIN
END RequestClose;

(**
   PublishUi — send a JSON string that replaces the entire native UI layout.
   `json` must be a null-terminated SHORTCHAR string.
*)
PROCEDURE PublishUi*(json: ARRAY OF SHORTCHAR);
BEGIN
END PublishUi;

(**
   PatchUi — reserved; spec_bind reconciles patches internally via PublishUi.
*)
PROCEDURE PatchUi*(patch: ARRAY OF SHORTCHAR);
BEGIN
END PatchUi;

(**
   WaitNamedEvent — block until the next UI event arrives or timeoutMs elapses.
   name    receives the event name (e.g. "__close_requested", a widget event id).
   payload receives the event payload as a JSON string.
   timeoutMs: milliseconds to wait; pass -1 to block indefinitely.
   Returns 1 if an event was delivered, 0 if the timeout elapsed.
*)
PROCEDURE WaitNamedEvent*(VAR name: ARRAY OF SHORTCHAR;
                          VAR payload: ARRAY OF SHORTCHAR;
                          timeoutMs: INTEGER): INTEGER;
BEGIN
  RETURN 0
END WaitNamedEvent;

END HostWindows.

   These procedure stubs are backed by Rust shims in newcp-runtime/src/wingui_host.rs.
   The Rust side stores the SuperTerminalClientContext in a process-wide OnceLock so
   CP code never sees a raw pointer.

   Event type constants (returned by WaitEvent):
     evNone          = 0   No event / timeout elapsed
     evKey           = 1   Keyboard key press or release
     evChar          = 2   Character input (Unicode codepoint)
     evMouse         = 3   Mouse move / button
     evPaneInput     = 4   Mouse or keyboard routed to a specific pane
     evResize        = 5   Window resized
     evFocus         = 6   Window gained or lost focus
     evNativeUi      = 7   Native UI control interaction (JSON payload)
     evCloseRequest  = 8   User clicked the close button
     evHostStopping  = 9   Host is shutting down
     evWindowCreated = 10  A secondary window was created
     evWindowClosed  = 11  A secondary window was closed
*)

CONST
  evNone*          = 0;
  evKey*           = 1;
  evChar*          = 2;
  evMouse*         = 3;
  evPaneInput*     = 4;
  evResize*        = 5;
  evFocus*         = 6;
  evNativeUi*      = 7;
  evCloseRequest*  = 8;
  evHostStopping*  = 9;
  evWindowCreated* = 10;
  evWindowClosed*  = 11;

  waitInfinite* = 0FFFFFFFFH;

(**
   RequestPresent — ask the UI thread to flush the current frame to screen.
   Call after a batch of drawing commands.
*)
PROCEDURE RequestPresent*;
BEGIN
END RequestPresent;

(**
   RequestClose — ask the UI thread to close the application.
*)
PROCEDURE RequestClose*;
BEGIN
END RequestClose;

(**
   PublishUi — send a JSON string that replaces the entire native UI layout
   for the default window.  `json` must be a null-terminated SHORTCHAR array.
*)
PROCEDURE PublishUi*(json: ARRAY OF SHORTCHAR);
BEGIN
END PublishUi;

(**
   PatchUi — send a JSON-Patch document that updates the native UI layout
   for the default window.  `patch` must be a null-terminated SHORTCHAR array.
*)
PROCEDURE PatchUi*(patch: ARRAY OF SHORTCHAR);
BEGIN
END PatchUi;

(**
   WaitEvent — block until an event arrives or `timeoutMs` milliseconds elapse.
   Pass `waitInfinite` to block indefinitely.
   Returns the event-type constant (evNone if timeout).
   `outEvent` receives the raw event bytes (at least 544 bytes recommended).
*)
PROCEDURE WaitEvent*(VAR outEvent: ARRAY OF BYTE; timeoutMs: INTEGER): INTEGER;
BEGIN
  RETURN 0
END WaitEvent;

BEGIN
END HostWindows.
