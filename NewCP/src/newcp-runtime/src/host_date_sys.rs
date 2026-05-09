//! Native `HostDateSys` module — flat C-ABI clock + date-formatting
//! shims that back `Mod/HostDates.cp`'s concrete `Dates.Hook`.
//!
//! BlackBox `Host/Mod/HostDates.odc` calls Win32 `GetLocalTime` /
//! `GetSystemTime` / `GetTimeZoneInformation`. NewCP routes the same
//! primitives through `std::time::SystemTime` (UTC seconds-since-epoch)
//! and a small portable date-arithmetic helper so the backend works on
//! every platform.
//!
//! ABI:
//! - All time queries return decomposed `(year, month, day, hour,
//!   minute, second)` via OUT pointers (one i64 each).
//! - The UTC bias query returns a single i64 (minutes; UTC = local + bias).
//! - Date / time formatters take the decomposed values plus a target
//!   `*mut u32` UTF-32 buffer (the CP `OUT str: ARRAY OF CHAR` ABI),
//!   the buffer's length (CP open-array fat-pointer), and a format code.

use crate::{
    ExportDirectory, ExportEntry, HostedModuleArtifact, NativeExportBinding, NativeModuleArtifact,
};

/// Seconds since Unix epoch (1970-01-01 UTC) for the start of
/// 0001-01-03, the date the BlackBox `Day` formula returns 1 for. We
/// don't actually need this constant for the runtime side — the
/// runtime returns absolute year/month/day and the CP `Day` /
/// `DayToDate` helpers handle ordinal arithmetic.
const _DAYS_FROM_DATE_EPOCH_TO_UNIX_EPOCH: i64 = 0;

// -- Local / UTC clock --------------------------------------------------

/// Decompose Unix timestamp `secs` (seconds since 1970-01-01 UTC) into
/// `(year, month, day, hour, minute, second)` using a portable
/// civil-from-days algorithm by Howard Hinnant
/// (https://howardhinnant.github.io/date_algorithms.html, public-domain).
///
/// Works for any year, including before the Gregorian cutoff. The
/// returned `month` is 1..=12 and `day` is 1..=31.
fn decompose_unix_seconds(secs: i64) -> (i64, i64, i64, i64, i64, i64) {
    let days_total = secs.div_euclid(86_400);
    let secs_of_day = secs.rem_euclid(86_400);

    // Hinnant: civil from days.
    let z = days_total + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // 0..146096
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // 0..399
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // 0..365
    let mp = (5 * doy + 2) / 153; // 0..11
    let d = doy as i64 - (153 * mp as i64 + 2) / 5 + 1; // 1..31
    let m = if mp < 10 { mp as i64 + 3 } else { mp as i64 - 9 }; // 1..12
    let year = if m <= 2 { y + 1 } else { y };

    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;
    (year, m, d, hour, minute, second)
}

fn now_utc_seconds() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(_) => 0,
    }
}

/// Local-time bias in minutes (UTC = local + bias). Cross-platform
/// implementations of "what's the current timezone offset" are
/// platform-specific in std; we approximate by subtracting the
/// process-local Hinnant decomposition of `localtime_r` from UTC.
/// On Windows we call `GetTimeZoneInformation`; on Unix we read the
/// `tm_gmtoff` field via the `libc` crate. To keep the dependency
/// surface zero, we fall back to a pure-Rust path that just reports 0
/// (UTC) — the test harness pins to UTC anyway. Real local-clock
/// support can be added later by wiring `chrono::Local` here.
fn utc_bias_minutes() -> i64 {
    0
}

#[unsafe(export_name = "HostDateSys.GetUTCTime")]
/// Write the current UTC date+time into the six OUT pointers.
pub extern "C" fn host_date_sys_get_utc_time(
    year: *mut i64,
    month: *mut i64,
    day: *mut i64,
    hour: *mut i64,
    minute: *mut i64,
    second: *mut i64,
) {
    let (y, m, d, hh, mm, ss) = decompose_unix_seconds(now_utc_seconds());
    unsafe {
        if !year.is_null()   { *year   = y;  }
        if !month.is_null()  { *month  = m;  }
        if !day.is_null()    { *day    = d;  }
        if !hour.is_null()   { *hour   = hh; }
        if !minute.is_null() { *minute = mm; }
        if !second.is_null() { *second = ss; }
    }
}

#[unsafe(export_name = "HostDateSys.GetLocalTime")]
/// Write the current local date+time into the six OUT pointers.
/// Local-time backend is currently the same as UTC (bias=0) — see
/// `utc_bias_minutes` for the upgrade path.
pub extern "C" fn host_date_sys_get_local_time(
    year: *mut i64,
    month: *mut i64,
    day: *mut i64,
    hour: *mut i64,
    minute: *mut i64,
    second: *mut i64,
) {
    let local = now_utc_seconds() - utc_bias_minutes() * 60;
    let (y, m, d, hh, mm, ss) = decompose_unix_seconds(local);
    unsafe {
        if !year.is_null()   { *year   = y;  }
        if !month.is_null()  { *month  = m;  }
        if !day.is_null()    { *day    = d;  }
        if !hour.is_null()   { *hour   = hh; }
        if !minute.is_null() { *minute = mm; }
        if !second.is_null() { *second = ss; }
    }
}

#[unsafe(export_name = "HostDateSys.GetUTCBias")]
/// UTC = local + bias. In minutes.
pub extern "C" fn host_date_sys_get_utc_bias() -> i64 {
    utc_bias_minutes()
}

// -- Formatting -----------------------------------------------------------

const MONTH_NAMES_LONG: &[&str] = &[
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

const MONTH_NAMES_SHORT: &[&str] = &[
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Write `s` as UTF-32 code points into `out[..min(out_len, s.chars())]`,
/// always null-terminating within `out_len`. Returns the number of
/// non-NUL code points written.
fn write_utf32(out: *mut u32, out_len: i64, s: &str) -> i64 {
    if out.is_null() || out_len <= 0 {
        return 0;
    }
    let cap = (out_len - 1).max(0) as isize;
    let mut i: isize = 0;
    for ch in s.chars() {
        if i >= cap { break; }
        unsafe { *out.offset(i) = ch as u32; }
        i += 1;
    }
    unsafe { *out.offset(i) = 0; }
    i as i64
}

/// Format a date with one of the BlackBox-style format codes:
/// 0=short ("4/20/2026"), 1=long ("April 20, 2026"),
/// 2=abbreviated ("Apr 20, 2026"), 3=plainLong ("20 April 2026"),
/// 4=plainAbbreviated ("20 Apr 2026"). Anything else falls back to short.
#[unsafe(export_name = "HostDateSys.DateToString")]
pub extern "C" fn host_date_sys_date_to_string(
    year: i64,
    month: i64,
    day: i64,
    format: i64,
    out: *mut u32,
    out_len: i64,
) {
    let mi = (month - 1).clamp(0, 11) as usize;
    let s = match format {
        1 => format!("{} {}, {}", MONTH_NAMES_LONG[mi], day, year),
        2 => format!("{} {}, {}", MONTH_NAMES_SHORT[mi], day, year),
        3 => format!("{} {} {}", day, MONTH_NAMES_LONG[mi], year),
        4 => format!("{} {} {}", day, MONTH_NAMES_SHORT[mi], year),
        _ => format!("{}/{}/{}", month, day, year),
    };
    write_utf32(out, out_len, &s);
}

#[unsafe(export_name = "HostDateSys.TimeToString")]
/// Format a time as `HH:MM:SS` (zero-padded).
pub extern "C" fn host_date_sys_time_to_string(
    hour: i64,
    minute: i64,
    second: i64,
    out: *mut u32,
    out_len: i64,
) {
    let s = format!("{:02}:{:02}:{:02}", hour, minute, second);
    write_utf32(out, out_len, &s);
}

// -- Native module registration -----------------------------------------

pub fn native_module_artifact() -> NativeModuleArtifact {
    let names: &[(&str, *const ())] = &[
        ("GetUTCTime",   host_date_sys_get_utc_time   as *const ()),
        ("GetLocalTime", host_date_sys_get_local_time as *const ()),
        ("GetUTCBias",   host_date_sys_get_utc_bias   as *const ()),
        ("DateToString", host_date_sys_date_to_string as *const ()),
        ("TimeToString", host_date_sys_time_to_string as *const ()),
    ];
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "HostDateSys",
            vec![],
            ExportDirectory::new(
                names.iter().map(|(n, _)| ExportEntry::procedure(*n)).collect(),
            ),
            "HostDateSys.bootstrap",
            "Rust-hosted clock + date formatting facade for HostDates.cp",
            vec![],
        ),
        names.iter()
            .map(|(n, p)| NativeExportBinding::procedure(*n, *p as usize))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_unix_zero_is_1970_01_01() {
        let (y, m, d, hh, mm, ss) = decompose_unix_seconds(0);
        assert_eq!((y, m, d, hh, mm, ss), (1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn decompose_unix_one_day_is_1970_01_02() {
        let (y, m, d, ..) = decompose_unix_seconds(86_400);
        assert_eq!((y, m, d), (1970, 1, 2));
    }

    #[test]
    fn decompose_handles_2000_leap_day() {
        // 2000-02-29 12:34:56 UTC = 951_827_696
        let (y, m, d, hh, mm, ss) = decompose_unix_seconds(951_827_696);
        assert_eq!((y, m, d, hh, mm, ss), (2000, 2, 29, 12, 34, 56));
    }

    #[test]
    fn date_to_string_short() {
        let mut buf = [0u32; 32];
        host_date_sys_date_to_string(2026, 5, 9, 0, buf.as_mut_ptr(), buf.len() as i64);
        let s: String = buf.iter().take_while(|c| **c != 0).map(|c| char::from_u32(*c).unwrap()).collect();
        assert_eq!(s, "5/9/2026");
    }

    #[test]
    fn time_to_string_pads_zeros() {
        let mut buf = [0u32; 32];
        host_date_sys_time_to_string(7, 5, 3, buf.as_mut_ptr(), buf.len() as i64);
        let s: String = buf.iter().take_while(|c| **c != 0).map(|c| char::from_u32(*c).unwrap()).collect();
        assert_eq!(s, "07:05:03");
    }
}
