MODULE LogProbe;

(* iGui sample — exercises the failover Rust log buffer.

   Writes a handful of varied lines to `iGui.LogAppend` (which lands
   in the process-wide ring on the UI side), then a burst of fifty
   identical lines to demonstrate adjacent-message coalescing. After
   that, a 500 ms tick emits one new line per beat so you can watch
   entries pile up live in the log view.

   To see the result, run:
     newcp-driver run-igui LogProbe.Run

   …and open `Tools > Log` (or press Ctrl+Shift+L) once the frame is
   up. Closing the frame ends the demo.

   The whole point is durability — even if this controller crashes
   mid-loop the log entries it produced before the crash stay
   readable in the view, because the buffer lives on the UI thread. *)

IMPORT iGui, Console;

CONST
  TickInterval = 500;
  BurstCount   = 50;

VAR
  tickCount: INTEGER;

PROCEDURE WriteIntInto(VAR buf: ARRAY OF SHORTCHAR;
                       VAR pos: INTEGER; n: INTEGER);
  VAR base: INTEGER;
BEGIN
  IF n = 0 THEN
    buf[pos] := SHORT("0"); INC(pos);
    RETURN
  END;
  IF n < 0 THEN
    buf[pos] := SHORT("-"); INC(pos);
    n := -n
  END;
  base := 1;
  WHILE base * 10 <= n DO base := base * 10 END;
  WHILE base > 0 DO
    buf[pos] := SHORT(CHR(ORD("0") + (n DIV base) MOD 10));
    INC(pos);
    base := base DIV 10
  END
END WriteIntInto;

PROCEDURE EmitVaried;
  VAR msg: ARRAY 80 OF SHORTCHAR;
BEGIN
  msg := "LogProbe starting"; iGui.LogAppend(msg);
  msg := "loading module foo"; iGui.LogAppend(msg);
  msg := "loading module bar"; iGui.LogAppend(msg);
  msg := "connecting to widget service"; iGui.LogAppend(msg);
  msg := "ready"; iGui.LogAppend(msg)
END EmitVaried;

PROCEDURE EmitBurst;
  (* Send the SAME message BurstCount times in a row. The UI side
     should coalesce these into a single entry with a count badge
     ((xN)) — that's the whole reason the ring tracks counts. *)
  VAR
    msg: ARRAY 80 OF SHORTCHAR;
    i: INTEGER;
BEGIN
  msg := "retrying connection...";
  i := 0;
  WHILE i < BurstCount DO
    iGui.LogAppend(msg);
    INC(i)
  END
END EmitBurst;

PROCEDURE EmitTick;
  VAR
    msg: ARRAY 64 OF SHORTCHAR;
    pos: INTEGER;
BEGIN
  INC(tickCount);
  pos := 0;
  msg[pos] := SHORT("t"); INC(pos);
  msg[pos] := SHORT("i"); INC(pos);
  msg[pos] := SHORT("c"); INC(pos);
  msg[pos] := SHORT("k"); INC(pos);
  msg[pos] := SHORT(" "); INC(pos);
  WriteIntInto(msg, pos, tickCount);
  msg[pos] := 0X;
  iGui.LogAppend(msg)
END EmitTick;

PROCEDURE Run*;
  VAR
    kind, childId, timeMs, p1, p2, p3, p4: INTEGER;
    ok: INTSHORT;
    probeId: INTEGER;
    msg: ARRAY 80 OF SHORTCHAR;
BEGIN
  Console.WriteShortString("LogProbe: emits to iGui.LogAppend"); Console.WriteLn;
  Console.WriteShortString("open Tools > Log (Ctrl+Shift+L) to view"); Console.WriteLn;

  ok := iGui.OpenChild("Log Probe", probeId);
  IF ok = 0 THEN RETURN END;

  tickCount := 0;
  EmitVaried;
  EmitBurst;
  msg := "burst complete"; iGui.LogAppend(msg);

  ok := iGui.SetRedrawRate(probeId, TickInterval);
  IF ok = 0 THEN
    Console.WriteShortString("SetRedrawRate failed"); Console.WriteLn;
    RETURN
  END;

  REPEAT
    ok := iGui.NextEvent(kind, childId, timeMs, p1, p2, p3, p4, -1);
    IF ok # 0 THEN
      IF kind = iGui.EvTick THEN
        IF childId = probeId THEN EmitTick END
      ELSIF kind = iGui.EvFrameClose THEN
        EXIT
      END
    END
  UNTIL FALSE;

  Console.WriteShortString("LogProbe done, ticks="); Console.WriteInt(tickCount); Console.WriteLn
END Run;

END LogProbe.
