#!/system/bin/sh
#
# UnTrickyStore - post-fs-data.sh
# Runs early in boot (post-fs-data stage). Composes uts subcommands.
#
# Copyright (C) 2025-2026 FRBLanApps
# SPDX-License-Identifier: AGPL-3.0-or-later
#

MODDIR="${0%/*}"
UTS="$MODDIR/bin/uts"
CFG="/data/adb/untrickystore"

# --- Root detection ---
"$UTS" rootdetect >/dev/null 2>&1

# --- Conflict: modules (block if found) ---
if ! "$UTS" conflict-mod; then
  # Conflicts detected — abort and mark module as broken
  touch "$MODDIR/disable"

  CONFLICT_LIST=$(cat "$CFG/conflict_list.txt" 2>/dev/null)
  [ -z "$CONFLICT_LIST" ] && CONFLICT_LIST="unknown"

  # Update only self module description
  NEW_DESC="description=有冲突模块${CONFLICT_LIST}，UnTrickyStore未启动"
  if grep -q "^description=" "$MODDIR/module.prop" 2>/dev/null; then
    sed -i "s|^description=.*|$NEW_DESC|" "$MODDIR/module.prop"
  else
    echo "$NEW_DESC" >> "$MODDIR/module.prop"
  fi

  exit 1
fi

# --- Environment sanity check ---
ROOT=$(cat "$CFG/root.txt" 2>/dev/null)
MULTI=$(cat "$CFG/multiple.txt" 2>/dev/null)
TS_DIR="/data/adb/modules/tricky_store"

if [ "$ROOT" = "Multiple" ] || [ ! -d "$TS_DIR" ] || [ -f "$TS_DIR/disable" ]; then
  # Bad environment: hide WebUI and action
  mv "$MODDIR/webroot" "$MODDIR/.webroot" 2>/dev/null
  mv "$MODDIR/action.sh" "$MODDIR/.action.sh" 2>/dev/null
else
  # Good environment: restore WebUI and action
  mv "$MODDIR/.webroot" "$MODDIR/webroot" 2>/dev/null
  # action.sh only restored for Magisk (not KSU/APatch which use WebUI natively)
  if [ -z "$KSU" ] && [ -z "$APATCH" ]; then
    mv "$MODDIR/.action.sh" "$MODDIR/action.sh" 2>/dev/null
  fi
fi

# --- Refresh description ---
"$UTS" status
