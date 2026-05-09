MODULE InParamWrite;
(* Sema must reject any write through an IN parameter — CP §10.1.1
   says IN parameters are read-only. The conservative rule covers
   direct re-assignment, field writes, and indexed-element writes. *)

TYPE
    Box = RECORD value*: INTEGER END;

(* Direct write to an IN scalar param — rejected. *)
PROCEDURE BadScalar (IN n: INTEGER);
BEGIN
    n := 7
END BadScalar;

(* Write to a field of an IN record param — rejected. *)
PROCEDURE BadField (IN b: Box);
BEGIN
    b.value := 99
END BadField;

(* Write to an indexed element of an IN array param — rejected. *)
PROCEDURE BadIndex (IN a: ARRAY OF INTEGER);
BEGIN
    a[0] := 1
END BadIndex;

END InParamWrite.
