// UnTrickyStore - root environment detection
//
// Reads dmesg to detect which root solution is active.
// Writes result to $UTS_CFG/root.txt and $UTS_CFG/kernel.txt
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common;
use std::process::Command;

pub fn run() -> anyhow::Result<()> {
    let cfg = common::uts_cfg();
    let _ = std::fs::create_dir_all(&cfg);

    let dmesg = Command::new("dmesg")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let adb = common::ADB;

    // Kernel-level tags (dmesg presence)
    let ksu_ktag = dmesg.contains("KernelSU");
    let apatch_ktag = dmesg.contains("KP I commit_common_su");
    let magisk_ktag = dmesg.contains("/debug_ramdisk/magisk") || dmesg.contains("magiskinit");

    // Userspace tags (binaries exist)
    let ksu_tag = ksu_ktag
        && std::path::Path::new(&format!("{adb}/ksu")).is_dir()
        && std::path::Path::new(&format!("{adb}/ksud")).is_file();

    let suckysu_tag = ksu_ktag && ksu_tag && {
        let is_sukisu = dmesg.contains("KP hook sukisu_kpm")
            || Command::new(format!("{adb}/ksud"))
                .arg("-V")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stderr).contains("zako"))
                .unwrap_or(false);
        is_sukisu
    };

    let kernelsu_tag = ksu_tag && !suckysu_tag;

    let apatch_tag = apatch_ktag
        && std::path::Path::new(&format!("{adb}/ap")).is_dir()
        && std::path::Path::new(&format!("{adb}/apd")).is_file();

    let magisk_tag = magisk_ktag
        && std::path::Path::new(&format!("{adb}/magisk")).is_dir()
        && std::path::Path::new(&format!("{adb}/magisk.db")).is_file();

    // Determine primary root
    let active_count = [kernelsu_tag, suckysu_tag, apatch_tag, magisk_tag]
        .iter()
        .filter(|&&x| x)
        .count();

    let root = if active_count > 1 {
        // Multiple root solutions: write multiple.txt
        let mut parts = Vec::new();
        if magisk_tag { parts.push("Magisk"); }
        if kernelsu_tag { parts.push("KernelSU"); }
        if apatch_tag { parts.push("APatch"); }
        if suckysu_tag { parts.push("SuckySU"); }
        let multi = parts.join(",");
        common::write_file(&format!("{cfg}/multiple.txt"), &multi);
        "Multiple"
    } else if kernelsu_tag {
        "KernelSU"
    } else if suckysu_tag {
        "SuckySU"
    } else if apatch_tag {
        "APatch"
    } else if magisk_tag {
        "Magisk"
    } else {
        "NULL"
    };

    common::write_file(&format!("{cfg}/root.txt"), root);
    log::info!(target: "rootdetect", "root={root}");

    // Kernel-level multi-detection (modules present in kernel but not active as primary)
    let kernel_count = [ksu_ktag, apatch_ktag, magisk_ktag]
        .iter()
        .filter(|&&x| x)
        .count();

    if kernel_count > 1 && root != "Multiple" {
        let mut kparts = Vec::new();
        if magisk_ktag && root != "Magisk" { kparts.push("Magisk"); }
        if ksu_ktag && root != "KernelSU" && root != "SuckySU" { kparts.push("KernelSU"); }
        if apatch_ktag && root != "APatch" { kparts.push("APatch"); }
        if !kparts.is_empty() {
            common::write_file(&format!("{cfg}/kernel.txt"), &kparts.join(","));
        }
    } else {
        // Clean up stale file
        let _ = std::fs::remove_file(format!("{cfg}/kernel.txt"));
    }

    // Clean up dmesg dump (we don't persist it unlike TSEE)
    print!("{root}");
    Ok(())
}
