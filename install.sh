#!/bin/bash
set -euo pipefail
cd -- "$(dirname -- "$(readlink -f -- "$0")")"
if [[ $EUID != 0 ]]; then
  echo 'Run this installer using sudo (terminal) or pkexec (desktop).' >&2
  exit 1
fi
binary="${1:-target/release/butterfly-guard}"
[[ -x $binary ]] || { echo 'Build first: cargo build --release --locked' >&2; exit 1; }
"$binary" check-config config.json
if [[ -e /etc/butterfly-guard.json ]]; then
  "$binary" check-config /etc/butterfly-guard.json
fi
# Stop before replacing the executable so upgrades do not hit ETXTBSY.
if systemctl is-active --quiet butterfly-guard.service; then
  systemctl stop butterfly-guard.service
fi
install -o root -g root -m 0755 "$binary" /usr/local/bin/butterfly-guard
install -o root -g root -m 0755 "$binary" /usr/local/bin/butterfly-guardctl
install -o root -g root -m 0644 butterfly-guard.service /etc/systemd/system/butterfly-guard.service
if [[ ! -e /etc/butterfly-guard.json ]]; then
  install -o root -g root -m 0644 config.json /etc/butterfly-guard.json
fi
modprobe uinput
systemctl daemon-reload
systemctl enable --now butterfly-guard.service
systemctl --no-pager status butterfly-guard.service
