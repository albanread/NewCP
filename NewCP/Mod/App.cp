MODULE App;
(**
   App — NewCP GUI application entry point.

   Run* is invoked by the Rust driver as the startup command when the
   user calls `newcp-driver run-gui`.

   Responsibilities:
     1. Configure and open the log view (Log.Open).
     2. Run the event loop, blocking on HostWindows.WaitNamedEvent.
     3. Exit cleanly on "__close_requested" or "__host_stopping".

   All output to the log window goes through Log.*; no direct WinSpec or
   HostWindows calls are needed here except for the event loop itself.
*)

IMPORT HostWindows, Log;

(* Compare two SHORTCHAR strings element-by-element.  Returns TRUE if equal. *)
PROCEDURE StrEq(a, b: ARRAY OF SHORTCHAR): BOOLEAN;
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (a[i] = b[i]) & (a[i] # 0X) DO INC(i) END;
  RETURN a[i] = b[i]
END StrEq;

PROCEDURE Run*;
  VAR
    name:    ARRAY 256  OF SHORTCHAR;
    payload: ARRAY 4096 OF SHORTCHAR;
    ok:      INTEGER;
BEGIN
  Log.SetTitle("NewCP");
  Log.Open;
  Log.String("NewCP ready."); Log.Ln;

  LOOP
    ok := HostWindows.WaitNamedEvent(name, payload, -1);
    IF ok = 0 THEN (* timeout — not expected with infinite wait *)
    ELSIF StrEq(name, "__close_requested") OR StrEq(name, "__host_stopping") THEN
      EXIT
    END
  END
END Run;

END App.
