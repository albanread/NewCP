MODULE M_Builtin_LEN_OnDifferentArrayKinds;
    PROCEDURE OpenLen (IN a: ARRAY OF INTEGER): INTEGER;
    BEGIN RETURN LEN(a) END OpenLen;

    PROCEDURE Run* (): INTEGER;
        VAR fixed: ARRAY 7 OF INTEGER;
    BEGIN
        (* LEN(fixed) is the static 7; OpenLen(fixed) reports the same 7
           via the open-array ABI's hidden length companion.  Combine
           into 7 * 10 + 7 + 10 = 87. *)
        RETURN LEN(fixed) * 10 + OpenLen(fixed) + 10
    END Run;
END M_Builtin_LEN_OnDifferentArrayKinds.
