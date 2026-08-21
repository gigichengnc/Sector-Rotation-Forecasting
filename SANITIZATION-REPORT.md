# Sanitization report

This report documents the checks performed on this public-release bundle before any Git repository is created from it.

## Privacy / disclosure checks

The bundle was scanned recursively after assembly.

- private email address patterns: **0 matches**
- Google Drive / Google Docs share links: **0 matches**
- known personal-name / private-repository username patterns from the source workspace: **0 matches**
- local absolute workspace / user-profile path patterns: **0 matches**
- high-risk secret patterns (private keys, AWS-style access keys, GitHub token patterns, client secrets): **0 matches**

The selected historical evidence files were taken directly from the recovered private 137-file source-only archive. All **15/15** selected evidence files matched the bytes and SHA-256 values recorded in the private source manifest.

## Historical release scope

- private recovered source-only archive: **137 files**
- historical files included publicly: **15**
- complete historical archive published: **no**
- full file-name/hash inventory published: **yes**
- `.kiro/` specifications published: **no**
- `OLD Assets/` published: **no**

The public wording explicitly describes `historical/audited-evidence/` as a curated subset.

## Software checks

The sanitized bundle contains **44 tests** across:

- data parsing / completed-week cutoffs: 7
- common-history dataset logic: 5
- prospective deployment: 5
- final-holdout guards: 6
- modeling: 5
- RRG calculation: 5
- strategy/backtest logic: 6
- targets: 2
- validation folds: 3

The assembled bundle was rerun locally after sanitization and all 44 tests passed.

Separately, the private integration PR from which the 2026 implementation was exported had already recorded a successful GitHub Actions run on Python 3.11.16 with 44 passing tests. The exact resolved package versions from that CI run are recorded in `rebuild/2026/requirements-lock.txt`.

A new public repository must run its own included GitHub Actions workflow before release.

## Excluded content

The bundle intentionally excludes:

- all private Git history and commit metadata;
- private author email metadata;
- the full historical source archive;
- raw/frozen market-data ZIPs;
- row-level final-holdout prediction dumps;
- large row-level strategy signal/holding-period tables;
- private-share links;
- local caches, bytecode, editor metadata, and build artifacts.

## Release status

This is a **clean-tree export**, not a Git repository. It is suitable as input to a new repository only after the user configures a GitHub noreply commit identity and completes `PUBLIC-RELEASE-CHECKLIST.md`.
