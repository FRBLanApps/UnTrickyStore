// UnTrickyStore - inotify watcher daemon
//
// Watches /data/app for app install/uninstall events
// and regenerates target.txt accordingly.
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common;
use std::ffi::CString;
use std::process::Command;
use std::time::{Duration, Instant};

const DEBOUNCE_MS: u64 = 1000;

fn uts_bin_path() -> String {
    format!("{}/uts", common::uts_bin())
}

/// Invoke uts subcommand.
fn invoke(cmd: &str) {
    let bin = uts_bin_path();
    log::info!(target: "daemon", "invoke: uts {cmd}");
    let _ = Command::new(&bin).arg(cmd).status();
}

pub fn run() -> anyhow::Result<()> {
    log::info!(target: "daemon", "starting daemon");

    let path = "/data/app";
    anyhow::ensure!(common::exists(path), "daemon: /data/app not found");

    let fd = unsafe { libc::inotify_init() };
    anyhow::ensure!(fd >= 0, "daemon: inotify_init failed");

    let c_path = CString::new(path).map_err(|_| anyhow::anyhow!("daemon: invalid path"))?;
    let events = (libc::IN_CREATE | libc::IN_DELETE) as u32;
    let wd = unsafe { libc::inotify_add_watch(fd, c_path.as_ptr(), events) };
    if wd < 0 {
        unsafe { libc::close(fd); }
        anyhow::bail!("daemon: inotify_add_watch failed");
    }

    log::info!(target: "daemon", "watching /data/app");

    let mut buf = [0u8; 1024];
    let mut last_fire = Instant::now() - Duration::from_secs(10);
    let debounce = Duration::from_millis(DEBOUNCE_MS);

    loop {
        let n = unsafe {
            libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
        };
        if n < 0 {
            log::error!(target: "daemon", "inotify read error, exiting");
            break;
        }
        if last_fire.elapsed() < debounce {
            continue;
        }
        last_fire = Instant::now();
        invoke("target");
    }

    unsafe {
        libc::inotify_rm_watch(fd, wd);
        libc::close(fd);
    }
    Ok(())
}
