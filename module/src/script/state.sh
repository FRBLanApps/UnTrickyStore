#
# UnTrickyStore - state.sh
# Runs from service.d to refresh module description state
#
# Copyright (C) 2025-2026 FRBLanApps
# SPDX-License-Identifier: AGPL-3.0-or-later
#

cd ${0%/*}
[ -d "../modules/untrickystore" ] || rm -f "$0"
source "../modules/untrickystore/script/util_functions.sh"

initwait
invoke "刷新运行状态" "--staterefresh"
