//! Native `SMath` module — SHORTREAL (f32) math.
//!
//! Same surface as `Math` but operates on `f32`. NewCP keeps the BlackBox
//! distinction between SHORTREAL (32-bit) and REAL (64-bit) — see
//! `Math` for the f64 counterpart.

use crate::{ExportEntry, ExportDirectory, HostedModuleArtifact, NativeModuleArtifact, NativeExportBinding};

#[unsafe(export_name = "SMath.Pi")]  pub extern "C" fn smath_pi() -> f32  { std::f32::consts::PI }
#[unsafe(export_name = "SMath.Eps")] pub extern "C" fn smath_eps() -> f32 { f32::EPSILON }

#[unsafe(export_name = "SMath.Sqrt")]  pub extern "C" fn smath_sqrt(x: f32) -> f32  { x.sqrt() }
#[unsafe(export_name = "SMath.Exp")]   pub extern "C" fn smath_exp(x: f32) -> f32   { x.exp() }
#[unsafe(export_name = "SMath.Ln")]    pub extern "C" fn smath_ln(x: f32) -> f32    { x.ln() }
#[unsafe(export_name = "SMath.Log")]   pub extern "C" fn smath_log(x: f32) -> f32   { x.log10() }
#[unsafe(export_name = "SMath.Power")] pub extern "C" fn smath_power(x: f32, y: f32) -> f32 { x.powf(y) }

#[unsafe(export_name = "SMath.IntPower")]
pub extern "C" fn smath_int_power(x: f32, n: i64) -> f32 {
    if let Ok(small) = i32::try_from(n) { x.powi(small) } else { x.powf(n as f32) }
}

#[unsafe(export_name = "SMath.Sin")]    pub extern "C" fn smath_sin(x: f32) -> f32    { x.sin() }
#[unsafe(export_name = "SMath.Cos")]    pub extern "C" fn smath_cos(x: f32) -> f32    { x.cos() }
#[unsafe(export_name = "SMath.Tan")]    pub extern "C" fn smath_tan(x: f32) -> f32    { x.tan() }
#[unsafe(export_name = "SMath.ArcSin")] pub extern "C" fn smath_arcsin(x: f32) -> f32 { x.asin() }
#[unsafe(export_name = "SMath.ArcCos")] pub extern "C" fn smath_arccos(x: f32) -> f32 { x.acos() }
#[unsafe(export_name = "SMath.ArcTan")] pub extern "C" fn smath_arctan(x: f32) -> f32 { x.atan() }
#[unsafe(export_name = "SMath.ArcTan2")]
pub extern "C" fn smath_arctan2(y: f32, x: f32) -> f32 { y.atan2(x) }

#[unsafe(export_name = "SMath.Sinh")]    pub extern "C" fn smath_sinh(x: f32) -> f32    { x.sinh() }
#[unsafe(export_name = "SMath.Cosh")]    pub extern "C" fn smath_cosh(x: f32) -> f32    { x.cosh() }
#[unsafe(export_name = "SMath.Tanh")]    pub extern "C" fn smath_tanh(x: f32) -> f32    { x.tanh() }
#[unsafe(export_name = "SMath.ArcSinh")] pub extern "C" fn smath_arcsinh(x: f32) -> f32 { x.asinh() }
#[unsafe(export_name = "SMath.ArcCosh")] pub extern "C" fn smath_arccosh(x: f32) -> f32 { x.acosh() }
#[unsafe(export_name = "SMath.ArcTanh")] pub extern "C" fn smath_arctanh(x: f32) -> f32 { x.atanh() }

#[unsafe(export_name = "SMath.Floor")]   pub extern "C" fn smath_floor(x: f32) -> f32   { x.floor() }
#[unsafe(export_name = "SMath.Ceiling")] pub extern "C" fn smath_ceiling(x: f32) -> f32 { x.ceil() }
#[unsafe(export_name = "SMath.Round")]   pub extern "C" fn smath_round(x: f32) -> f32   { x.round() }
#[unsafe(export_name = "SMath.Trunc")]   pub extern "C" fn smath_trunc(x: f32) -> f32   { x.trunc() }
#[unsafe(export_name = "SMath.Frac")]    pub extern "C" fn smath_frac(x: f32) -> f32    { x.fract() }

#[unsafe(export_name = "SMath.Sign")]
pub extern "C" fn smath_sign(x: f32) -> f32 {
    if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 }
}

const F32_EXP_BIAS: i32 = 127;
const F32_MANTISSA_BITS: u32 = 23;
const F32_EXP_MASK: u32 = 0x7F80_0000;
const F32_MANTISSA_MASK: u32 = 0x007F_FFFF;
const F32_SIGN_MASK: u32 = 0x8000_0000;

#[unsafe(export_name = "SMath.Mantissa")]
pub extern "C" fn smath_mantissa(x: f32) -> f32 {
    if x == 0.0 { return 0.0; }
    if x.is_infinite() { return if x.is_sign_positive() { 1.0 } else { -1.0 }; }
    if x.is_nan() {
        return if (x.to_bits() & F32_SIGN_MASK) == 0 { 1.5 } else { -1.5 };
    }
    let bits = x.to_bits();
    let sign = bits & F32_SIGN_MASK;
    let mantissa = bits & F32_MANTISSA_MASK;
    let new_bits = sign | (F32_EXP_BIAS as u32) << F32_MANTISSA_BITS | mantissa;
    f32::from_bits(new_bits)
}

#[unsafe(export_name = "SMath.Exponent")]
pub extern "C" fn smath_exponent(x: f32) -> i64 {
    if x == 0.0 { return 0; }
    if x.is_infinite() || x.is_nan() { return i64::MAX; }
    let bits = x.to_bits();
    let raw = ((bits & F32_EXP_MASK) >> F32_MANTISSA_BITS) as i32;
    (raw - F32_EXP_BIAS) as i64
}

#[unsafe(export_name = "SMath.Real")]
pub extern "C" fn smath_real(m: f32, e: i64) -> f32 {
    if m == 0.0 { return 0.0; }
    if e == i64::MAX {
        return if m.is_sign_positive() { f32::INFINITY } else { f32::NEG_INFINITY };
    }
    let clamped = e.clamp(-300, 300) as i32;
    if clamped == e as i32 {
        m * (clamped as f32).exp2()
    } else if e > 0 {
        f32::INFINITY * m.signum()
    } else {
        0.0 * m.signum()
    }
}

pub fn native_module_artifact() -> NativeModuleArtifact {
    let names: &[(&str, *const ())] = &[
        ("Pi", smath_pi as *const ()),
        ("Eps", smath_eps as *const ()),
        ("Sqrt", smath_sqrt as *const ()),
        ("Exp", smath_exp as *const ()),
        ("Ln", smath_ln as *const ()),
        ("Log", smath_log as *const ()),
        ("Power", smath_power as *const ()),
        ("IntPower", smath_int_power as *const ()),
        ("Sin", smath_sin as *const ()),
        ("Cos", smath_cos as *const ()),
        ("Tan", smath_tan as *const ()),
        ("ArcSin", smath_arcsin as *const ()),
        ("ArcCos", smath_arccos as *const ()),
        ("ArcTan", smath_arctan as *const ()),
        ("ArcTan2", smath_arctan2 as *const ()),
        ("Sinh", smath_sinh as *const ()),
        ("Cosh", smath_cosh as *const ()),
        ("Tanh", smath_tanh as *const ()),
        ("ArcSinh", smath_arcsinh as *const ()),
        ("ArcCosh", smath_arccosh as *const ()),
        ("ArcTanh", smath_arctanh as *const ()),
        ("Floor", smath_floor as *const ()),
        ("Ceiling", smath_ceiling as *const ()),
        ("Round", smath_round as *const ()),
        ("Trunc", smath_trunc as *const ()),
        ("Frac", smath_frac as *const ()),
        ("Sign", smath_sign as *const ()),
        ("Mantissa", smath_mantissa as *const ()),
        ("Exponent", smath_exponent as *const ()),
        ("Real", smath_real as *const ()),
    ];
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "SMath",
            vec![],
            ExportDirectory::new(names.iter().map(|(n, _)| ExportEntry::procedure(*n)).collect()),
            "SMath.bootstrap",
            "Rust-hosted SHORTREAL (f32) math facade for CP modules",
            vec![],
        ),
        names.iter()
            .map(|(n, p)| NativeExportBinding::procedure(*n, *p as usize))
            .collect(),
    )
}
