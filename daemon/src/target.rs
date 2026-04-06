// UnTrickyStore - target.txt generation
//
// Generates the target package list for TrickyStore from
// installed third-party packages, filtered by user config.
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common;
use std::collections::BTreeSet;

/// Generate target.txt in TS config directory.
///
/// Logic:
/// - Base set = all third-party packages (`pm list packages -3`)
/// - If `usr.txt` (user blacklist) exists, subtract those
/// - If `sys.txt` (system additions) exists, add those
/// - Write result to `<ts_dir>/target.txt`
pub fn target() -> Result<(), String> {
    common::log_i("target", "start");

    let cfg = common::uts_cfg();
    let ts_dir = common::ts_dir();

    let usr_path = format!("{cfg}/usr.txt");
    let sys_path = format!("{cfg}/sys.txt");
    let target_path = format!("{ts_dir}/target.txt");

    // Base: third-party packages
    let packages = common::pm_list_packages_3();
    let mut set: BTreeSet<String> = packages.into_iter().collect();

    // Subtract user blacklist
    if common::exists(&usr_path) {
        let content = common::read_file(&usr_path);
        for line in content.lines() {
            let pkg = line.trim();
            if !pkg.is_empty() && !pkg.starts_with('#') {
                set.remove(pkg);
            }
        }
    }

    // Add system additions
    if common::exists(&sys_path) {
        let content = common::read_file(&sys_path);
        for line in content.lines() {
            let pkg = line.trim();
            if !pkg.is_empty() && !pkg.starts_with('#') {
                set.insert(pkg.to_string());
            }
        }
    }

    // Write target.txt
    let body: String = set.into_iter().collect::<Vec<_>>().join("\n");
    common::write_file(&target_path, &body);

    common::log_i("target", &format!("wrote {} packages to target.txt", body.lines().count()));
    Ok(())
}
