### Repository URL

https://github.com/jimg4/oncekey

### Category

Hardware

### Tags

bar, quickshell, system

### Suggest a missing tag

keyboard

### Maintainer notes

The plugin ID, commands, service, configuration paths, and documentation now consistently use `dimef.oncekey` / `oncekey`, with `onceKey` as the display name. Rust tests, JavaScript settings tests, Clippy, the release build, and Omarchy manifest validation passed. The renamed service is running locally with the existing settings preserved. Please validate the current repository HEAD.

Update: commit `8298e07` fixes intermittent settings-panel keyboard focus on opening/reopening and releases focus before administrator authentication. Tested live across the initial opening and three rapid reopen cycles. Please validate the updated repository HEAD.

onceKey mitigates accidental double letters / switch chatter on supported Apple
SPI butterfly keyboards under Omarchy. Inspired by Unshaky on macOS, with an
independently implemented Rust evdev/uinput filter and QML settings panel.

The user can set a global delay (0–200 ms), tune individual keys, exempt keys,
search the key catalog, test typing, and explicitly save or discard changes.

MANUAL SETUP REQUIRED: adding the plugin installs its UI; the user must build the
Rust executable and run the documented administrator installer for filtering.
Please classify this as manual-setup. The root service is required to grab the
matching keyboard and create a uinput device. The panel uses pkexec for explicit
configuration and service changes. No network access or typing telemetry.

Hardware is limited to Apple SPI Keyboard / applespi/input0; this is not a claim
of support for every butterfly MacBook. Missed presses and physical damage are
not repaired. Caps Lock LED synchronization is a documented limitation.

Includes original plugin screenshots, MIT license, dependency/permission notes,
configuration tests, installation and removal instructions, and preview.png.
The plugin ID is dimef.oncekey; the display name is onceKey.

### Submission checklist

- [x] The repository is public and contains installation and removal instructions.
- [x] I have documented the plugin license and any external dependencies.
- [x] I confirm that I own or have permission to submit this plugin and its preview assets.
- [x] The plugin does not overwrite user configuration without explicit consent.
- [x] I understand that approval is for listing and is not a security review.
