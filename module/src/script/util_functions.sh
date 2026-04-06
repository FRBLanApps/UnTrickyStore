#
# UnTrickyStore - util_functions.sh
# Common utility functions for all module scripts
#
# Copyright (C) 2025-2026 FRBLanApps
# SPDX-License-Identifier: AGPL-3.0-or-later
#

##VARIABLE##
#ALIAS#
TS="tricky_store"
UTS="untrickystore"
S="service.sh"
D=".uts_state.sh"
P="post-fs-data.sh"
#ZERO LEVEL#
ADB="/data/adb"
#ONE LEVEL#
MODULESDIR="$ADB/modules"
UTSCONFIG="$ADB/$UTS"
SD="$ADB/service.d"
#TWO LEVEL#
UTSMODDIR="$MODULESDIR/$UTS"
TSMODDIR="$MODULESDIR/$TS"
#THREE LEVEL#
MULTIPLETYPE="$UTSCONFIG/multiple.txt"
KERNELTYPE="$UTSCONFIG/kernel.txt"
UTSLOG="$UTSCONFIG/log/log.log"
UTSBIN="$UTSMODDIR/bin"
TYPE="$UTSCONFIG/root.txt"
#OTHER#
ORIGIN=$(basename "$0")
##END##

##FUNCTIONS##
#MULTILINGUAL#
[[ "$(getprop persist.sys.locale)" == *"zh"* || "$(getprop ro.product.locale)" == *"zh"* ]] && LOCALE="CN" || LOCALE="EN"

println() {
  [ "$LOCALE" = "$1" ] && {
    shift
    if [ "$1" = "-n" ]; then
      shift
      echo -n "$@"
    else
      echo "$@"
    fi
  }
}

echo_cn() { println "CN" "$@"; }
echo_en() { println "EN" "$@"; }

#LOGGING#
logout() { echo "$(date "+%m-%d %H:%M:%S.$(date +%3N)")  $$  $$ $1 System.out: [UTS]$2" >> "$UTSLOG"; }
logs() { logout "$1" "<Service>$2"; }
logd() { logout "$1" "<Service.D>$2"; }
logp() { logout "$1" "<Post-Fs-Data>$2"; }

#INVOKE#
invoke() {
  case "$ORIGIN" in
    *"$S"*)
      class="logs"
      ;;
    *"$P"*)
      class="logp"
      ;;
    *"$D"*)
      class="logd"
      ;;
  esac
  "$class" "I" "$1"
  if $UTSBIN/uts $2; then
    "$class" "I" "完毕"
  else
    "$class" "W" "失败"
  fi
}

#ENVIRONMENT CHECK#
check() {
  if [ "$(cat "$TYPE" 2>/dev/null)" = "Multiple" ] || [ ! -d "$TSMODDIR" ] || [ -f "$TSMODDIR/disable" ] || sed -n '5p' "$TSMODDIR/module.prop" 2>/dev/null | grep -q -F "Enginex0"; then
    case "$ORIGIN" in
      *"$P"*)
        logp "E" "环境异常,拦截执行"
        mv "$UTSMODDIR/webroot" "$UTSMODDIR/.webroot" 2>/dev/null
        mv "$UTSMODDIR/action.sh" "$UTSMODDIR/.action.sh" 2>/dev/null
        ;;
      *"$S"*)
        exit
        ;;
    esac
  else
    [[ "$ORIGIN" == *"$P"* ]] && {
      logp "I" "环境正常,继续执行"
      mv "$UTSMODDIR/.webroot" "$UTSMODDIR/webroot" 2>/dev/null
      if [[ ! "$APATCH" && ! "$KSU" ]]; then
        mv "$UTSMODDIR/.action.sh" "$UTSMODDIR/action.sh" 2>/dev/null
      else
        mv "$UTSMODDIR/action.sh" "$UTSMODDIR/.action.sh" 2>/dev/null
      fi
    }
  fi
}

#DETECT RESULT#
detect() {
  if [ $? -eq 0 ]; then
    echo_cn "完毕"
    echo_en "Complete"
  else
    echo_cn "失败"
    echo_en "Failed"
  fi
}

#WAIT FOR BOOT#
initwait() {
  until [ "$(getprop sys.boot_completed)" = "1" ]; do
    sleep 1s
  done
}
##END##

# Clean logs on post-fs-data phase
[[ "$ORIGIN" == *"$P"* ]] && {
  rm -f "$MULTIPLETYPE"
  rm -f "$KERNELTYPE"
  rm -f "$UTSLOG"
  rm -f "$TYPE"
}
