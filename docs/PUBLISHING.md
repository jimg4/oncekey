# Publication status

Public repository: https://github.com/jimg4/oncekey
Marketplace submission: https://github.com/omacom/omarchy-plugin-marketplace/issues/5257
Plugin ID: `dimef.oncekey`
Display name: onceKey

The existing submission is awaiting marketplace maintainer review. Update that
issue after pushing changes to trigger validation of the current repository HEAD.
The complete submission body is in `docs/marketplace-submission.md`.

```sh
git push origin main
gh issue edit 5257 --repo omacom/omarchy-plugin-marketplace \
  --body-file docs/marketplace-submission.md
```

Manual setup is required: marketplace installation loads the UI; building and
installing the system filter requires the documented administrator installer.
Do not mark the plugin approved or verified; marketplace maintainers handle review.

Validation covers Rust tests, JS settings tests, Clippy, release build, manifest
validation, and local service operation. It does not establish support for additional
hardware models, suspend/resume behavior, latency benchmarks, or a security audit.
