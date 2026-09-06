# Third-party components

The debounce implementation and UI are original project code. Unshaky is an
inspiration and reference, not a vendored dependency; no Unshaky code or artwork
is included. Screenshots capture this plugin running on Omarchy.

The Linux key catalog contains key identifiers/numeric constants from
`linux/input-event-codes.h` (Linux UAPI header, GPL-2.0 WITH Linux-syscall-note).
The `builtIn` flags describe the development machine's Apple SPI keyboard.

Rust dependencies are pinned in Cargo.lock. Their package metadata declares
licenses. Review them with `cargo metadata --locked --format-version 1` before
redistributing compiled binaries; retain upstream notices as applicable.

Omarchy and Quickshell are external runtime components and are not bundled.
