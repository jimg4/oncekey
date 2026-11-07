# Changelog

## 1.2.1

- Fix duplicate letters slipping through after longer key holds: the delay now counts from the key's last press or release (including suppressed bounces), not only from the original press.
- Standardize the plugin ID, commands, service, config paths, and documentation on onceKey (`dimef.oncekey` / `oncekey`).

- Use Omarchy KeyboardPanel so reopening settings reliably acquires typing focus.
- Release panel focus before administrator authentication for save/start/stop.

## 1.2.0 — publication candidate

- onceKey branding and icon-only bar display.
- Global and per-key 0–200 ms settings, key exemptions and inherited defaults.
- Search, reset, apply-to-all, discard/reload, and typing-test controls.
- Explicit batched configuration saves with stale-settings conflict detection.
- Rust filter for the built-in Apple SPI keyboard; automatic startup via systemd.
- Marketplace preview, screenshots, compatibility guide and removal instructions.

Known limitation: physical Caps Lock LED is not synchronized while filtering.
