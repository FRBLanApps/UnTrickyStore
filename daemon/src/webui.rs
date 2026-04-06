// UnTrickyStore - WebUI launcher
//
// Opens the module WebUI in a suitable host app.
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common;

// WebUI host apps in preference order
const MMRL_PKG: &str = "com.dergoogler.mmrl.wx";
const KSU_WEBUI_PKG: &str = "io.github.a13e300.ksuwebui";

/// Launch the module WebUI.
///
/// Host detection order:
/// 1. MMRL (com.dergoogler.mmrl.wx) - preferred, native WebUI support
/// 2. KSU WebUI (io.github.a13e300.ksuwebui) - KernelSU WebUI host
/// 3. Fallback: install bundled webui.apk, then launch KSU WebUI
///
/// The WebUI index page lives at `<uts_dir>/webroot/index.html`.
pub fn launch() -> Result<(), String> {
    common::log_i("webui", "launch");

    let uts_dir = common::uts_dir();
    let module_id = common::UTS;

    // Check installed apps
    let packages = common::pm_list_packages_3();

    let has_mmrl = packages.iter().any(|p| p == MMRL_PKG);
    let has_ksu_webui = packages.iter().any(|p| p == KSU_WEBUI_PKG);

    if has_mmrl {
        return launch_mmrl(module_id);
    }

    if has_ksu_webui {
        return launch_ksu_webui(module_id);
    }

    // Fallback: install bundled WebUI host, then launch
    let webui_apk = format!("{uts_dir}/webui.apk");
    if common::exists(&webui_apk) {
        common::log_i("webui", "installing bundled webui host");
        if common::pm_install(&webui_apk) {
            return launch_ksu_webui(module_id);
        }
        return Err("webui: failed to install webui host apk".into());
    }

    Err(common::bi_str(
        "webui: 未找到WebUI宿主应用，请安装MMRL或KSU WebUI",
        "webui: no WebUI host app found, install MMRL or KSU WebUI",
    )
    .to_string())
}

fn launch_mmrl(module_id: &str) -> Result<(), String> {
    common::log_i("webui", "launching MMRL");

    let uri = format!("mmrl://webui/{module_id}");
    let status = std::process::Command::new("am")
        .args([
            "start",
            "-a",
            "android.intent.action.VIEW",
            "-d",
            &uri,
            "-n",
            &format!("{MMRL_PKG}/.ui.activity.webui.WebUIActivity"),
        ])
        .status()
        .map_err(|e| format!("am start failed: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("webui: am start for MMRL failed".into())
    }
}

fn launch_ksu_webui(module_id: &str) -> Result<(), String> {
    common::log_i("webui", "launching KSU WebUI");

    let status = std::process::Command::new("am")
        .args([
            "start",
            "-a",
            "io.github.a13e300.ksuwebui.OPEN_WEBUI",
            "-e",
            "id",
            module_id,
            "-n",
            &format!("{KSU_WEBUI_PKG}/.WebUIActivity"),
        ])
        .status()
        .map_err(|e| format!("am start failed: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("webui: am start for KSU WebUI failed".into())
    }
}
