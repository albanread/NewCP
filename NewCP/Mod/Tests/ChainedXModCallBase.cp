MODULE ChainedXModCallBase;
(* Cross-module receiver for deferred_fixes #33 — Inner is
   the type that gets RETURNED by a method on Outer; the
   trailing `.Total()` is the call whose receiver type
   originates in this module. *)

TYPE
    InnerDesc* = RECORD value*: INTEGER END;
    Inner* = POINTER TO InnerDesc;

    OuterDesc* = RECORD slot*: Inner END;
    Outer* = POINTER TO OuterDesc;


PROCEDURE (i: InnerDesc) Total* (): INTEGER, NEW;
BEGIN
    RETURN i.value * 7
END Total;

PROCEDURE (o: OuterDesc) Pick* (): Inner, NEW;
BEGIN
    RETURN o.slot
END Pick;

END ChainedXModCallBase.
