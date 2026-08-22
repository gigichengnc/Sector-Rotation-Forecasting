# Phase 4 prospective sector-forecasting freeze

This directory publicly timestamps the prospective-confirmation design that follows the v0.2 structural-null correction and the Phase 3 exploratory macro-regime study.

The purpose of this freeze is **not** to publish a new predictive claim. It records the research contract, training boundaries, model/hash commitments, provenance, reproduction gate, and empty prospective-ledger schema before the first eligible official prediction week.

First official prospective decision: **2026-08-28**.

No Phase 4 prediction or matured outcome is included in this freeze commit.

## Public freeze record

- `PHASE4-PROSPECTIVE-CONTRACT.md`: target, frozen A/B specifications, primary endpoint, timing, and interpretation rules.
- `FREEZE-PROVENANCE.md`: hashes linking the freeze to the development archive, Phase 3 contract, replay reference, and macro snapshot.
- `model_freeze_summary.csv`: training boundaries, feature counts, and SHA-256 commitments for each fixed model file.
- `phase3_reproduction_check.csv`: numerical replay gate completed before the Phase 4 freeze.
- `MACRO-SOURCE-MANIFEST.csv`: source metadata for the macro snapshot available at freeze time.
- `prospective_registry.csv`: empty immutable-ledger schema; it contains no prospective predictions at freeze time.
- `ARTIFACT-HASHES.csv`: hashes for the public freeze files themselves.

The coefficient JSON files and market-data archive are not published in this timestamp PR. Their SHA-256 commitments are recorded here before the first official prospective decision. Any later coefficient file used for Phase 4 must match the committed hash exactly; otherwise it is a different model and cannot inherit the prospective claim.

## Research boundary

The fixed `price_macro_interactions` candidate is compared with the fixed `price_sector_fe` baseline. RRG-style variables are not credited as the Phase 4 predictive engine because Phase 3 did not establish robust incremental value after the macro-interaction block.

The preselected primary horizon is 4 weeks. The minimum confirmatory sample is 52 official weekly decisions, evaluated only after all corresponding 4-week outcomes have matured.

Until that test is complete, the strongest allowed wording remains exploratory: lagged macro regimes may improve cross-sectional sector ranking, but a genuine forward-looking edge has not yet been confirmed.
