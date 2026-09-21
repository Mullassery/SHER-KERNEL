# Changelog

All notable changes to this project are documented here, in the style of
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). This file was
added retroactively during a 2026-09 documentation-honesty pass; entries
before that date are reconstructed from real `git log` history (commit
hashes cited below), not invented. There is no prior published
`CHANGELOG.md` to reconcile against.

There have been no tagged releases published to any package registry
(this workspace is not on crates.io) other than the git tag `v0.2.0`.
Workspace version is currently `0.3.0` (`Cargo.toml`), which has not been
tagged.

## [Unreleased]

### Changed
- Documentation-honesty pass (this pass): fixed `Cargo.lock` being
  gitignored/untracked despite this workspace producing a real binary
  (`sher-kernel`) and benches — it's now committed. Corrected a stale
  test-count mismatch between `CLAUDE.md` (767) and `README.md` (768).
  Rewrote `.github/GITHUB_GUIDE.md` and `.github/CONTRIBUTING.md`, which
  still contained fabricated performance numbers, a false "proves you can
  run Linux drivers on a different kernel" claim, references to
  non-existent root-level docs, and invented per-crate test-count targets
  — none of which matched this repo's own README/CLAUDE/VISION/ROADMAP
  honesty standard from the prior pass (see `6b45086`, `67423bf`,
  `e5e8386`). Added a correction notice to `API_REFERENCE.md`, the one
  remaining top-level doc still claiming "Version 1.0.0"/"Production Use"/
  "Phase 13" framing. Added `SECURITY.md`, `ROADMAP_HONEST.md` (technical
  debt ledger), `.github/dependabot.yml`, issue/PR templates, and
  `docs/architecture/README.md` (crate dependency graph), none of which
  existed before.

## [0.3.0] - unreleased (current `Cargo.toml` version, not tagged)

Work landed on `main` at this version, per real commit history:

- `95e8ba5` (2026-08-26): `crates/core`'s IPC mailbox rewritten from a
  `HashMap<String, VecDeque<Message>>` behind `&mut self` to a real
  lock-free, zero-copy (`Arc<[u8]>` payload) bounded ring buffer
  (`crossbeam_queue::ArrayQueue`).
- `658d178` (2026-09-06): relicensed the whole workspace to Apache
  License 2.0 (previously a custom attribution-based license).
- `e5e8386` (2026-08-18): added this repo's first CI workflow
  (`.github/workflows/ci.yml`: fmt/build/test/clippy) — there was none
  before; fixed 5 benchmark files that didn't compile at all.
- `48d4493` (2026-08-16): implemented real logic for ~15 crates that had
  previously been near-empty stubs.
- `b10ff52` (2026-08-16) through `6b45086` (2026-09-11): multi-commit
  documentation-honesty pass correcting "v1.0.0 Production Ready"/
  phase-complete claims across `README.md`, `CLAUDE.md`, `VISION.md`,
  `ROADMAP.md`; archived 23 older docs with that framing into
  `docs/archive/` with correction notices.

## [0.2.0] - 2026-08-16 (tag `v0.2.0`)

Earliest tagged point in this repo's history. Predates the documentation-
honesty pass above; docs at this tag described the project in "v1.0.0
Production Ready," phase-complete-schedule terms that later commits (see
0.3.0 above) corrected. See `git log v0.2.0` for the full pre-tag history
if needed; not reconstructed here in detail to avoid re-stating claims
this changelog's own later entries disown.
