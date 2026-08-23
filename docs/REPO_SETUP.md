# Repository settings

## The api-linter version must match CI

CI pins **api-linter 2.3.1**, and `cargo xtask lint-proto` refuses to run
against a different version.

That is not fussiness. A newer linter adds rules, so a local install behind CI
reports clean on protos CI rejects — which is exactly how AIP-191's
`proto-package` rule reached a pull request while a three-version-old binary
insisted everything passed. A check that disagrees with CI is worse than no
check, because it is believed.

Install the pinned build from the
[releases page](https://github.com/googleapis/api-linter/releases/tag/v2.3.1),
not `go install ...@latest`. When CI's pin changes, `PINNED_VERSION` in
`tools/xtask/src/lint_proto.rs` changes with it.

## Editor: the AIP linter needs import paths

If the editor reports dozens of `imported file does not exist` and
`cannot find google.api.field_behavior in this scope` errors, none of them are
real. They are **one** missing setting, cascading: when `google/api/*` will not
resolve, every annotation in every file that imports it becomes unresolvable
too, so a single missing import path reads as a hundred and sixty errors.

Fix it once:

```bash
cargo xtask vendor-proto   # writes protobuf/.deps (gitignored)
```

`.vscode/settings.json` already points `gapi.protoPath` at it, so this is the
only step. Re-run it if the dependency set in `buf.yaml` changes.

Two related settings are also committed there:

- **`gapi.ignoreCommentDisables: true`** — matching CI, so a suppression cannot
  appear to satisfy a rule locally and then fail on push.
- **`files.exclude: protobuf/.deps`** — the vendored files are Google's, and
  linting them reports their style as though it were this repository's.

**`cargo xtask lint-proto` is the local authority**, and CI uses the
`setup-google-api-linter` action for the same job — the action installs the
linter, so running the xtask there as well would need a second install for no
gain. Both resolve imports through `buf export` and both ignore suppressions,
so they agree.

`cargo xtask lint-proto`  It resolves the closure through
`buf export`, runs the linter with suppressions ignored, and checks the exit
status. The editor is a convenience that agrees with it once configured.


## Local: make sure rustup is actually in charge

If Rust was ever installed via Homebrew, `/opt/homebrew/bin/rustc` shadows
rustup's shims in `~/.cargo/bin`, and **`rust-toolchain.toml` is silently
ignored** — rustup knows about the pin, but the `rustc` your shell runs is not
rustup's.

Check:

```bash
which rustc                  # want ~/.cargo/bin/rustc
rustc --version              # want the version in rust-toolchain.toml
rustup which rustc           # what rustup *would* run
```

If those disagree, either `brew uninstall rust`, or put `~/.cargo/bin` ahead of
`/opt/homebrew/bin` on `PATH`. The `toolchain is pinned` CI job catches this,
but only after a push — locally it fails silently and you build against the
wrong compiler without noticing.


Two of these cannot be set from a file in the repo. Without them, the
auto-merge workflow does the opposite of what it looks like it does.

## Required before auto-merge is safe

### 1. Enable auto-merge

**Settings → General → Pull Requests → Allow auto-merge**

Without it, `gh pr merge --auto` fails and every Dependabot PR waits for a human.
That is a nuisance, not a hazard.

### 2. Protect `main` with required status checks

**Settings → Branches → Add branch ruleset → Require status checks to pass**

This one *is* a hazard if skipped. **`--auto` means "merge when requirements are
met." With no branch protection, there are no requirements, so the PR merges
immediately and CI passing becomes irrelevant** — the workflow would then be a
machine for merging unreviewed dependency bumps into `main`.

Mark these checks required:

```
linux · x86_64
linux · arm64
macos · arm64
fmt · clippy
file length
no C toolchain
toolchain is pinned
crash consistency · telemetry leaks
```

Also enable **Require branches to be up to date before merging**, so a bump is
tested against current `main` rather than against whatever `main` was when the
PR opened.

## What is auto-merged

| Update | Behaviour |
|---|---|
| Patch, minor | Auto-merged once every required check passes |
| Major | Labelled `needs-review`, commented on, left alone |
| Rust toolchain | Never — pinned in `rust-toolchain.toml`, upgraded by a reviewed commit |

Majors are excluded because that is where an on-disk format, a hash output, or a
trait signature changes. `sha2` is the sharpest example: a change to its output
would invalidate every schema and model fingerprint ever written, and the tests
would still pass, because they compare fingerprints to each other rather than to
known constants.

## The check that earns its place

`no C toolchain` greps `cargo tree` for `cc`, `cmake` and `bindgen`. It caught a
real violation the first time it ran — `blake3`, added for fingerprints, pulls
`cc` as a build dependency for its assembly paths, which broke the
toolchain-free build rule.

That is precisely the failure an unattended merge would otherwise ship, since
nothing else about such a bump looks wrong.
