#!/bin/bash
set -euo pipefail
[[ $EUID == 0 ]] || { echo 'Run with sudo or pkexec.' >&2; exit 1; }
[[ $# == 0 || ( $# == 1 && $1 == --purge ) ]] || { echo 'Usage: uninstall.sh [--purge]' >&2; exit 1; }
systemctl disable --now oncekey.service
rm -f /etc/systemd/system/oncekey.service /usr/local/bin/oncekey /usr/local/bin/oncekeyctl
if [[ ${1:-} == --purge ]]; then rm -f /etc/oncekey.json; fi
systemctl daemon-reload
printf '%s\n' 'System filter removed. Remove the bar plugin as your normal user: omarchy plugin remove dimef.oncekey --yes'
