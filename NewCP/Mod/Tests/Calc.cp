MODULE Calc;
(**
   Calc — pure-value test functions exercising numeric and char operations.
   Each exported procedure takes no arguments and returns an INTEGER result:
     0  = pass (expected value produced)
     non-zero = the unexpected value that was computed (makes failures readable)

   These are the canonical result-calculation tests: no Console I/O,
   no IR-shape inspection.  Just load, call, and assert on i64.
*)

(* --- Named record type used by RecordFields test --- *)
TYPE
  Point = RECORD x, y: INTEGER END;
  (* --- Procedure types used by ProcTypeCall / ProcTypeParamCall tests --- *)
  NullaryIntProc  = PROCEDURE(): INTEGER;
  BinaryIntProc   = PROCEDURE(a, b: INTEGER): INTEGER;
  (* --- Record type used by ArrayOfRecord test --- *)
  Pair = RECORD a, b: INTEGER END;

(* --- Module-level variable (tested by GlobVarTest) --- *)
VAR
  globalX: INTEGER;

(* --- Arithmetic --- *)

PROCEDURE Add*(): INTEGER;
BEGIN RETURN 3 + 4 END Add;           (* expected 7 *)

PROCEDURE Sub*(): INTEGER;
BEGIN RETURN 10 - 3 END Sub;          (* expected 7 *)

PROCEDURE Mul*(): INTEGER;
BEGIN RETURN 6 * 7 END Mul;           (* expected 42 *)

PROCEDURE DivPos*(): INTEGER;
BEGIN RETURN 17 DIV 5 END DivPos;     (* expected 3 *)

PROCEDURE ModPos*(): INTEGER;
BEGIN RETURN 17 MOD 5 END ModPos;     (* expected 2 *)

PROCEDURE NegArith*(): INTEGER;
BEGIN RETURN -(3 + 4) END NegArith;   (* expected -7 *)

(* --- Boolean / comparison --- *)

PROCEDURE CmpTrue*(): INTEGER;
  VAR r: INTEGER;
BEGIN
  IF 3 < 5 THEN r := 1 ELSE r := 0 END;
  RETURN r
END CmpTrue;                           (* expected 1 *)

PROCEDURE CmpFalse*(): INTEGER;
  VAR r: INTEGER;
BEGIN
  IF 5 < 3 THEN r := 1 ELSE r := 0 END;
  RETURN r
END CmpFalse;                          (* expected 0 *)

(* --- CHAR (32-bit Unicode scalar) --- *)

PROCEDURE CharOrd*(): INTEGER;
BEGIN RETURN ORD('A') END CharOrd;    (* expected 65 *)

PROCEDURE CharHex*(): INTEGER;
BEGIN RETURN ORD(41X) END CharHex;    (* 41H = 65 decimal *)

PROCEDURE CharChr*(): INTEGER;
BEGIN RETURN ORD(CHR(90)) END CharChr; (* expected 90 *)

(* --- SHORTCHAR (8-bit byte) --- *)

PROCEDURE ShortCharOrd*(): INTEGER;
  VAR c: SHORTCHAR;
BEGIN
  c := 61X;   (* 'a' *)
  RETURN ORD(c)
END ShortCharOrd;                      (* expected 97 *)

PROCEDURE ShortCharLit*(): INTEGER;
  VAR c: SHORTCHAR;
BEGIN
  c := SHORT(CHR(42));   (* '*' = 42 *)
  RETURN ORD(c)
END ShortCharLit;                      (* expected 42 *)

(* --- SHORTCHAR array / string --- *)

(* Return length of a null-terminated SHORTCHAR array. *)
PROCEDURE StrLen(s: ARRAY OF SHORTCHAR): INTEGER;
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE s[i] # 0X DO INC(i) END;
  RETURN i
END StrLen;

PROCEDURE LiteralLen*(): INTEGER;
BEGIN RETURN StrLen("hello") END LiteralLen;  (* expected 5 *)

PROCEDURE ArrayCopy*(): INTEGER;
  VAR buf: ARRAY 32 OF SHORTCHAR;
      i: INTEGER;
BEGIN
  i := 0;
  buf[0] := 72X;  (* 'H' *)
  buf[1] := 69X;  (* 'i' *)
  buf[2] := 0X;
  RETURN StrLen(buf)   (* expected 2 *)
END ArrayCopy;

(* --- SET --- *)

PROCEDURE SetIn*(): INTEGER;
  VAR s: SET; r: INTEGER;
BEGIN
  s := {3, 5, 7};
  IF 5 IN s THEN r := 1 ELSE r := 0 END;
  RETURN r
END SetIn;                             (* expected 1 *)

PROCEDURE SetNotIn*(): INTEGER;
  VAR s: SET; r: INTEGER;
BEGIN
  s := {3, 5, 7};
  IF 4 IN s THEN r := 1 ELSE r := 0 END;
  RETURN r
END SetNotIn;                          (* expected 0 *)

(* --- Loops --- *)

PROCEDURE SumTo10*(): INTEGER;
  VAR i, s: INTEGER;
BEGIN
  s := 0; i := 1;
  WHILE i <= 10 DO s := s + i; INC(i) END;
  RETURN s
END SumTo10;                           (* expected 55 *)

PROCEDURE Factorial5*(): INTEGER;
  VAR i, f: INTEGER;
BEGIN
  f := 1; i := 1;
  REPEAT f := f * i; INC(i) UNTIL i > 5;
  RETURN f
END Factorial5;                        (* expected 120 *)

(* --- CASE --- *)

PROCEDURE CaseSides*(n: INTEGER): INTEGER;
  VAR r: INTEGER;
BEGIN
  CASE n OF
    1: r := 0   (* Circle *)
  | 2: r := 3   (* Triangle *)
  | 3: r := 4   (* Square *)
  ELSE r := -1
  END;
  RETURN r
END CaseSides;

PROCEDURE CaseCircle*(): INTEGER;
BEGIN RETURN CaseSides(1) END CaseCircle;     (* expected 0 *)
PROCEDURE CaseTriangle*(): INTEGER;
BEGIN RETURN CaseSides(2) END CaseTriangle;   (* expected 3 *)
PROCEDURE CaseElse*(): INTEGER;
BEGIN RETURN CaseSides(99) END CaseElse;      (* expected -1 *)

(* --- Floor DIV/MOD (negative operands — CP uses floor division, not truncation) --- *)
(* Spec: x DIV y = ENTIER(x/y);  0 <= (x MOD y) < y  when y > 0           *)
(* Note: write (-5) DIV 3, not -5 DIV 3 — the latter means -(5 DIV 3)     *)

PROCEDURE DivNeg*(): INTEGER;
  VAR x: INTEGER;
BEGIN x := -5; RETURN x DIV 3 END DivNeg;   (* -2, floor(-5/3) *)

PROCEDURE ModNeg*(): INTEGER;
  VAR x: INTEGER;
BEGIN x := -5; RETURN x MOD 3 END ModNeg;   (* 1, satisfies 0 <= r < 3 *)

PROCEDURE DivNegY*(): INTEGER;
  VAR y: INTEGER;
BEGIN y := -3; RETURN 5 DIV y END DivNegY;  (* -2, floor(5/-3) *)

PROCEDURE ModNegY*(): INTEGER;
  VAR y: INTEGER;
BEGIN y := -3; RETURN 5 MOD y END ModNegY;  (* -1, satisfies r <= 0 *)

PROCEDURE DivBothNeg*(): INTEGER;
  VAR x, y: INTEGER;
BEGIN x := -5; y := -3; RETURN x DIV y END DivBothNeg; (* 1, floor(-5/-3) *)

(* --- SET binary operators (union +, difference -, intersection *, range {a..b}) --- *)

PROCEDURE SetUnion*(): INTEGER;
  VAR s: SET; r: INTEGER;
BEGIN
  s := {1, 2} + {3, 4};
  IF 3 IN s THEN r := 1 ELSE r := 0 END;
  RETURN r
END SetUnion;                               (* expected 1 *)

PROCEDURE SetIntersect*(): INTEGER;
  VAR s: SET; r: INTEGER;
BEGIN
  s := {1, 2, 3} * {2, 3, 4};
  IF (2 IN s) & ~(1 IN s) THEN r := 1 ELSE r := 0 END;
  RETURN r
END SetIntersect;                           (* expected 1 *)

PROCEDURE SetDiff*(): INTEGER;
  VAR s: SET; r: INTEGER;
BEGIN
  s := {1, 2, 3} - {2, 3, 4};
  IF (1 IN s) & ~(2 IN s) THEN r := 1 ELSE r := 0 END;
  RETURN r
END SetDiff;                                (* expected 1 *)

PROCEDURE SetSymDiff*(): INTEGER;
  VAR s: SET; r: INTEGER;
BEGIN
  s := {1, 2, 3} / {2, 3, 4};
  IF (1 IN s) & (4 IN s) & ~(2 IN s) THEN r := 1 ELSE r := 0 END;
  RETURN r
END SetSymDiff;                             (* expected 1 *)

PROCEDURE SetRange*(): INTEGER;
  VAR s: SET; r: INTEGER;
BEGIN
  s := {3..7};
  IF (5 IN s) & ~(2 IN s) & ~(8 IN s) THEN r := 1 ELSE r := 0 END;
  RETURN r
END SetRange;                               (* expected 1 *)

(* --- ABS, ODD, ASH --- *)

PROCEDURE AbsPos*(): INTEGER;
BEGIN RETURN ABS(7) END AbsPos;             (* expected 7 *)

PROCEDURE AbsNeg*(): INTEGER;
BEGIN RETURN ABS(-7) END AbsNeg;            (* expected 7 *)

PROCEDURE OddTrue*(): INTEGER;
  VAR r: INTEGER;
BEGIN
  IF ODD(3) THEN r := 1 ELSE r := 0 END;
  RETURN r
END OddTrue;                                (* expected 1 *)

PROCEDURE OddFalse*(): INTEGER;
  VAR r: INTEGER;
BEGIN
  IF ODD(4) THEN r := 1 ELSE r := 0 END;
  RETURN r
END OddFalse;                               (* expected 0 *)

PROCEDURE AshLeft*(): INTEGER;
BEGIN RETURN ASH(1, 4) END AshLeft;         (* expected 16 *)

PROCEDURE AshRight*(): INTEGER;
BEGIN RETURN ASH(16, -2) END AshRight;      (* expected 4 *)

(* --- FOR loop --- *)

PROCEDURE ForSum*(): INTEGER;
  VAR i, s: INTEGER;
BEGIN
  s := 0;
  FOR i := 1 TO 5 DO s := s + i END;
  RETURN s
END ForSum;                                 (* expected 15 *)

PROCEDURE ForBy2*(): INTEGER;
  VAR i, s: INTEGER;
BEGIN
  s := 0;
  FOR i := 0 TO 10 BY 2 DO s := s + i END;
  RETURN s
END ForBy2;                                 (* expected 30 *)

PROCEDURE ForDown*(): INTEGER;
  VAR i, s: INTEGER;
BEGIN
  s := 0;
  FOR i := 5 TO 1 BY -1 DO s := s + i END;
  RETURN s
END ForDown;                                (* expected 15 *)

(* --- LOOP / EXIT --- *)

PROCEDURE LoopExit*(): INTEGER;
  VAR i: INTEGER;
BEGIN
  i := 0;
  LOOP
    INC(i);
    IF i >= 5 THEN EXIT END
  END;
  RETURN i
END LoopExit;                               (* expected 5 *)

(* --- Two-argument MAX / MIN --- *)

PROCEDURE MaxOfTwo*(): INTEGER;
BEGIN RETURN MAX(3, 7) END MaxOfTwo;        (* expected 7 *)

PROCEDURE MinOfTwo*(): INTEGER;
BEGIN RETURN MIN(3, 7) END MinOfTwo;        (* expected 3 *)

(* --- INC / DEC with step argument --- *)

PROCEDURE IncStep*(): INTEGER;
  VAR x: INTEGER;
BEGIN x := 3; INC(x, 4); RETURN x END IncStep;       (* expected 7 *)

PROCEDURE DecOne*(): INTEGER;
  VAR x: INTEGER;
BEGIN x := 8; DEC(x); RETURN x END DecOne;            (* expected 7 *)

PROCEDURE DecStep*(): INTEGER;
  VAR x: INTEGER;
BEGIN x := 10; DEC(x, 3); RETURN x END DecStep;       (* expected 7 *)

(* --- INCL / EXCL --- *)

PROCEDURE InclExcl*(): INTEGER;
  VAR s: SET; r: INTEGER;
BEGIN
  s := {};
  INCL(s, 5);
  EXCL(s, 5);
  INCL(s, 3);
  IF (3 IN s) & ~(5 IN s) THEN r := 1 ELSE r := 0 END;
  RETURN r
END InclExcl;                                          (* expected 1 *)

(* --- Monadic SET complement  (-s = all bits NOT in s) --- *)

PROCEDURE SetComplement*(): INTEGER;
  VAR s, t: SET; r: INTEGER;
BEGIN
  s := {0, 1, 2};
  t := -s;
  IF ~(0 IN t) & (3 IN t) THEN r := 1 ELSE r := 0 END;
  RETURN r
END SetComplement;                                     (* expected 1 *)

(* --- ELSIF chain --- *)

PROCEDURE ElsifChain*(): INTEGER;
  VAR x, r: INTEGER;
BEGIN
  x := 5;
  IF x < 0 THEN r := -1
  ELSIF x = 0 THEN r := 0
  ELSIF x < 10 THEN r := 1
  ELSE r := 2
  END;
  RETURN r
END ElsifChain;                                        (* expected 1 *)

(* --- CASE with range labels (a..b) --- *)

PROCEDURE CaseRange*(): INTEGER;
  VAR x, r: INTEGER;
BEGIN
  x := 7;
  CASE x OF
    1..3: r := 1
  | 4..6: r := 2
  | 7..9: r := 3
  ELSE r := 0
  END;
  RETURN r
END CaseRange;                                         (* expected 3 *)

(* --- BOOLEAN as an assignable value --- *)

PROCEDURE BoolVal*(): INTEGER;
  VAR b: BOOLEAN; r: INTEGER;
BEGIN
  b := 3 > 2;
  IF b THEN r := 1 ELSE r := 0 END;
  RETURN r
END BoolVal;                                           (* expected 1 *)

(* --- Nested WHILE (double loop, 3x3 iterations) --- *)

PROCEDURE DoubleLoop*(): INTEGER;
  VAR i, j, s: INTEGER;
BEGIN
  s := 0; i := 1;
  WHILE i <= 3 DO
    j := 1;
    WHILE j <= 3 DO
      INC(s);
      INC(j)
    END;
    INC(i)
  END;
  RETURN s
END DoubleLoop;                                        (* expected 9 *)

(* --- Early RETURN from inside a loop --- *)

PROCEDURE EarlyReturn*(): INTEGER;
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE i < 100 DO
    IF i = 5 THEN RETURN i END;
    INC(i)
  END;
  RETURN -1
END EarlyReturn;                                       (* expected 5 *)

(* --- Recursive call (factorial) --- *)

PROCEDURE RecFact(n: INTEGER): INTEGER;
BEGIN
  IF n <= 1 THEN RETURN 1 END;
  RETURN n * RecFact(n - 1)
END RecFact;

PROCEDURE RecFactorial5*(): INTEGER;
BEGIN RETURN RecFact(5) END RecFactorial5;             (* expected 120 *)

(* --- REPEAT / UNTIL with DEC --- *)

PROCEDURE RepeatDown*(): INTEGER;
  VAR i: INTEGER;
BEGIN
  i := 10;
  REPEAT DEC(i) UNTIL i <= 5;
  RETURN i
END RepeatDown;                                        (* expected 5 *)

(* --- Local CONST declaration --- *)

PROCEDURE LocalConst*(): INTEGER;
  CONST N = 6;
  VAR r: INTEGER;
BEGIN r := N * N; RETURN r END LocalConst;             (* expected 36 *)

(* --- LEN of a fixed-size array --- *)

PROCEDURE LenFixed*(): INTEGER;
  VAR a: ARRAY 10 OF INTEGER;
BEGIN RETURN LEN(a) END LenFixed;                      (* expected 10 *)

(* --- ENTIER: floor of a real number → INTEGER (LONGINT in spec) --- *)

PROCEDURE EntierFloor*(): LONGINT;
BEGIN RETURN ENTIER(3.7) END EntierFloor;              (* expected 3 *)

PROCEDURE EntierNeg*(): LONGINT;
BEGIN RETURN ENTIER(-1.2) END EntierNeg;               (* expected -2 *)

PROCEDURE RealAdd*(): LONGINT;
BEGIN RETURN ENTIER(1.5 + 1.5) END RealAdd;            (* expected 3 *)

(* --- CAP: capitalize a Latin-1 character --- *)

PROCEDURE CapLower*(): INTEGER;
BEGIN RETURN ORD(CAP('a')) END CapLower;               (* expected 65 = ORD('A') *)

(* --- OR logical operator --- *)

PROCEDURE OrTrue*(): INTEGER;
  VAR r: INTEGER;
BEGIN
  IF ODD(3) OR ODD(4) THEN r := 1 ELSE r := 0 END;
  RETURN r
END OrTrue;                                            (* expected 1: TRUE OR FALSE *)

PROCEDURE OrFalse*(): INTEGER;
  VAR r: INTEGER;
BEGIN
  IF ODD(4) OR ODD(6) THEN r := 1 ELSE r := 0 END;
  RETURN r
END OrFalse;                                           (* expected 0: FALSE OR FALSE *)

(* --- Real division / (§8.2.2) --- *)

PROCEDURE RealDiv*(): LONGINT;
BEGIN RETURN ENTIER(7.0 / 2.0) END RealDiv;            (* expected 3 *)

(* --- Hex integer literal: H suffix = 32-bit constant (§3) --- *)

PROCEDURE HexLit*(): INTEGER;
BEGIN RETURN 0FFH END HexLit;                          (* expected 255 *)

(* --- SHORT / LONG roundtrip (§10.3) --- *)

PROCEDURE ShortLong*(): INTEGER;
  VAR x: INTSHORT;
BEGIN x := SHORT(1000); RETURN LONG(x) END ShortLong;  (* expected 1000 *)

(* --- ENTIER of SHORTREAL (§10.3) --- *)

PROCEDURE ShortRealFloor*(): LONGINT;
BEGIN RETURN ENTIER(SHORT(3.7)) END ShortRealFloor;    (* expected 3 *)

(* --- BITS: integer bitfield → SET (§10.3) --- *)

PROCEDURE BitsTest*(): INTEGER;
  VAR s: SET; r: INTEGER;
BEGIN
  s := BITS(5);              (* 5 = 101b → {0, 2} *)
  IF (0 IN s) & ~(1 IN s) & (2 IN s) THEN r := 1 ELSE r := 0 END;
  RETURN r
END BitsTest;                                          (* expected 1 *)

(* --- ORD of SET: sum of 2^i for each member i (§10.3) --- *)

PROCEDURE OrdSet*(): INTEGER;
BEGIN RETURN ORD({0, 2}) END OrdSet;                   (* 2^0+2^2 = 5 *)

(* --- CASE with CHAR expression and range labels (§9.5) --- *)

PROCEDURE CaseChar*(): INTEGER;
  VAR ch: CHAR; r: INTEGER;
BEGIN
  ch := 'M';
  CASE ch OF
    'A'..'Z': r := 1
  | 'a'..'z': r := 2
  ELSE r := 0
  END;
  RETURN r
END CaseChar;                                          (* expected 1 *)

(* --- CASE with comma-separated label list in one arm (§9.5) --- *)

PROCEDURE CaseMultiLabel*(): INTEGER;
  VAR x, r: INTEGER;
BEGIN
  x := 3;
  CASE x OF
    1, 3, 5: r := 1
  | 2, 4, 6: r := 2
  ELSE r := 0
  END;
  RETURN r
END CaseMultiLabel;                                    (* expected 1 *)

(* --- Record field read/write (§6.3) --- *)

PROCEDURE RecordFields*(): INTEGER;
  VAR p: Point;
BEGIN
  p.x := 3;
  p.y := 4;
  RETURN p.x + p.y
END RecordFields;                                      (* expected 7 *)

(* --- 2-dimensional array indexing (§6.2) --- *)

PROCEDURE Array2D*(): INTEGER;
  VAR a: ARRAY 3 OF ARRAY 3 OF INTEGER;
BEGIN
  a[1][2] := 7;
  RETURN a[1][2]
END Array2D;                                           (* expected 7 *)

(* --- Abbreviated ARRAY 3, 3 syntax + multi-index selector a[i,j] (§6.2) --- *)

PROCEDURE Array2DComma*(): INTEGER;
  VAR a: ARRAY 3, 3 OF INTEGER;
BEGIN
  a[1, 2] := 42;
  RETURN a[1, 2]
END Array2DComma;                                          (* expected 42 *)

(* --- Comparison operators: #  >=  <=  (§8.2.5) --- *)

PROCEDURE CmpNeq*(): INTEGER;
  VAR r: INTEGER;
BEGIN
  IF 3 # 5 THEN r := 1 ELSE r := 0 END;
  RETURN r
END CmpNeq;                                                (* expected 1 *)

PROCEDURE CmpGeq*(): INTEGER;
  VAR r: INTEGER;
BEGIN
  IF 5 >= 5 THEN r := 1 ELSE r := 0 END;
  RETURN r
END CmpGeq;                                                (* expected 1 *)

PROCEDURE CmpLeq*(): INTEGER;
  VAR r: INTEGER;
BEGIN
  IF 3 <= 5 THEN r := 1 ELSE r := 0 END;
  RETURN r
END CmpLeq;                                                (* expected 1 *)

(* --- Boolean NOT of a variable (§8.2.1) --- *)

PROCEDURE BoolNot*(): INTEGER;
  VAR b: BOOLEAN; r: INTEGER;
BEGIN
  b := ~(3 > 5);     (* ~FALSE = TRUE *)
  IF b THEN r := 1 ELSE r := 0 END;
  RETURN r
END BoolNot;                                               (* expected 1 *)

(* --- Module-level global variable read/write (§7, §11) --- *)

PROCEDURE GlobVarTest*(): INTEGER;
BEGIN
  globalX := 99;
  RETURN globalX
END GlobVarTest;                                           (* expected 99 *)

(* --- L-suffix integer literal → LONGINT (§3) --- *)

PROCEDURE LLit*(): LONGINT;
BEGIN RETURN 0FFFF0000L END LLit;  (* 0FFFF0000hex = 4294901760, fits in LONGINT *)

(* --- VAR parameter (pass by reference, §10.1) --- *)

PROCEDURE Increment(VAR x: INTEGER);
BEGIN INC(x) END Increment;

PROCEDURE VarParamTest*(): INTEGER;
  VAR n: INTEGER;
BEGIN n := 14; Increment(n); RETURN n END VarParamTest;   (* expected 15 *)

(* --- LOOP with two EXIT points (§9.9, §9.10) --- *)

PROCEDURE LoopMultiExit*(): INTEGER;
  VAR i: INTEGER;
BEGIN
  i := 0;
  LOOP
    INC(i);
    IF i = 3 THEN EXIT END;
    IF i = 10 THEN EXIT END
  END;
  RETURN i
END LoopMultiExit;                                         (* expected 3 *)

(* --- Nested local procedure (§10: procedure declarations may be nested) --- *)

PROCEDURE NestedProcTest*(): INTEGER;
  PROCEDURE Double(x: INTEGER): INTEGER;
  BEGIN RETURN x * 2 END Double;
BEGIN RETURN Double(21) END NestedProcTest;                (* expected 42 *)

(* --- CHAR comparison (§8.2.5 relations on character types) --- *)

PROCEDURE CharCmp*(): INTEGER;
  VAR r: INTEGER;
BEGIN
  IF 'b' > 'a' THEN r := 1 ELSE r := 0 END;
  RETURN r
END CharCmp;                                               (* expected 1 *)

(* --- SHORTINT arithmetic via SHORT / LONG (§6.1, §10.3) --- *)

PROCEDURE ShortIntArith*(): INTEGER;
  VAR x: SHORTINT;
BEGIN x := SHORT(100); RETURN LONG(x) * 2 END ShortIntArith; (* expected 200 *)

(* --- IN parameter: read-only array parameter (§10.1) --- *)

PROCEDURE SumArray(IN a: ARRAY OF INTEGER; n: INTEGER): INTEGER;
  VAR i, s: INTEGER;
BEGIN
  s := 0; i := 0;
  WHILE i < n DO s := s + a[i]; INC(i) END;
  RETURN s
END SumArray;

PROCEDURE InParamTest*(): INTEGER;
  VAR a: ARRAY 4 OF INTEGER;
BEGIN
  a[0] := 1; a[1] := 2; a[2] := 3; a[3] := 4;
  RETURN SumArray(a, 4)
END InParamTest;                                           (* expected 10 *)

(* --- Procedure type: nullary proc stored in a variable (§10.1) --- *)

PROCEDURE ReturnSeven(): INTEGER;
BEGIN RETURN 7 END ReturnSeven;

PROCEDURE ProcTypeCall*(): INTEGER;
  VAR fn: NullaryIntProc;
BEGIN
  fn := ReturnSeven;
  RETURN fn()
END ProcTypeCall;                                          (* expected 7 *)

(* --- Procedure type: proc with parameters stored and called --- *)

PROCEDURE SumTwo(a, b: INTEGER): INTEGER;
BEGIN RETURN a + b END SumTwo;

PROCEDURE ProcTypeParamCall*(): INTEGER;
  VAR fn: BinaryIntProc;
BEGIN
  fn := SumTwo;
  RETURN fn(10, 32)
END ProcTypeParamCall;                                     (* expected 42 *)

(* --- Array of RECORD: index into an array of records --- *)

PROCEDURE ArrayOfRecord*(): INTEGER;
  VAR pairs: ARRAY 4 OF Pair;
BEGIN
  pairs[2].a := 3;
  pairs[2].b := 4;
  RETURN pairs[2].a + pairs[2].b
END ArrayOfRecord;                                         (* expected 7 *)

(* --- REAL as a procedure parameter and return value --- *)

PROCEDURE AddReal(x, y: REAL): REAL;
BEGIN RETURN x + y END AddReal;

PROCEDURE RealParam*(): LONGINT;
BEGIN RETURN ENTIER(AddReal(1.5, 2.5)) END RealParam;      (* expected 4 *)

END Calc.
