// UnTrickyStore - property spoofing
//
// propstate: spoof bootloader unlock state properties
// vbhash:    fix VBMeta hash via ContentProvider app
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common;

/// Spoof bootloader-related properties to appear locked/verified.
pub fn propstate() -> Result<(), String> {
    common::log_i("propstate", "start");

    // Get vbmeta partition size for property
    let vbmeta_size = {
        let slot = common::getprop("ro.boot.slot_suffix");
        let dev = format!("/dev/block/by-name/vbmeta{slot}");
        let out = std::process::Command::new("busybox")
            .args(["blockdev", "--getbsz", &dev])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        if out.is_empty() { "4096".to_string() } else { out }
    };

    // Clear ADB disabled flag
    common::resetprop("sys.usb.adb.disabled", " ");

    // --- Properties: check_missing_match (set if missing OR wrong) ---
    let cmm = |name: &str, expected: &str| {
        let val = common::getprop(name);
        if val.is_empty() || val != expected {
            common::resetprop(name, expected);
        }
        if val.is_empty() {
            common::resetprop(name, expected);
        }
    };

    cmm("ro.boot.vbmeta.device_state", "locked");
    cmm("ro.boot.verifiedbootstate", "green");
    cmm("ro.boot.veritymode", "enforcing");
    cmm("ro.boot.warranty_bit", "0");
    cmm("ro.boot.flash.locked", "1");

    // --- Properties: contains_reset (reset if value contains substring) ---
    let cr = |name: &str, contains: &str, newval: &str| {
        let val = common::getprop(name);
        if val.contains(contains) {
            common::resetprop(name, newval);
        }
    };

    cr("vendor.boot.bootmode", "recovery", "unknown");
    cr("ro.boot.bootmode", "recovery", "unknown");
    cr("ro.bootmode", "recovery", "unknown");

    // --- Properties: check_missing (set only if missing) ---
    let cm = |name: &str, expected: &str| {
        if common::getprop(name).is_empty() {
            common::resetprop(name, expected);
        }
    };

    cm("ro.boot.vbmeta.invalidate_on_error", "yes");
    cm("ro.boot.vbmeta.size", &vbmeta_size);
    cm("ro.boot.vbmeta.hash_alg", "sha256");
    cm("ro.boot.vbmeta.avb_version", "1.2");

    // --- Properties: check_reset (force set if non-empty and wrong) ---
    let crs = |name: &str, expected: &str| {
        let val = common::getprop(name);
        if !val.is_empty() && val != expected {
            common::resetprop(name, expected);
        }
    };

    crs("vendor.boot.vbmeta.device_state", "locked");
    crs("vendor.boot.verifiedbootstate", "green");
    crs("ro.secureboot.lockstate", "locked");
    crs("ro.boot.realmebootstate", "green");
    crs("ro.vendor.boot.warranty_bit", "0");
    crs("sys.oem_unlock_allowed", "0");
    crs("ro.boot.realme.lockstate", "1");
    crs("ro.build.tags", "release-keys");
    crs("ro.crypto.state", "encrypted");
    crs("ro.vendor.warranty_bit", "0");
    crs("ro.force.debuggable", "0");
    crs("ro.build.type", "user");
    crs("ro.warranty_bit", "0");
    crs("ro.debuggable", "0");
    crs("ro.kernel.qemu", "");
    crs("ro.adb.secure", "1");
    crs("ro.secure", "1");

    common::log_i("propstate", "done");
    Ok(())
}

/// Fix VBMeta digest hash by querying the service app's ContentProvider.
pub fn vbhash() -> Result<(), String> {
    common::log_i("vbhash", "start");

    let cfg = common::uts_cfg();
    let uts_dir = common::uts_dir();
    let ts_dir = common::ts_dir();
    let apk = format!("{uts_dir}/service.apk");
    let cache_file = format!("{cfg}/verifiedboothash.txt");

    let now = common::getprop("ro.boot.vbmeta.digest");

    // Check if TrickyStore is new enough (versionCode >= 245) to accept external hash
    let ts_module_prop = format!("{ts_dir}/module.prop");
    let ts_ver = if common::exists(&ts_module_prop) {
        common::read_file(&ts_module_prop)
            .lines()
            .find(|l| l.starts_with("versionCode="))
            .and_then(|l| l.strip_prefix("versionCode="))
            .and_then(|v| v.trim().parse::<u32>().ok())
            .unwrap_or(0)
    } else {
        0
    };

    let is_ts245 = ts_ver >= 245;

    // Try to use cached hash first
    if is_ts245 {
        if let Some(persisted) = read_cached_hash(&cache_file) {
            if persisted == now {
                common::log_i("vbhash", &format!("cached hash matches current: {now}"));
                return Ok(());
            }
            common::resetprop_n("ro.boot.vbmeta.digest", &persisted);
            common::log_i("vbhash", &format!("restored cached hash: {persisted}"));
            return Ok(());
        }
    }

    // Need to query ContentProvider
    let hash = query_content_provider(&apk)?;

    if hash.is_empty() {
        return Err("vbhash: empty hash from ContentProvider".into());
    }

    if now == hash {
        common::log_i("vbhash", &format!("no change needed, current: {now}"));
    } else {
        common::resetprop_n("ro.boot.vbmeta.digest", &hash);
        common::log_i("vbhash", &format!("set digest to {hash}"));
    }

    // Cache for persistence (TS >= 245)
    if is_ts245 {
        common::write_file(&cache_file, &hash);
    }

    Ok(())
}

fn read_cached_hash(path: &str) -> Option<String> {
    let content = common::read_file(path);
    let hash = content.lines().next().unwrap_or("").trim().to_string();
    if hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(hash)
    } else {
        None
    }
}

/// Install the service APK, query content://Provider for VBHash, uninstall.
fn query_content_provider(apk: &str) -> Result<String, String> {
    let provider_pkg = "io.github.frblanapps.untrickystore";
    let provider_uri = "content://Provider";

    // Install
    common::log_i("vbhash", "installing service app");
    if !common::pm_install(apk) {
        return Err("vbhash: failed to install service.apk".into());
    }

    // Query via content call
    common::log_i("vbhash", "querying ContentProvider");
    let output = std::process::Command::new("content")
        .args(["call", "--uri", provider_uri, "--method", "GET"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    // Also grab logcat output for debugging
    let _ = std::process::Command::new("logcat")
        .args(["-d"])
        .output()
        .map(|o| {
            let text = String::from_utf8_lossy(&o.stdout);
            for line in text.lines() {
                if line.contains("[UTS]") {
                    common::log_i("vbhash", line);
                }
            }
        });

    // Parse hash from output: look for 64-char hex followed by =VBHash
    let hash = extract_vbhash(&output);

    // Uninstall
    common::log_i("vbhash", "uninstalling service app");
    common::pm_uninstall(provider_pkg);

    if let Some(h) = hash {
        Ok(h)
    } else {
        // Fallback: generate random hash for boot signature
        common::log_w("vbhash", "content provider returned no valid hash, generating random");
        let random = generate_random_hex(64);
        Ok(random)
    }
}

fn extract_vbhash(output: &str) -> Option<String> {
    // Pattern: 64 hex chars followed by =VBHash
    for segment in output.split_whitespace() {
        if segment.ends_with("=VBHash") {
            let hex = segment.trim_end_matches("=VBHash");
            if hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(hex.to_string());
            }
        }
    }
    // Try: just find a 64-char hex string
    for word in output.split_whitespace() {
        let cleaned: String = word.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        if cleaned.len() == 64 {
            return Some(cleaned);
        }
    }
    None
}

fn generate_random_hex(len: usize) -> String {
    let mut buf = vec![0u8; len / 2];
    let fd = unsafe { libc::open(b"/dev/urandom\0".as_ptr() as *const _, libc::O_RDONLY) };
    if fd >= 0 {
        unsafe {
            libc::read(fd, buf.as_mut_ptr() as *mut _, buf.len());
            libc::close(fd);
        }
    }
    buf.iter().map(|b| format!("{b:02x}")).collect()
}
