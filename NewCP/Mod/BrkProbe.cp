MODULE BrkProbe;
(* Smoke test for the BRK statement.  Calling BrkProbe.Run should
   dump heap / type / module / register / stack-walk to stderr and
   return 42.  The second BRK form takes a pointer and additionally
   dumps the heap block's TypeDesc + raw payload bytes. *)

TYPE
    NodeDesc* = RECORD
        id-: INTEGER;
        next-: POINTER TO NodeDesc
    END;
    Node* = POINTER TO NodeDesc;

PROCEDURE Run* (): INTEGER;
    VAR x: INTEGER;
        n: Node;
BEGIN
    x := 7;
    NEW(n);
    n.id := 1234;
    BRK;            (* process-wide snapshot *)
    BRK(n);         (* + typed dump of node's heap block *)
    x := x * 6;
    RETURN x
END Run;

END BrkProbe.
