#!/system/bin/sh
#
# UnTrickyStore - service.sh
# Runs at late_start service stage. Composes uts subcommands.
#
# Copyright (C) 2025-2026 FRBLanApps
# SPDX-License-Identifier: AGPL-3.0-or-later
#

MODDIR="${0%/*}"
UTS="$MODDIR/bin/uts"
CFG="/data/adb/untrickystore"

# Wait for boot to complete
while [ "$(getprop sys.boot_completed)" != "1" ]; do
  sleep 1
done
sleep 5

# --- Conflict notification (deferred from post-fs-data) ---
if [ -f "$CFG/conflict_list.txt" ] && [ -f "$MODDIR/disable" ]; then
  CONFLICT_LIST=$(cat "$CFG/conflict_list.txt" 2>/dev/null)
  [ -n "$CONFLICT_LIST" ] && \
    cmd notification post -S bigtext untrickystore conflict \
      "UnTrickyStore" "有冲突模块${CONFLICT_LIST}，UnTrickyStore未启动" >/dev/null 2>&1
  exit 0
fi

# --- Start inotify daemon in background ---
"$UTS" daemon &

# --- Generate target list ---
"$UTS" target

# --- Conflict: apps (block if found) ---
if ! "$UTS" conflict-app; then
  "$UTS" status
  exit 1
fi

# --- Security patch sync ---
"$UTS" patch-sync

# --- Property spoofing ---
"$UTS" propstate

# --- VBMeta hash fix ---
"$UTS" vbhash

# --- Refresh description ---
"$UTS" status
