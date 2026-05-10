MODULE StringArrayCompare;
(* Direct repro for the ARRAY OF CHAR vs string-literal compare
   codegen.  Each procedure isolates one comparison shape so a
   regression points at exactly one path. *)

(* Equal: a char array filled to match the literal exactly. *)
PROCEDURE ArrayEqualsLiteral* (): INTEGER;
    VAR a: ARRAY 32 OF CHAR;
BEGIN
    a[0] := "F"; a[1] := "o"; a[2] := "o"; a[3] := 0X;
    IF a = "Foo" THEN RETURN 1 ELSE RETURN 0 END
END ArrayEqualsLiteral;

(* Not-equal: prefix differs at index 0. *)
PROCEDURE ArrayDiffersFromLiteral* (): INTEGER;
    VAR a: ARRAY 32 OF CHAR;
BEGIN
    a[0] := "B"; a[1] := "a"; a[2] := "r"; a[3] := 0X;
    IF a # "Foo" THEN RETURN 1 ELSE RETURN 0 END
END ArrayDiffersFromLiteral;

(* Equal-prefix-but-array-shorter: array terminates at 3, literal
   is "Foobar".  Must compare unequal — the array's terminator
   beats the literal's continuation. *)
PROCEDURE ArrayShorterThanLiteral* (): INTEGER;
    VAR a: ARRAY 32 OF CHAR;
BEGIN
    a[0] := "F"; a[1] := "o"; a[2] := "o"; a[3] := 0X;
    IF a # "Foobar" THEN RETURN 1 ELSE RETURN 0 END
END ArrayShorterThanLiteral;

(* Equal-prefix-but-array-longer: array has "Foobar", literal "Foo".
   Same outcome — terminators don't line up. *)
PROCEDURE ArrayLongerThanLiteral* (): INTEGER;
    VAR a: ARRAY 32 OF CHAR;
BEGIN
    a[0] := "F"; a[1] := "o"; a[2] := "o";
    a[3] := "b"; a[4] := "a"; a[5] := "r"; a[6] := 0X;
    IF a # "Foo" THEN RETURN 1 ELSE RETURN 0 END
END ArrayLongerThanLiteral;

(* Reversed operand order: literal on the left, array on the right. *)
PROCEDURE LiteralEqualsArray* (): INTEGER;
    VAR a: ARRAY 16 OF CHAR;
BEGIN
    a[0] := "x"; a[1] := "y"; a[2] := 0X;
    IF "xy" = a THEN RETURN 1 ELSE RETURN 0 END
END LiteralEqualsArray;

(* Two array variables compared.  Different storage addresses,
   matching contents — must compare equal. *)
PROCEDURE TwoArraysEqual* (): INTEGER;
    VAR a, b: ARRAY 16 OF CHAR;
BEGIN
    a[0] := "h"; a[1] := "i"; a[2] := 0X;
    b[0] := "h"; b[1] := "i"; b[2] := 0X;
    IF a = b THEN RETURN 1 ELSE RETURN 0 END
END TwoArraysEqual;

PROCEDURE TwoArraysDiffer* (): INTEGER;
    VAR a, b: ARRAY 16 OF CHAR;
BEGIN
    a[0] := "h"; a[1] := "i"; a[2] := 0X;
    b[0] := "h"; b[1] := "o"; b[2] := 0X;
    IF a # b THEN RETURN 1 ELSE RETURN 0 END
END TwoArraysDiffer;

END StringArrayCompare.
