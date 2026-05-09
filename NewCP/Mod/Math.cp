DEFINITION MODULE Math;
(**
   NewCP port of BlackBox `System/Mod/Math.odc`.

   The BlackBox original was implemented in 80387 FPU assembly via
   `PROCEDURE [code]` directives — not portable. NewCP reimplements
   the surface as a Rust-resident native module backed by libm
   (Rust's f64 methods); the CP-side DEFINITION MODULE here just
   declares the signatures for the type-checker and loader.

   REAL is IEEE-754 binary64 in NewCP, matching BlackBox.
   See `SMath` for the SHORTREAL (f32) counterpart.
*)

PROCEDURE Pi*(): REAL;
PROCEDURE Eps*(): REAL;

PROCEDURE Sqrt*(x: REAL): REAL;
PROCEDURE Exp*(x: REAL): REAL;
PROCEDURE Ln*(x: REAL): REAL;
PROCEDURE Log*(x: REAL): REAL;
PROCEDURE Power*(x, y: REAL): REAL;
PROCEDURE IntPower*(x: REAL; n: INTEGER): REAL;

PROCEDURE Sin*(x: REAL): REAL;
PROCEDURE Cos*(x: REAL): REAL;
PROCEDURE Tan*(x: REAL): REAL;
PROCEDURE ArcSin*(x: REAL): REAL;
PROCEDURE ArcCos*(x: REAL): REAL;
PROCEDURE ArcTan*(x: REAL): REAL;
PROCEDURE ArcTan2*(y, x: REAL): REAL;

PROCEDURE Sinh*(x: REAL): REAL;
PROCEDURE Cosh*(x: REAL): REAL;
PROCEDURE Tanh*(x: REAL): REAL;
PROCEDURE ArcSinh*(x: REAL): REAL;
PROCEDURE ArcCosh*(x: REAL): REAL;
PROCEDURE ArcTanh*(x: REAL): REAL;

PROCEDURE Floor*(x: REAL): REAL;
PROCEDURE Ceiling*(x: REAL): REAL;
PROCEDURE Round*(x: REAL): REAL;
PROCEDURE Trunc*(x: REAL): REAL;
PROCEDURE Frac*(x: REAL): REAL;
PROCEDURE Sign*(x: REAL): REAL;

(* IEEE-754 decomposition: x = Mantissa(x) * 2 ^ Exponent(x).
   For normals 1 <= |Mantissa(x)| < 2.
   Special-value conventions:
     x = 0:    Mantissa = 0,  Exponent = 0
     x = inf:  Mantissa = +-1, Exponent = MAX(INTEGER)
     x = nan:  Mantissa = +-1.5, Exponent = MAX(INTEGER) *)
PROCEDURE Mantissa*(x: REAL): REAL;
PROCEDURE Exponent*(x: REAL): INTEGER;
PROCEDURE Real*(m: REAL; e: INTEGER): REAL;

END Math.
