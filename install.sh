#!/bin/bash
set -euo pipefail
cd -- "$(dirname -- "$(readlink -f -- "$0")")"
if [[ $EUID != 0 ]]; then
  echo 'Run this installer using sudo (terminal) or pkexec (desktop).' >&2
  exit 1
fi
binary="${1:-target/release/oncekey}"
[[ -x $binary ]] || { echo 'Build first: cargo build --release --locked' >&2; exit 1; }
"$binary" check-config config.json
if [[ -e /etc/oncekey.json ]]; then
  "$binary" check-config /etc/oncekey.json
fi
# Stop before replacing the executable so upgrades do not hit ETXTBSY.
if systemctl is-active --quiet oncekey.service; then
  systemctl stop oncekey.service
fi
install -o root -g root -m 0755 "$binary" /usr/local/bin/oncekey
install -o root -g root -m 0755 "$binary" /usr/local/bin/oncekeyctl
install -o root -g root -m 0644 oncekey.service /etc/systemd/system/oncekey.service
if [[ ! -e /etc/oncekey.json ]]; then
  install -o root -g root -m 0644 config.json /etc/oncekey.json
fi
modprobe uinput
systemctl daemon-reload
systemctl enable --now oncekey.service
systemctl --no-pager status oncekey.service
