// UnTrickyStore - security patch management
//
// sync:  apply persisted security patch date to system properties
// fetch: download latest security patch date from source.android.com
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common;

const PATCH_FILE: &str = "security_patch.txt";

/// Apply persisted security patch date to system properties.
///
/// Reads `security_patch.txt` from UTS config, normalizes to YYYY-MM-DD,
/// and sets `ro.vendor.build.security_patch` + `ro.build.version.security_patch`.
/// Then kills GMS unstable (com.google.android.gms.unstable) to force re-evaluation.
pub fn sync() -> Result<(), String> {
    common::log_i("patch-sync", "start");

    let cfg = common::uts_cfg();
    let path = format!("{cfg}/{PATCH_FILE}");

    if !common::exists(&path) {
        common::log_i("patch-sync", "no security_patch.txt, skip");
        return Ok(());
    }

    let raw = common::read_file(&path);
    let date = normalize_date(raw.trim());

    if date.is_empty() {
        common::log_w("patch-sync", "invalid date in security_patch.txt");
        return Err("invalid date".into());
    }

    common::resetprop("ro.vendor.build.security_patch", &date);
    common::resetprop("ro.build.version.security_patch", &date);

    // Kill GMS unstable to pick up new value
    let pids = common::pidof("com.google.android.gms.unstable");
    if !pids.is_empty() {
        common::kill_9("com.google.android.gms.unstable");
        common::log_i("patch-sync", "killed GMS unstable");
    }

    common::log_i("patch-sync", &format!("applied {date}"));
    Ok(())
}

/// Fetch latest security patch date from source.android.com and persist.
pub fn fetch() -> Result<(), String> {
    common::log_i("patch-fetch", "start");

    let url = "https://source.android.com/docs/security/bulletin/pixel";
    let cfg = common::uts_cfg();
    let path = format!("{cfg}/{PATCH_FILE}");

    let body = common::crawl(url);
    if body.is_empty() {
        return Err("patch-fetch: failed to download bulletin page".into());
    }

    // Extract date from page content
    // Look for patterns like "2025-06-05" or "2025-06-01" in the bulletin
    let date = extract_patch_date(&body);

    if date.is_empty() {
        return Err("patch-fetch: could not parse date from bulletin".into());
    }

    common::write_file(&path, &date);
    common::log_i("patch-fetch", &format!("saved {date}"));
    Ok(())
}

/// Normalize date string: accept YYYY-MM-DD or YYYYMMDD, return YYYY-MM-DD.
fn normalize_date(s: &str) -> String {
    let s = s.trim();
    if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
        // Already YYYY-MM-DD
        if s.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return s.to_string();
        }
    }
    if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
        // YYYYMMDD -> YYYY-MM-DD
        return format!("{}-{}-{}", &s[0..4], &s[4..6], &s[6..8]);
    }
    String::new()
}

/// Extract the latest security patch date from the Pixel bulletin HTML page.
/// Looks for YYYY-MM-DD patterns (day typically 01 or 05) near "patch level".
fn extract_patch_date(html: &str) -> String {
    // Simple regex-free approach: scan for YYYY-MM-DD patterns
    let mut best = String::new();

    let bytes = html.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i + 10 <= len {
        // Check for YYYY-MM-DD pattern
        if bytes[i].is_ascii_digit()
            && bytes[i + 1].is_ascii_digit()
            && bytes[i + 2].is_ascii_digit()
            && bytes[i + 3].is_ascii_digit()
            && bytes[i + 4] == b'-'
            && bytes[i + 5].is_ascii_digit()
            && bytes[i + 6].is_ascii_digit()
            && bytes[i + 7] == b'-'
            && bytes[i + 8].is_ascii_digit()
            && bytes[i + 9].is_ascii_digit()
        {
            let candidate = &html[i..i + 10];
            // Validate year (2020-2035 range)
            if let Ok(year) = candidate[0..4].parse::<u32>() {
                if (2020..=2035).contains(&year) {
                    // Pick the latest date lexicographically
                    if candidate > best.as_str() {
                        best = candidate.to_string();
                    }
                }
            }
        }
        i += 1;
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_date() {
        assert_eq!(normalize_date("2025-06-05"), "2025-06-05");
        assert_eq!(normalize_date("20250605"), "2025-06-05");
        assert_eq!(normalize_date("bad"), "");
        assert_eq!(normalize_date("  2025-01-01  "), "2025-01-01");
    }
}
