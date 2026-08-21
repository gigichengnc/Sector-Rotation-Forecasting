# Changelog

All notable public changes to this research repository are recorded here.

## 0.2.0 - 2026-08-21

### Corrected

- Retired the earlier interpretation that the 80.6% one-week holdout accuracy demonstrated market predictability.
- Added a no-signal synthetic structural-null benchmark. The existing coordinate-forecast pipeline reproduces a similar linear-vs-persistence advantage even when returns have no serial predictability.
- Clarified that `StandardScaler` is fitted inside each training fold, while the RRG-style rolling coordinate normalization is causal but computed on the chronological series before model splitting.
- Reframed development-vs-final differences as sampling variation rather than evidence of stability.
- Reframed the first strategy result as "not supported by this backtest" rather than evidence that no alpha exists.
- Corrected the public historical-evidence inventory: three files previously listed were not actually committed.
- Added an executable historical subset verifier that documents the recovered-archive CRLF versus repository LF line-ending difference.

### Added

- Synthetic null benchmark code and tests.
- A fully future sector-relative-return target for the next research phase.
- Post-publication methodological audit documentation.
- `CITATION.cff` and this changelog.

## 0.1.0 - 2026-08-21

- Initial public packaging of the 2026 reconstruction, historical audit evidence, CLI, tests, frozen holdout records, and first pre-declared strategy test.
