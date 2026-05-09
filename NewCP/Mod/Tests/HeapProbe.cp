MODULE HeapProbe;
(* Smoke probe for dump-heap: allocates a few records via NEW so the heap
   snapshot has something to show. *)

IMPORT Console;

TYPE
  NodeDesc* = RECORD value: INTEGER END;
  Node*     = POINTER TO NodeDesc;

(* Method present so a TypeDesc is emitted and NEW routes through __newcp_new_rec. *)
PROCEDURE (n: NodeDesc) Touch*(), NEW;
BEGIN
END Touch;

PROCEDURE Run*;
  VAR i: INTEGER; n: Node;
BEGIN
  i := 0;
  WHILE i < 8 DO
    NEW(n); n.value := i; INC(i)
  END;
  Console.WriteInt(8); Console.WriteLn()
END Run;

END HeapProbe.
