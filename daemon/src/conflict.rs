// UnTrickyStore - conflict detection and removal
//
// conflict_mod: detect and tag conflicting Magisk/KSU/APatch modules
// conflict_app: detect and uninstall conflicting apps
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

/// Modules that should be removed entirely (rm -rf) rather than just disabled.
const RMRF_MODULES: &[&str] = &[
    "TA_utl",
    ".TA_utl",
    "yamabukiko",
];

/// Apps (package names) that conflict with us.
const CONFLICT_APPS: &[&str] = &[
    "com.lingqiaohan.appbl",
    "com.topmiaohuan.hidebllist",
];

/// Tag conflicting modules found in any module root directory.
pub fn conflict_mod() -> Result<(), String> {
    common::log_i("conflict-mod", "start");

    let modules_dir = common::MODULES;
    let modules_update_dir = format!("{modules_dir}_update");

    let mut tagged = 0u32;
    let mut removed = 0u32;

    for base in [&modules_dir, &modules_update_dir] {
        if !common::exists(base) {
            continue;
        }

        // RMRF modules: remove entirely
        for id in RMRF_MODULES {
            let dir = format!("{base}/{id}");
            if common::exists(&dir) {
                common::log_i("conflict-mod", &format!("removing {id}"));
                let _ = std::fs::remove_dir_all(&dir);
                removed += 1;
            }
        }

        // Conflict modules: tag with disable + remove marker files
        for id in CONFLICT_MODULES {
            let dir = format!("{base}/{id}");
            if !common::exists(&dir) {
                continue;
            }

            let disable = format!("{dir}/disable");
            let remove = format!("{dir}/remove");

            if !common::exists(&disable) {
                common::write_file(&disable, "");
            }
            if !common::exists(&remove) {
                common::write_file(&remove, "");
            }

            common::log_i("conflict-mod", &format!("tagged {id}"));
            tagged += 1;
        }
    }

    common::log_i(
        "conflict-mod",
        &format!("done: tagged={tagged} removed={removed}"),
    );
    Ok(())
}

/// Uninstall conflicting apps found on the device.
pub fn conflict_app() -> Result<(), String> {
    common::log_i("conflict-app", "start");

    let installed = common::pm_list_packages_3();
    let mut removed = 0u32;

    for pkg in CONFLICT_APPS {
        if installed.iter().any(|p| p == pkg) {
            common::log_i("conflict-app", &format!("uninstalling {pkg}"));
            common::pm_uninstall(pkg);
            removed += 1;
        }
    }

    common::log_i("conflict-app", &format!("done: removed={removed}"));
    Ok(())
}
