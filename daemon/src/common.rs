// UnTrickyStore - common utilities
// Paths, logging, i18n, shell helpers
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::Write as FmtWrite;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::Command;

// ── Path constants ──────────────────────────────────────────
pub const ADB: &str = "/data/adb";
pub const MODULES: &str = "/data/adb/modules";
pub const SERVICE_D: &str = "/data/adb/service.d";

pub const TS: &str = "tricky_store";
pub const UTS: &str = "untrickystore";

pub fn ts_dir() -> String { format!("{MODULES}/{TS}") }
pub fn uts_dir() -> String { format!("{MODULES}/{UTS}") }
pub fn uts_cfg() -> String { format!("{ADB}/{UTS}") }
pub fn ts_cfg() -> String { format!("{ADB}/{TS}") }
pub fn uts_bin() -> String { format!("{}/{UTS}/bin", MODULES) }
pub fn log_path() -> String { format!("{ADB}/{UTS}/log/log.log") }

// ── Locale ──────────────────────────────────────────────────
pub fn is_chinese() -> bool {
    let locale = getprop("persist.sys.locale");
    let product = getprop("ro.product.locale");
    locale.contains("zh") || product.contains("zh")
}

/// Print bilingual message: (chinese, english)
pub fn bi(cn: &str, en: &str) {
    if is_chinese() { println!("{cn}"); } else { println!("{en}"); }
}

/// Return bilingual string
pub fn bi_str(cn: &str, en: &str) -> &str {
    if is_chinese() { cn } else { en }
}

// ── Logging ─────────────────────────────────────────────────
pub fn log_init() {
    let dir = format!("{ADB}/{UTS}/log");
    let _ = fs::create_dir_all(&dir);
}

fn log_raw(level: char, tag: &str, msg: &str) {
    let path = log_path();
    let ts = timestamp();
    let pid = unsafe { libc::getpid() };
    let line = format!("{ts}  {pid}  {pid} {level} System.out: [UTS]<{tag}>{msg}\n");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}

pub fn log_i(tag: &str, msg: &str) { log_raw('I', tag, msg); }
pub fn log_w(tag: &str, msg: &str) { log_raw('W', tag, msg); }
pub fn log_e(tag: &str, msg: &str) { log_raw('E', tag, msg); }

fn timestamp() -> String {
    unsafe {
        let mut ts: libc::timespec = std::mem::zeroed();
        let mut tm: libc::tm = std::mem::zeroed();
        libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts);
        libc::localtime_r(&ts.tv_sec, &mut tm);
        format!(
            "{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
            tm.tm_mon + 1, tm.tm_mday, tm.tm_hour, tm.tm_min, tm.tm_sec,
            ts.tv_nsec / 1_000_000
        )
    }
}

// ── Shell helpers ───────────────────────────────────────────
pub fn getprop(name: &str) -> String {
    Command::new("getprop").arg(name)
        .output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

pub fn resetprop(name: &str, value: &str) -> bool {
    Command::new("resetprop").arg(name).arg(value)
        .status().map(|s| s.success()).unwrap_or(false)
}

pub fn resetprop_n(name: &str, value: &str) -> bool {
    Command::new("resetprop").arg("-n").arg(name).arg(value)
        .status().map(|s| s.success()).unwrap_or(false)
}

pub fn pm_list_packages_3() -> Vec<String> {
    Command::new("pm").args(["list", "packages", "-3"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter_map(|l| l.strip_prefix("package:").map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

pub fn pm_path(pkg: &str) -> bool {
    Command::new("pm").args(["path", pkg])
        .output().map(|o| o.status.success()).unwrap_or(false)
}

pub fn pm_uninstall(pkg: &str) -> bool {
    Command::new("pm").args(["uninstall", pkg])
        .output().map(|o| o.status.success()).unwrap_or(false)
}

pub fn pm_install(apk: &str) -> bool {
    Command::new("pm").args(["install", apk])
        .output().map(|o| o.status.success()).unwrap_or(false)
}

pub fn pidof(name: &str) -> String {
    Command::new("pidof").arg(name)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

pub fn kill_9(name: &str) {
    let pids = pidof(name);
    for pid in pids.split_whitespace() {
        let _ = Command::new("kill").args(["-9", pid]).status();
    }
}

/// Fetch URL content via curl or busybox wget
pub fn crawl(url: &str) -> String {
    // Try curl first
    if let Ok(o) = Command::new("curl")
        .args(["--connect-timeout", "10", "-Ls", url])
        .output()
    {
        if o.status.success() {
            return String::from_utf8_lossy(&o.stdout).to_string();
        }
    }
    // Fallback: try busybox wget
    for prefix in &["/data/adb/ap/bin", "/data/adb/ksu/bin", "/data/adb/magisk"] {
        let bb = format!("{prefix}/busybox");
        if std::path::Path::new(&bb).exists() {
            if let Ok(o) = Command::new(&bb)
                .args(["wget", "-T", "10", "--no-check-certificate", "-qO-", url])
                .output()
            {
                if o.status.success() {
                    return String::from_utf8_lossy(&o.stdout).to_string();
                }
            }
        }
    }
    String::new()
}

/// Read file to string, empty on error
pub fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Read file lines, skip empty
pub fn read_lines(path: &str) -> Vec<String> {
    read_file(path).lines().filter(|l| !l.trim().is_empty()).map(String::from).collect()
}

/// Write string to file (create dirs if needed)
pub fn write_file(path: &str, content: &str) -> bool {
    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, content).is_ok()
}

/// Append line to file
pub fn append_line(path: &str, line: &str) {
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{line}");
    }
}

/// Check if path exists
pub fn exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}
