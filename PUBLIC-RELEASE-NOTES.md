# Public release notes

This directory is a sanitized export prepared from a private development/archive repository.

## Intentionally excluded

- all Git history from the private repository;
- private commit author email metadata;
- the original multi-gigabyte historical archive;
- historical files not needed to substantiate material audit findings;
- raw/frozen market-data archives;
- row-level final-holdout prediction dumps;
- large row-level strategy signal/holding-period tables;
- Google Drive or other private-share links;
- local cache, build, bytecode, and test-cache files.

## Historical-source wording

The public `historical/audited-evidence/` directory is a **curated subset**, not the complete recovered source snapshot. `historical/SOURCE-MANIFEST.md` is an inventory of the private 137-file source-only archive.

## Commit-history policy

Create a brand-new repository from this folder. Do not preserve or import commits from the private development repository.

Suggested clean public commit sequence:

1. `Add sanitized historical audit evidence`
2. `Add reproducible 2026 RRG rebuild`
3. `Add frozen model selection and final holdout results`
4. `Add prospective forecast mode`
5. `Add economic-value backtest and negative result`

Before committing, configure a GitHub noreply identity and verify the author email with `git config user.email`.
