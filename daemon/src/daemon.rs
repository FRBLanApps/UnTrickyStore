// UnTrickyStore - inotify watcher daemon
//
// Watches /data/adb/modules_update (new modules) and /data/app (app install/uninstall).
// On events, invokes the appropriate uts subcommands.
//
// Copyright (C) 2025-2026 FRBLanApps
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common;
use std::ffi::CString;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const DEBOUNCE_MS: u64 = 1000;

fn uts_bin_path() -> String {
    format!("{}/uts", common::uts_bin())
}

/// Invoke uts subcommand(s) in sequence.
fn invoke(cmds: &[&str]) {
    let bin = uts_bin_path();
    for cmd in cmds {
        common::log_i("daemon", &format!("invoke: uts {cmd}"));
        let _ = Command::new(&bin).arg(cmd).status();
    }
}

/// Watch a directory with inotify, invoke commands on events.
fn watch(path: &str, events: u32, cmds: Vec<&'static str>, tx: mpsc::Sender<bool>) {
    if !common::exists(path) {
        common::log_e("daemon", &format!("directory not found: {path}"));
        tx.send(false).ok();
        return;
    }

    let fd = unsafe { libc::inotify_init() };
    if fd < 0 {
        common::log_e("daemon", "inotify_init failed");
        tx.send(false).ok();
        return;
    }

    let c_path = match CString::new(path) {
        Ok(p) => p,
        Err(_) => {
            common::log_e("daemon", "invalid path");
            tx.send(false).ok();
            return;
        }
    };

    let wd = unsafe { libc::inotify_add_watch(fd, c_path.as_ptr(), events) };
    if wd < 0 {
        common::log_e("daemon", &format!("inotify_add_watch failed for {path}"));
        unsafe { libc::close(fd); }
        tx.send(false).ok();
        return;
    }

    common::log_i("daemon", &format!("watching {path}"));
    tx.send(true).ok();

    let mut buf = [0u8; 1024];
    let mut last_fire = Instant::now() - Duration::from_secs(10);
    let debounce = Duration::from_millis(DEBOUNCE_MS);

    loop {
        // Blocking read
        let n = unsafe {
            libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
        };
        if n < 0 {
            common::log_e("daemon", "inotify read error, exiting watch loop");
            break;
        }
        // Debounce: skip if fired recently
        if last_fire.elapsed() < debounce {
            continue;
        }
        last_fire = Instant::now();
        invoke(&cmds);
    }

    unsafe {
        libc::inotify_rm_watch(fd, wd);
        libc::close(fd);
    }
}

pub fn run() -> Result<(), String> {
    common::log_i("daemon", "starting daemon");

    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    // Thread 1: watch modules_update for new conflicting modules
    thread::spawn(move || {
        watch(
            "/data/adb/modules_update",
            (libc::IN_CREATE | libc::IN_ISDIR) as u32,
            vec!["conflict-mod"],
            tx1,
        );
    });

    // Thread 2: watch /data/app for app installs/uninstalls → refresh targets & check conflicts
    thread::spawn(move || {
        watch(
            "/data/app",
            (libc::IN_CREATE | libc::IN_DELETE) as u32,
            vec!["conflict-app", "target"],
            tx2,
        );
    });

    let ok1 = rx1.recv().unwrap_or(false);
    let ok2 = rx2.recv().unwrap_or(false);

    if ok1 && ok2 {
        common::log_i("daemon", "all watchers ready");
    } else if ok1 || ok2 {
        common::log_w("daemon", "partial watcher startup");
    } else {
        common::log_e("daemon", "all watchers failed");
        return Err("daemon: all watchers failed to start".into());
    }

    // Park main thread forever (threads do the work)
    thread::park();
    Ok(())
}
