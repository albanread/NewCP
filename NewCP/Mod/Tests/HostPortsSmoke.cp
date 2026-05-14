MODULE HostPortsSmoke;
(* Smoke test for the HostPorts → HostPortsSys → iGui dispatch
   chain.

   What this probe DOESN'T do: open an actual iGui window.
   `HostPorts.NewPort` calls iGui.OpenChild which requires a
   running iGui frame.  Spinning one up isn't safe inside the
   unit-test process (no message-loop, no shutdown hook).

   What it DOES do: allocate a HostPort manually (bypassing the
   NewPort factory), get a HostRider from it, and call DrawRect /
   DrawLine / DrawString.  Those forward through HostPortsSys to
   iGui's `EmitFillRect` / `EmitDrawLine` / `EmitDrawTextRun`,
   which push commands into the iGui batch queue.  No window
   means the batch is never submitted, but the dispatch all the
   way to iGui's batch push DID execute — the probe verifies via
   a successful return (no traps, no asserts) that all five
   abstract Rider methods we care about light up.

   Logs from iGui's [igui-export] eprintln lines show the actual
   forwarded arguments — useful when staring at the test
   manually but not checked here. *)

IMPORT Ports, HostPorts, HostPortsSys;

PROCEDURE Run* (): INTEGER;
    VAR p: HostPorts.HostPort;
        rd: Ports.Rider;
        result: INTEGER;
        unpackR, unpackG, unpackB, unpackA: REAL;
BEGIN
    result := 0;

    (* Hand-build a HostPort without going through iGui.OpenChild. *)
    NEW(p);
    p.Init(1, FALSE);
    p.SetSize(800, 600);
    IF (p.unit = 1) & (p.widthDip = 800) THEN INC(result, 1) END;

    (* Allocate a Rider — should be a HostRider. *)
    rd := p.NewRider();
    IF rd # NIL THEN INC(result, 2) END;
    IF rd.Base() = p THEN INC(result, 4) END;

    (* Color unpacking sanity: red = 0x000000FF should give
       (1.0, 0.0, 0.0, 1.0).  Tests the byte-channel arithmetic
       at the Sys boundary. *)
    HostPortsSys.UnpackColor(Ports.red, unpackR, unpackG, unpackB, unpackA);
    IF (unpackR > 0.99) & (unpackR < 1.01)
     & (unpackG > -0.01) & (unpackG < 0.01)
     & (unpackB > -0.01) & (unpackB < 0.01) THEN
        INC(result, 8)
    END;

    (* Drive the rider's paint methods.  Each call forwards to
       iGui and pushes into the batch queue.  Success = no trap. *)
    rd.DrawRect(10, 20, 110, 80, Ports.fill, Ports.red);
    INC(result, 16);     (* DrawRect returned *)

    rd.DrawLine(0, 0, 100, 100, 1, Ports.black);
    INC(result, 32);     (* DrawLine returned *)

    rd.DrawString(50, 100, Ports.black, "Hi!", NIL);
    INC(result, 64);     (* DrawString returned *)

    RETURN result   (* expect 127 = 1+2+4+8+16+32+64 *)
END Run;

END HostPortsSmoke.
