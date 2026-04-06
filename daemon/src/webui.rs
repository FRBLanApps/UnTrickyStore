// UnTrickyStore - WebUI launcher
//
// Opens the module WebUI in a suitable host app.
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common;
use anyhow::Context;

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
pub fn launch() -> anyhow::Result<()> {
    log::info!(target: "webui", "launch");

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
        log::info!(target: "webui", "installing bundled webui host");
        if common::pm_install(&webui_apk) {
            return launch_ksu_webui(module_id);
        }
        anyhow::bail!("webui: failed to install webui host apk");
    }

    anyhow::bail!("{}", crate::i18n::t("No WebUI host app found, please install MMRL or KSU WebUI"))
}

fn launch_mmrl(module_id: &str) -> anyhow::Result<()> {
    log::info!(target: "webui", "launching MMRL");

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
        .context("am start failed")?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("webui: am start for MMRL failed")
    }
}

fn launch_ksu_webui(module_id: &str) -> anyhow::Result<()> {
    log::info!(target: "webui", "launching KSU WebUI");

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
        .context("am start failed")?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("webui: am start for KSU WebUI failed")
    }
}
