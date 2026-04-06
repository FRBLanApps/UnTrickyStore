#!/system/bin/sh
#
# UnTrickyStore - uninstall.sh
# Cleanup when module is removed.
#
# Copyright (C) 2025-2026 FRBLanApps
# SPDX-License-Identifier: AGPL-3.0-or-later
#

# Remove config directory
rm -rf /data/adb/untrickystore

# Kill daemon if running
killall uts 2>/dev/null
