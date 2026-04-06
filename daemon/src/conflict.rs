// UnTrickyStore - conflict detection
//
// conflict_mod: detect conflicting modules (returns error if found)
// conflict_app: detect conflicting apps (returns error if found)
//
// Boot-time only. If conflicts exist, caller should block execution.
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common;

/// Module IDs that conflict with UnTrickyStore/TrickyStore.
const CONFLICT_MODULES: &[&str] = &[
    "Yurikey",
    "xiaocaiye",
    "safetynet-fix",
    "vbmeta-fixer",
    "playintegrity",
    "integrity_box",
    "SukiSU_module",
    "Reset_BootHash",
    "PlayIntegrityFix",
    "Tricky_store-bm",
    "Hide_Bootloader",
    "ShaikoManager",
    "extreme_hide_root",
    "Tricky_Store-xiaoyi",
    "tricky_store_assistant",
    "extreme_hide_bootloader",
    "wjw_hiderootauxiliarymod",
];

/// Apps (package names) that conflict with us.
const CONFLICT_APPS: &[&str] = &[
    "com.lingqiaohan.appbl",
    "com.topmiaohuan.hidebllist",
];

/// Check for conflicting modules. Returns Err with list if any found.
pub fn conflict_mod() -> Result<(), String> {
    common::log_i("conflict-mod", "start");

    let modules_dir = common::MODULES;
    let mut found: Vec<&str> = Vec::new();

    for id in CONFLICT_MODULES {
        let dir = format!("{modules_dir}/{id}");
        // Only count as conflict if present and not already disabled
        if common::exists(&dir) && !common::exists(&format!("{dir}/disable")) {
            found.push(id);
        }
    }

    if found.is_empty() {
        common::log_i("conflict-mod", "no conflicts");
        Ok(())
    } else {
        let list = found.join(", ");
        common::log_e("conflict-mod", &format!("conflicts found: {list}"));
        Err(format!("conflicting modules: {list}"))
    }
}

/// Check for conflicting apps. Returns Err with list if any found.
pub fn conflict_app() -> Result<(), String> {
    common::log_i("conflict-app", "start");

    let installed = common::pm_list_packages_3();
    let mut found: Vec<&str> = Vec::new();

    for pkg in CONFLICT_APPS {
        if installed.iter().any(|p| p == pkg) {
            found.push(pkg);
        }
    }

    if found.is_empty() {
        common::log_i("conflict-app", "no conflicts");
        Ok(())
    } else {
        let list = found.join(", ");
        common::log_e("conflict-app", &format!("conflicts found: {list}"));
        Err(format!("conflicting apps: {list}"))
    }
}
