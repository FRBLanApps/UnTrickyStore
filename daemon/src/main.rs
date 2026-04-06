// UnTrickyStore (uts) - TrickyStore Enhancement Toolkit
//
// Each subcommand does one thing. Orchestration belongs to shell.
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

mod common;
mod conflict;
mod daemon;
mod i18n;
mod patch;
mod props;
mod rootdetect;
mod service;
mod target;
mod webui;

use std::env;
use std::process::ExitCode;

fn usage() {
    eprintln!(
        "\
uts - UnTrickyStore toolkit

Usage: uts <command> [args...]

Commands:
  daemon             inotify watcher (modules_update + /data/app)
  rootdetect         detect & print root environment
  propstate          spoof bootloader unlock state
  vbhash             fix VBMeta hash via ContentProvider
  conflict-mod       tag conflicting modules for removal
  conflict-app       detect & optionally remove conflicting apps
  target             regenerate target.txt from package list
  patch-sync         sync security patch level to properties
  patch-fetch        fetch latest patch date from bulletin
  ts-ctl <action>    TrickyStore service: stop|start|restart
  uts-ctl <action>   UTS daemon: stop|start|restart
  status             refresh module description text
  webui              launch WebUI
  init-cfg           create default config dirs & files

Each command exits 0 on success, non-zero on failure.
Compose with shell for orchestration."
    );
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
        return ExitCode::from(1);
    }

    common::log_init();
    i18n::init();

    let cmd = args[1].as_str();
    let rest: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

    let result: anyhow::Result<()> = match cmd {
        "daemon"       => daemon::run(),
        "rootdetect"   => rootdetect::run(),
        "propstate"    => props::propstate(),
        "vbhash"       => props::vbhash(),
        "conflict-mod" => conflict::conflict_mod(),
        "conflict-app" => conflict::conflict_app(),
        "target"       => target::target(),
        "patch-sync"   => patch::sync(),
        "patch-fetch"  => patch::fetch(),
        "ts-ctl"       => service::ts_ctl(rest.first().copied().unwrap_or("start")),
        "uts-ctl"      => service::uts_ctl(rest.first().copied().unwrap_or("start")),
        "status"       => service::status(),
        "webui"        => webui::launch(),
        "init-cfg"     => init_cfg(),
        _ => {
            usage();
            Err(format!("unknown command: {cmd}"))
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::error!(target: "main", "{e:#}");
            eprintln!("uts: {e:#}");
            ExitCode::FAILURE
        }
    }
}

/// Create default config directories and files (idempotent).
fn init_cfg() -> anyhow::Result<()> {
    let cfg = common::uts_cfg();
    let _ = std::fs::create_dir_all(format!("{cfg}/keybox"));
    let _ = std::fs::create_dir_all(format!("{cfg}/log"));
    for name in &["usr.txt", "sys.txt", "security_patch.txt"] {
        let path = format!("{cfg}/{name}");
        if !common::exists(&path) {
            common::write_file(&path, "");
        }
    }
    Ok(())
}
