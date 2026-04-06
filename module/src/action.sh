#!/system/bin/sh
#
# UnTrickyStore - action.sh
# Runs when user taps the module action button (Magisk only).
# KSU/APatch use webui natively.
#
# Copyright (C) 2025-2026 FRBLanApps
# SPDX-License-Identifier: AGPL-3.0-or-later
#

MODDIR="${0%/*}"
UTS="$MODDIR/bin/uts"

"$UTS" webui
