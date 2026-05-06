MODULE ArrayMethods;

(* Test: calling a bound procedure on an element of an array of pointers-to-record. *)

TYPE
    Node* = EXTENSIBLE RECORD
        val*: INTEGER
    END;

VAR
    nodes: ARRAY 3 OF POINTER TO Node;

PROCEDURE (n: Node) GetVal*(): INTEGER, NEW, EXTENSIBLE;
BEGIN
    RETURN n.val
END GetVal;

PROCEDURE SetNode*(idx: INTEGER; p: POINTER TO Node);
BEGIN
    nodes[idx] := p
END SetNode;

PROCEDURE CallGetVal*(idx: INTEGER): INTEGER;
BEGIN
    RETURN nodes[idx].GetVal()
END CallGetVal;

END ArrayMethods.
