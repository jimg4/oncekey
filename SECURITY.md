# Permissions and data handling

The shell widget runs as the desktop user. Reading settings and aggregate status
needs no administrator privileges. Saving settings or starting/stopping the
system service invokes the fixed root-owned helper through pkexec.

The filter runs as root because it exclusively grabs the matching physical
keyboard and creates a uinput virtual keyboard. It necessarily sees that
keyboard's events, including while entering passwords. It does not write typed
text, per-key history, or per-key statistics to disk, and makes no network calls.
Only aggregate dropped-press counts and service state are written to /run.
The typing-test field is local UI state, not a telemetry feature.

Installation copies a locally built executable to /usr/local/bin and a systemd
unit to /etc/systemd/system. There are no passwordless sudoers rules, remote
installer scripts, or broadly writable input-device permissions. Configuration
is stored in /etc/butterfly-guard.json and explicitly saved through the panel.

The service uses NoNewPrivileges, ProtectSystem=strict, ProtectHome, PrivateTmp,
and restricted address families. This is not a security audit or a claim that
handling raw keyboard input is risk-free.

For reports, do not include passwords, typed input recordings, or other sensitive
material. Use repository issues for non-sensitive problems. Stop filtering with
`sudo systemctl disable --now butterfly-guard.service` if input misbehaves.
