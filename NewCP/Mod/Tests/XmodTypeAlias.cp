MODULE XmodTypeAlias;
(* Blocker 5 repro: a typedef in another module should be transparent
   when passed where its underlying form is expected.

   XmodTypeAliasBase.cp declares
       Name = ARRAY 16 OF CHAR
   and a procedure that takes ARRAY OF CHAR.

   This consumer passes a value of XmodTypeAliasBase.Name to that
   procedure. Sema currently rejects it as
       "expected ARRAY OF CHAR, found imported:XmodTypeAliasBase.Name"
*)

IMPORT XmodTypeAliasBase;

(* Local proc that takes open arrays — exact shape of HostFiles.CopyName. *)
PROCEDURE LocalTake (IN a, b: ARRAY OF CHAR): INTEGER;
BEGIN RETURN 0 END LocalTake;

PROCEDURE PassThroughLocal* (): INTEGER;
    VAR n, m: XmodTypeAliasBase.Name; r: INTEGER;
BEGIN
    (* Pass cross-module typedef'd fixed array to *local* open-array proc.
       This is the exact shape that fails in HostFiles.CopyName(sl.path, f.path). *)
    r := LocalTake(n, m);
    RETURN r
END PassThroughLocal;

PROCEDURE PassThroughImported* (): INTEGER;
    VAR n, m: XmodTypeAliasBase.Name; r: INTEGER;
BEGIN
    (* Same but the open-array proc lives in the SAME module as Name. *)
    XmodTypeAliasBase.TwoStrings(n, m);
    r := XmodTypeAliasBase.LengthOf(n);
    RETURN r
END PassThroughImported;

END XmodTypeAlias.
