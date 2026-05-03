MODULE Vars;
(* Module-level variables of basic types; read-only export with "-". *)

VAR
    count*    : INTEGER;
    total*    : LONGINT;
    average*  : REAL;
    active*   : BOOLEAN;
    initial*  : CHAR;
    version-  : INTEGER;   (* read-only export *)

PROCEDURE Reset*;
BEGIN
    count   := 0;
    total   := 0;
    average := 0.0;
    active  := FALSE;
    initial := 0X;
    version := 1
END Reset;

BEGIN
    Reset
END Vars.
