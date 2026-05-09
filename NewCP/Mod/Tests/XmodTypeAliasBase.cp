MODULE XmodTypeAliasBase;
(* Defines a fixed-size array type alias and a consumer that takes
   open ARRAY OF CHAR. Used by XmodTypeAlias.cp to reproduce
   Blocker 5 (cross-module typedef compatibility). *)

TYPE Name* = ARRAY 16 OF CHAR;

PROCEDURE LengthOf* (IN s: ARRAY OF CHAR): INTEGER;
    VAR i: INTEGER;
BEGIN
    i := 0;
    WHILE (i < LEN(s)) & (s[i] # 0X) DO INC(i) END;
    RETURN i
END LengthOf;

(* Two ARRAY OF CHAR params — exact shape of HostFileSys.Rename. *)
PROCEDURE TwoStrings* (IN a, b: ARRAY OF CHAR);
BEGIN
END TwoStrings;

END XmodTypeAliasBase.
