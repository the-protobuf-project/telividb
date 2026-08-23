# Repository settings

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
