// UnTrickyStore - service control and status
//
// ts_ctl:  stop/start/restart TrickyStore service
// uts_ctl: stop/start/restart UTS daemon
// status:  refresh module description with live status info
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common;

/// Control TrickyStore daemon. Action: "start", "stop", "restart".
pub fn ts_ctl(action: &str) -> Result<(), String> {
    common::log_i("ts-ctl", &format!("action={action}"));

    let ts_dir = common::ts_dir();
    let service_sh = format!("{ts_dir}/service.sh");

    // Detect TS variant from module.prop
    let variant = detect_ts_variant(&ts_dir);
    let daemon_name = match variant.as_str() {
        "oss" => "trick_store",       // TrickyStore OSS
        "sim" => "trick_simulator",   // TrickyStore Simulator
        _ => "trick_store",           // Standard/unknown
    };

    match action {
        "stop" => {
            common::kill_9(daemon_name);
            common::log_i("ts-ctl", &format!("stopped {daemon_name}"));
        }
        "start" => {
            if !common::pidof(daemon_name).is_empty() {
                common::log_w("ts-ctl", "already running");
                return Ok(());
            }
            // Execute service.sh to start
            if common::exists(&service_sh) {
                let _ = std::process::Command::new("sh")
                    .arg(&service_sh)
                    .spawn();
                common::log_i("ts-ctl", "started via service.sh");
            } else {
                return Err(crate::i18n::t("TrickyStore service.sh not found"));
            }
        }
        "restart" => {
            common::kill_9(daemon_name);
            std::thread::sleep(std::time::Duration::from_millis(500));
            if common::exists(&service_sh) {
                let _ = std::process::Command::new("sh")
                    .arg(&service_sh)
                    .spawn();
                common::log_i("ts-ctl", "restarted");
            }
        }
        _ => return Err(format!("ts-ctl: unknown action '{action}'")),
    }

    Ok(())
}

/// Control UTS daemon process. Action: "start", "stop", "restart".
pub fn uts_ctl(action: &str) -> Result<(), String> {
    common::log_i("uts-ctl", &format!("action={action}"));

    let uts = common::uts_bin();

    match action {
        "stop" => {
            common::kill_9("uts");
            common::log_i("uts-ctl", "stopped");
        }
        "start" => {
            if !common::pidof("uts").is_empty() {
                common::log_w("uts-ctl", "already running");
                return Ok(());
            }
            let _ = std::process::Command::new(&uts)
                .arg("daemon")
                .spawn();
            common::log_i("uts-ctl", "started daemon");
        }
        "restart" => {
            common::kill_9("uts");
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = std::process::Command::new(&uts)
                .arg("daemon")
                .spawn();
            common::log_i("uts-ctl", "restarted daemon");
        }
        _ => return Err(format!("uts-ctl: unknown action '{action}'")),
    }

    Ok(())
}

/// Generate module description string with live status and write to module.prop.
/// This appears in Magisk/KSU module manager as the module description.
pub fn status() -> Result<(), String> {
    common::log_i("status", "refreshing description");

    let uts_dir = common::uts_dir();
    let cfg = common::uts_cfg();
    let ts_dir = common::ts_dir();
    let module_prop = format!("{uts_dir}/module.prop");

    if !common::exists(&module_prop) {
        return Err("status: module.prop not found".into());
    }

    // --- Gather info ---

    // Root solution
    let root_file = format!("{cfg}/root.txt");
    let root_name = if common::exists(&root_file) {
        common::read_file(&root_file).trim().to_string()
    } else {
        "Unknown".to_string()
    };

    let kernel_file = format!("{cfg}/kernel.txt");
    let kernel_ver = if common::exists(&kernel_file) {
        let raw = common::read_file(&kernel_file).trim().to_string();
        if raw.is_empty() { String::new() } else { format!(" ({raw})") }
    } else {
        String::new()
    };

    // Multiple root warning
    let multi_file = format!("{cfg}/multiple.txt");
    let multi_warn = if common::exists(&multi_file) {
        let m = common::read_file(&multi_file).trim().to_string();
        if m.is_empty() {
            String::new()
        } else {
            format!(" ⚠{}", crate::i18n::t_fmt("Multi: %s", &m))
        }
    } else {
        String::new()
    };

    // TS service status
    let ts_variant = detect_ts_variant(&ts_dir);
    let ts_daemon = match ts_variant.as_str() {
        "oss" => "trick_store",
        "sim" => "trick_simulator",
        _ => "trick_store",
    };
    let ts_running = !common::pidof(ts_daemon).is_empty();
    let ts_status = if ts_running { "✅" } else { "❌" };

    // UTS daemon status
    let uts_running = !common::pidof("uts").is_empty();
    let uts_status = if uts_running { "✅" } else { "❌" };

    // Security patch
    let patch_file = format!("{cfg}/security_patch.txt");
    let patch_info = if common::exists(&patch_file) {
        let date = common::read_file(&patch_file).trim().to_string();
        if date.is_empty() {
            String::new()
        } else {
            format!(" | SPL: {date}")
        }
    } else {
        String::new()
    };

    // Build description line
    let desc = format!(
        "{root_name}{kernel_ver}{multi_warn} | TS: {ts_status} UTS: {uts_status}{patch_info}"
    );

    // Update module.prop description= line
    let prop_content = common::read_file(&module_prop);
    let mut lines: Vec<String> = Vec::new();
    let mut found = false;

    for line in prop_content.lines() {
        if line.starts_with("description=") {
            lines.push(format!("description={desc}"));
            found = true;
        } else {
            lines.push(line.to_string());
        }
    }

    if !found {
        lines.push(format!("description={desc}"));
    }

    common::write_file(&module_prop, &lines.join("\n"));
    common::log_i("status", &format!("desc: {desc}"));
    Ok(())
}

/// Detect TrickyStore variant from module.prop description.
fn detect_ts_variant(ts_dir: &str) -> String {
    let prop = format!("{ts_dir}/module.prop");
    if !common::exists(&prop) {
        return "unknown".to_string();
    }
    let content = common::read_file(&prop);
    let desc = content
        .lines()
        .find(|l| l.starts_with("description="))
        .unwrap_or("");

    if desc.contains("OSS") || desc.contains("oss") {
        "oss".to_string()
    } else if desc.contains("Simulator") || desc.contains("simulator") {
        "sim".to_string()
    } else {
        "standard".to_string()
    }
}
