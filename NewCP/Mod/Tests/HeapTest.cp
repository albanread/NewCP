MODULE HeapTest;
(*  Heap exerciser for `dump-heap`.

    Allocates many records across several distinct types and sizes so that
    the heap snapshot's per-cluster occupancy and per-type catalog have
    something interesting to report:

      - Tiny    8-byte payload, 200 instances
      - Small  24-byte payload, 100 instances
      - Mid    72-byte payload,  40 instances
      - Big   256-byte payload,  10 instances

    Every type carries a method so `newcp-llvm` emits a `TypeDesc` for it
    and `NEW` routes through `__newcp_new_rec`. None of the allocations are
    rooted in module globals, so they are all unreachable garbage as soon
    as `Run` returns — the snapshot is taken before any explicit collect,
    so they all show up as live.
*)

IMPORT Console;

TYPE
    TinyDesc*  = RECORD a: INTEGER END;
    Tiny*      = POINTER TO TinyDesc;

    SmallDesc* = RECORD a, b, c, d, e, f: INTEGER END;
    Small*     = POINTER TO SmallDesc;

    MidDesc*   = RECORD
        data: ARRAY 16 OF INTEGER;
        meta: INTEGER
    END;
    Mid*       = POINTER TO MidDesc;

    BigDesc*   = RECORD
        bytes: ARRAY 60 OF INTEGER;
        tag:   INTEGER
    END;
    Big*       = POINTER TO BigDesc;

(* Methods: their presence forces a TypeDesc per record type. *)

PROCEDURE (t: TinyDesc) Touch*(), NEW;
BEGIN END Touch;

PROCEDURE (s: SmallDesc) Touch*(), NEW;
BEGIN END Touch;

PROCEDURE (m: MidDesc) Touch*(), NEW;
BEGIN END Touch;

PROCEDURE (b: BigDesc) Touch*(), NEW;
BEGIN END Touch;

(* Allocators — kept as separate procs so each has its own short-lived
   stack frame; locals do not escape and the conservative scanner cannot
   accidentally pin garbage from earlier calls. *)

PROCEDURE FillTiny(n: INTEGER);
    VAR i: INTEGER; p: Tiny;
BEGIN
    i := 0;
    WHILE i < n DO
        NEW(p); p.a := i; INC(i)
    END
END FillTiny;

PROCEDURE FillSmall(n: INTEGER);
    VAR i: INTEGER; p: Small;
BEGIN
    i := 0;
    WHILE i < n DO
        NEW(p);
        p.a := i;     p.b := i + 1; p.c := i + 2;
        p.d := i + 3; p.e := i + 4; p.f := i + 5;
        INC(i)
    END
END FillSmall;

PROCEDURE FillMid(n: INTEGER);
    VAR i, j: INTEGER; p: Mid;
BEGIN
    i := 0;
    WHILE i < n DO
        NEW(p); p.meta := i;
        j := 0;
        WHILE j < 16 DO p.data[j] := i + j; INC(j) END;
        INC(i)
    END
END FillMid;

PROCEDURE FillBig(n: INTEGER);
    VAR i, j: INTEGER; p: Big;
BEGIN
    i := 0;
    WHILE i < n DO
        NEW(p); p.tag := i;
        j := 0;
        WHILE j < 60 DO p.bytes[j] := i + j; INC(j) END;
        INC(i)
    END
END FillBig;

PROCEDURE Run*;
BEGIN
    FillTiny(200);
    FillSmall(100);
    FillMid(40);
    FillBig(10);
    Console.WriteInt(350);
    Console.WriteLn()
END Run;

END HeapTest.
