#!/system/bin/sh
#
# UnTrickyStore - customize.sh
# Runs at module install time. Composes uts subcommands.
#
# Copyright (C) 2025-2026 FRBLanApps
# SPDX-License-Identifier: AGPL-3.0-or-later
#

SKIPUNZIP=1

# --- Check OverlayFS compatibility ---
if [ -f "/data/adb/.overlayfs_enable" ] || \
   ([ -f "/data/adb/ksu/mount_system" ] && grep -q "OVERLAYFS" "/data/adb/ksu/mount_system" 2>/dev/null); then
    ui_print "! Error: OverlayFS mount system not supported"
    ui_print "! Please switch to Magic Mount or Meta Module mount system"
    abort "UnTrickyStore requires Magic Mount or Meta Module mount system"
fi

# Extract module files
ui_print "- Extracting module files"
unzip -o "$ZIPFILE" -x 'META-INF/*' -d "$MODPATH" >&2

# Set permissions
set_perm_recursive "$MODPATH" 0 0 0755 0644
set_perm_recursive "$MODPATH/bin" 0 0 0755 0755
[ -f "$MODPATH/service.apk" ] && set_perm "$MODPATH/service.apk" 0 0 0644

UTS="$MODPATH/bin/uts"
chmod 0755 "$UTS"

# --- Init config ---
ui_print "- Initializing config"
"$UTS" init-cfg

# --- Root detection ---
ui_print "- Detecting root environment"
ROOT=$("$UTS" rootdetect)
ui_print "  Root: $ROOT"

# --- Conflict: modules ---
ui_print "- Checking conflicting modules"
"$UTS" conflict-mod

# --- Conflict: apps ---
ui_print "- Checking conflicting apps"
"$UTS" conflict-app

# --- Generate target.txt ---
ui_print "- Generating target list"
"$UTS" target

# --- Refresh description ---
"$UTS" status

ui_print "- Done!"
