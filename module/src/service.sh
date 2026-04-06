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

# Wait for boot to complete
while [ "$(getprop sys.boot_completed)" != "1" ]; do
  sleep 1
done
sleep 5

# --- Start inotify daemon in background ---
"$UTS" daemon &

# --- Generate target list ---
"$UTS" target

# --- Conflict: apps (needs pm to be ready) ---
"$UTS" conflict-app

# --- Security patch sync ---
"$UTS" patch-sync

# --- Property spoofing ---
"$UTS" propstate

# --- VBMeta hash fix ---
"$UTS" vbhash

# --- Refresh description ---
"$UTS" status
