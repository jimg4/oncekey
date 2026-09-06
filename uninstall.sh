#!/bin/bash
set -euo pipefail
[[ $EUID == 0 ]] || { echo 'Run with sudo or pkexec.' >&2; exit 1; }
[[ $# == 0 || ( $# == 1 && $1 == --purge ) ]] || { echo 'Usage: uninstall.sh [--purge]' >&2; exit 1; }
systemctl disable --now butterfly-guard.service
rm -f /etc/systemd/system/butterfly-guard.service /usr/local/bin/butterfly-guard /usr/local/bin/butterfly-guardctl
if [[ ${1:-} == --purge ]]; then rm -f /etc/butterfly-guard.json; fi
systemctl daemon-reload
printf '%s\n' 'System filter removed. Remove the bar plugin as your normal user: omarchy plugin remove dimef.butterfly-guard --yes'
