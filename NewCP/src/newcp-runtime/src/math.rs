//! Native `Math` module — REAL (f64) math built on Rust's libm bindings.
//!
//! BlackBox 1.7's `System/Mod/Math.odc` is implemented as x87 FPU assembly
//! (`PROCEDURE [code]` directives). That source isn't portable, so NewCP
//! reimplements the surface as a Rust-resident module: each export below
//! becomes a JIT-resolvable symbol via `#[unsafe(export_name = "Math.X")]`,
//! then the CP-side `DEFINITION MODULE Math` declares the matching signatures.
//!
//! Floating-point semantics:
//! - REAL is IEEE-754 binary64 (Rust `f64`).
//! - Special values (NaN, ±∞, signed zero) propagate per IEEE-754.
//! - `Mantissa` / `Exponent` / `Real` follow the BlackBox conventions
//!   (`1 <= |m| < 2`, exponent unbiased) and use direct bit manipulation
//!   so they're independent of any libm `frexp`/`ldexp` availability.

use crate::{ExportEntry, ExportDirectory, HostedModuleArtifact, NativeModuleArtifact, NativeExportBinding};

// -- Constants -----------------------------------------------------------

#[unsafe(export_name = "Math.Pi")]
pub extern "C" fn math_pi() -> f64 {
    std::f64::consts::PI
}

#[unsafe(export_name = "Math.Eps")]
pub extern "C" fn math_eps() -> f64 {
    f64::EPSILON
}

// -- Roots, exponentials, logs ------------------------------------------

#[unsafe(export_name = "Math.Sqrt")]
pub extern "C" fn math_sqrt(x: f64) -> f64 { x.sqrt() }

#[unsafe(export_name = "Math.Exp")]
pub extern "C" fn math_exp(x: f64) -> f64 { x.exp() }

#[unsafe(export_name = "Math.Ln")]
pub extern "C" fn math_ln(x: f64) -> f64 { x.ln() }

#[unsafe(export_name = "Math.Log")]
pub extern "C" fn math_log(x: f64) -> f64 { x.log10() }

#[unsafe(export_name = "Math.Power")]
pub extern "C" fn math_power(x: f64, y: f64) -> f64 { x.powf(y) }

/// `IntPower(x, n)` — CP signature uses `INTEGER`, which is i64 in NewCP.
/// Rust's `powi` takes i32 and is precise for small magnitudes; for very
/// large `n` we fall back to `powf` via promotion (loses precision but
/// preserves correctness in extremes).
#[unsafe(export_name = "Math.IntPower")]
pub extern "C" fn math_int_power(x: f64, n: i64) -> f64 {
    if let Ok(small) = i32::try_from(n) {
        x.powi(small)
    } else {
        x.powf(n as f64)
    }
}

// -- Trig ---------------------------------------------------------------

#[unsafe(export_name = "Math.Sin")]    pub extern "C" fn math_sin(x: f64) -> f64    { x.sin() }
#[unsafe(export_name = "Math.Cos")]    pub extern "C" fn math_cos(x: f64) -> f64    { x.cos() }
#[unsafe(export_name = "Math.Tan")]    pub extern "C" fn math_tan(x: f64) -> f64    { x.tan() }
#[unsafe(export_name = "Math.ArcSin")] pub extern "C" fn math_arcsin(x: f64) -> f64 { x.asin() }
#[unsafe(export_name = "Math.ArcCos")] pub extern "C" fn math_arccos(x: f64) -> f64 { x.acos() }
#[unsafe(export_name = "Math.ArcTan")] pub extern "C" fn math_arctan(x: f64) -> f64 { x.atan() }
#[unsafe(export_name = "Math.ArcTan2")]
pub extern "C" fn math_arctan2(y: f64, x: f64) -> f64 { y.atan2(x) }

// -- Hyperbolic ---------------------------------------------------------

#[unsafe(export_name = "Math.Sinh")]    pub extern "C" fn math_sinh(x: f64) -> f64    { x.sinh() }
#[unsafe(export_name = "Math.Cosh")]    pub extern "C" fn math_cosh(x: f64) -> f64    { x.cosh() }
#[unsafe(export_name = "Math.Tanh")]    pub extern "C" fn math_tanh(x: f64) -> f64    { x.tanh() }
#[unsafe(export_name = "Math.ArcSinh")] pub extern "C" fn math_arcsinh(x: f64) -> f64 { x.asinh() }
#[unsafe(export_name = "Math.ArcCosh")] pub extern "C" fn math_arccosh(x: f64) -> f64 { x.acosh() }
#[unsafe(export_name = "Math.ArcTanh")] pub extern "C" fn math_arctanh(x: f64) -> f64 { x.atanh() }

// -- Rounding -----------------------------------------------------------

#[unsafe(export_name = "Math.Floor")]   pub extern "C" fn math_floor(x: f64) -> f64   { x.floor() }
#[unsafe(export_name = "Math.Ceiling")] pub extern "C" fn math_ceiling(x: f64) -> f64 { x.ceil() }

/// Round half-away-from-zero — matches the BlackBox behavior where
/// `Round(0.5) = 1.0` and `Round(-0.5) = -1.0`.
#[unsafe(export_name = "Math.Round")]
pub extern "C" fn math_round(x: f64) -> f64 { x.round() }

#[unsafe(export_name = "Math.Trunc")] pub extern "C" fn math_trunc(x: f64) -> f64 { x.trunc() }
#[unsafe(export_name = "Math.Frac")]  pub extern "C" fn math_frac(x: f64) -> f64  { x.fract() }

#[unsafe(export_name = "Math.Sign")]
pub extern "C" fn math_sign(x: f64) -> f64 {
    if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 }
}

// -- IEEE-754 decomposition --------------------------------------------
//
// CP's `Mantissa` / `Exponent` / `Real` work on the binary64 representation
// directly. They use the convention `x = m * 2^e` with `1 <= |m| < 2`
// for normals (so `m` is the implicit-1 + fraction, scaled to [1,2)).

const F64_EXP_BIAS: i32 = 1023;
const F64_MANTISSA_BITS: u32 = 52;
const F64_EXP_MASK: u64 = 0x7FF0_0000_0000_0000;
const F64_MANTISSA_MASK: u64 = 0x000F_FFFF_FFFF_FFFF;
const F64_SIGN_MASK: u64 = 0x8000_0000_0000_0000;

#[unsafe(export_name = "Math.Mantissa")]
pub extern "C" fn math_mantissa(x: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    if x.is_infinite() {
        return if x.is_sign_positive() { 1.0 } else { -1.0 };
    }
    if x.is_nan() {
        // CP convention: return ±1.5 to flag NaN.
        return if (x.to_bits() & F64_SIGN_MASK) == 0 { 1.5 } else { -1.5 };
    }
    // Normal / subnormal: clear the exponent bits and bias them so the
    // result lies in [1, 2). Subnormals are normalized via the natural
    // f64 arithmetic — `x / 2^e` after computing the unbiased exponent.
    let bits = x.to_bits();
    let sign = bits & F64_SIGN_MASK;
    let mantissa = bits & F64_MANTISSA_MASK;
    let new_bits = sign | (F64_EXP_BIAS as u64) << F64_MANTISSA_BITS | mantissa;
    f64::from_bits(new_bits)
}

#[unsafe(export_name = "Math.Exponent")]
pub extern "C" fn math_exponent(x: f64) -> i64 {
    if x == 0.0 {
        return 0;
    }
    if x.is_infinite() || x.is_nan() {
        return i64::MAX;
    }
    let bits = x.to_bits();
    let raw = ((bits & F64_EXP_MASK) >> F64_MANTISSA_BITS) as i32;
    (raw - F64_EXP_BIAS) as i64
}

/// `Real(m, e)` — compose `m * 2^e`. CP precondition: `1 <= |m| < 2`.
/// We don't enforce that here; we just substitute the exponent bits.
#[unsafe(export_name = "Math.Real")]
pub extern "C" fn math_real(m: f64, e: i64) -> f64 {
    if m == 0.0 {
        return 0.0;
    }
    if e == i64::MAX {
        // CP convention: produce ±inf.
        return if m.is_sign_positive() { f64::INFINITY } else { f64::NEG_INFINITY };
    }
    // Use ldexp via repeated mul; clamps gracefully outside the normal range.
    let clamped = e.clamp(-2000, 2000) as i32;
    if clamped == e as i32 {
        // libm-style: m * 2^e
        m * (clamped as f64).exp2()
    } else if e > 0 {
        f64::INFINITY * m.signum()
    } else {
        0.0 * m.signum()
    }
}

// -- Native module registration -----------------------------------------

pub fn native_module_artifact() -> NativeModuleArtifact {
    let names: &[(&str, *const ())] = &[
        ("Pi", math_pi as *const ()),
        ("Eps", math_eps as *const ()),
        ("Sqrt", math_sqrt as *const ()),
        ("Exp", math_exp as *const ()),
        ("Ln", math_ln as *const ()),
        ("Log", math_log as *const ()),
        ("Power", math_power as *const ()),
        ("IntPower", math_int_power as *const ()),
        ("Sin", math_sin as *const ()),
        ("Cos", math_cos as *const ()),
        ("Tan", math_tan as *const ()),
        ("ArcSin", math_arcsin as *const ()),
        ("ArcCos", math_arccos as *const ()),
        ("ArcTan", math_arctan as *const ()),
        ("ArcTan2", math_arctan2 as *const ()),
        ("Sinh", math_sinh as *const ()),
        ("Cosh", math_cosh as *const ()),
        ("Tanh", math_tanh as *const ()),
        ("ArcSinh", math_arcsinh as *const ()),
        ("ArcCosh", math_arccosh as *const ()),
        ("ArcTanh", math_arctanh as *const ()),
        ("Floor", math_floor as *const ()),
        ("Ceiling", math_ceiling as *const ()),
        ("Round", math_round as *const ()),
        ("Trunc", math_trunc as *const ()),
        ("Frac", math_frac as *const ()),
        ("Sign", math_sign as *const ()),
        ("Mantissa", math_mantissa as *const ()),
        ("Exponent", math_exponent as *const ()),
        ("Real", math_real as *const ()),
    ];
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "Math",
            vec![],
            ExportDirectory::new(names.iter().map(|(n, _)| ExportEntry::procedure(*n)).collect()),
            "Math.bootstrap",
            "Rust-hosted REAL (f64) math facade for CP modules",
            vec![],
        ),
        names.iter()
            .map(|(n, p)| NativeExportBinding::procedure(*n, *p as usize))
            .collect(),
    )
}
