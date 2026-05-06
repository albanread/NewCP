MODULE StrArrays;
(**
   StrArrays — exercises fixed-size SHORTCHAR array passing.

   Verifies that:
   1. A literal string is correctly copied into a fixed-size local array.
   2. A fixed-size array is correctly passed by pointer to a procedure
      that takes ARRAY OF SHORTCHAR (open array).
   3. A fixed-size array global is passed to an imported procedure.
   4. Arrays of different sizes can all be passed to the same open-array param.

   Run* prints four lines to the console; the test harness checks them.
*)

IMPORT Console;

CONST
  BufSize = 64;

VAR
  greeting: ARRAY BufSize OF SHORTCHAR;

(* Copy src into dst up to dstLen-1 characters. *)
PROCEDURE CopyStr(VAR dst: ARRAY OF SHORTCHAR; dstLen: INTEGER; src: ARRAY OF SHORTCHAR);
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (src[i] # 0X) & (i < dstLen - 1) DO
    dst[i] := src[i]; INC(i)
  END;
  dst[i] := 0X
END CopyStr;

(* Print an open-array string followed by a newline. *)
PROCEDURE PrintLn(s: ARRAY OF SHORTCHAR);
BEGIN
  Console.WriteShortString(s);
  Console.WriteLn
END PrintLn;

PROCEDURE Run*;
  VAR
    local32: ARRAY 32 OF SHORTCHAR;
    local8:  ARRAY 8  OF SHORTCHAR;
BEGIN
  (* 1. pass a string literal to a procedure taking ARRAY OF SHORTCHAR *)
  PrintLn("hello from literal");

  (* 2. copy a literal into a local fixed array then pass that array *)
  CopyStr(local32, 32, "fixed array copy");
  PrintLn(local32);

  (* 3. copy into a small array (truncation test — fits exactly) *)
  CopyStr(local8, 8, "seven!");   (* 6 chars + NUL fits in 8 *)
  PrintLn(local8);

  (* 4. use the module-level global array *)
  CopyStr(greeting, BufSize, "global array");
  PrintLn(greeting)
END Run;

END StrArrays.
