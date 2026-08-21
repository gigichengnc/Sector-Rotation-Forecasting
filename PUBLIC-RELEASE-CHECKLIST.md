# Public release checklist

Do not make the new repository public until every item below is complete.

- [ ] Create a **new repository**; do not reuse private Git history.
- [ ] Enable GitHub **Keep my email addresses private**.
- [ ] Enable **Block command line pushes that expose my email**.
- [ ] Configure `git config user.email` to the exact GitHub `users.noreply.github.com` address.
- [ ] Confirm `git config user.email` before the first commit.
- [ ] After each commit, inspect `git log --format='%h %an <%ae>'`.
- [ ] Confirm no private email occurs anywhere in repository file contents.
- [ ] Confirm no Google Drive/private-share link occurs anywhere in repository file contents.
- [ ] Run a secret scan for API keys, passwords, access tokens, and private keys.
- [ ] Confirm `historical/audited-evidence/` is described as a curated subset.
- [ ] Confirm `historical/SOURCE-MANIFEST.md` is described as inventory only.
- [ ] Confirm raw market-data ZIPs are not committed.
- [ ] Confirm row-level final-holdout predictions are not committed.
- [ ] Install the locked environment on Python 3.11 and run `pytest`.
- [ ] Confirm all tests pass on the new public repository's own GitHub Actions run.
- [ ] Confirm final-holdout result files retain `holdout_reuse_allowed: false`.
- [ ] Confirm the negative strategy result remains visible in the README.
- [ ] Confirm README does not imply proprietary JdK equivalence, calibrated probability, or profitable trading.
- [ ] Decide separately whether to add a software license. No reuse license is granted by this bundle by default.
