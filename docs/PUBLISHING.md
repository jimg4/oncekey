# Publication handoff

Prepared repository: ~/projects/oncekey
Proposed public repository: https://github.com/jimg4/oncekey
Submission title: [Plugin]: onceKey
Proposed license: MIT, copyright jimg4 (2026)

No GitHub repository, push, release, or marketplace issue has been created by this
preparation step. Confirm the repository name and license before publishing.

## Review before the first push

- Inspect README, LICENSE, SECURITY.md and both screenshots.
- The settings screenshot shows a real unsaved 75 ms draft, not the shipped
  default and not a recommended value for everybody.
- Confirm the legacy identifier dimef.butterfly-guard; marketplace IDs are
  permanent. Choose any new ID before the first submission and migrate all
  references together if desired.
- Verify the ID is available in the marketplace at submission time.
- The public Git install command becomes usable only after the repository exists.
- Current validation: unit/config tests, JS settings tests, lint, release build,
  manifest validation, and live panel loading. Do not claim hardware testing on
  additional models, suspend/resume testing, latency benchmarks, or a security audit.
- Manual setup is necessary: marketplace installation alone loads the UI.

## Commands after authorization to publish

From the repository root, review `git status` and the initial commit, then:

```sh
gh repo create jimg4/oncekey --public --source . --remote origin --push
```

After verifying the public README, files, and checks, review all five checklist
statements in docs/marketplace-submission.md. Check them only when confirmed true.
The first item is currently unchecked because the repository is not public yet.

Show the final submission body to the owner before submitting:

```sh
gh issue create --repo omacom/omarchy-plugin-marketplace \
  --title '[Plugin]: onceKey' --body-file docs/marketplace-submission.md
```

Do not mark the plugin approved/verified yourself; marketplace maintainers handle
listing review. Track the exact submitted commit and any manual-setup findings.

Sources checked while preparing this package:
- https://plugins.omarchy.org/publish.html
- https://github.com/omacom/omarchy-plugin-marketplace/blob/main/SUBMISSION.md
