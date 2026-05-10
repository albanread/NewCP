MODULE M_Type_LinkedList_SelfReference;
    TYPE
        NodeDesc = RECORD value: INTEGER; next: Node END;
        Node     = POINTER TO NodeDesc;

    PROCEDURE Run* (): INTEGER;
        VAR head, a, b, p: Node; sum: INTEGER;
    BEGIN
        NEW(head); head.value := 10;
        NEW(a);    a.value    := 20;
        NEW(b);    b.value    := 30;
        head.next := a;
        a.next    := b;
        b.next    := NIL;
        sum := 0;
        p := head;
        WHILE p # NIL DO
            sum := sum + p.value;
            p := p.next
        END;
        RETURN sum                            (* 60 *)
    END Run;
END M_Type_LinkedList_SelfReference.
