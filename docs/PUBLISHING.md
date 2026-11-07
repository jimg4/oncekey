# Publication status

Public repository: https://github.com/jimg4/oncekey
Marketplace submission: https://github.com/omacom/omarchy-plugin-marketplace/issues/5257
Plugin ID: `dimef.oncekey`
Display name: onceKey

The plugin is listed and verified at
https://omarchyplugins.com/plugin.html?id=dimef.oncekey. Submission issue 5257 is
closed; editing it does not update the listing.

To publish a newer commit, push it, then open a **Plugin verification** issue
("Verify and publish a newer upstream commit") with the full HEAD SHA. A
marketplace maintainer approves it before the listing switches to that commit.

```sh
git push origin main
gh issue create --repo omacom/omarchy-plugin-marketplace \
  --title "[Verify]: dimef.oncekey update verification $(git rev-parse --short HEAD)" \
  --body "..."  # sections from verify-plugin.yml
```

Manual setup is required: marketplace installation loads the UI; building and
installing the system filter requires the documented administrator installer.
Do not mark the plugin approved or verified; marketplace maintainers handle review.

Validation covers Rust tests, JS settings tests, Clippy, release build, manifest
validation, and local service operation. It does not establish support for additional
hardware models, suspend/resume behavior, latency benchmarks, or a security audit.
