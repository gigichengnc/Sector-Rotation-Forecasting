# Phase 4 prospective sector-forecasting contract

Frozen on 2026-08-22, before the first official prospective decision week.

## Purpose

Phase 3 produced exploratory evidence that lagged macro regimes, interacted with sector identity, may improve cross-sectional forecasts of fully future sector returns relative to SPY. Phase 4 is the prospective confirmation layer. It is designed to test that hypothesis on decisions that occur only after this contract and the model coefficients are frozen.

No Phase 4 observation before the freeze date can count as prospective evidence.

## Candidate and baseline

The candidate is the Phase 3 `price_macro_interactions` specification:

- Phase 2 price and market-regime features
- sector fixed effects
- sector-by-macro interactions
- `StandardScaler`
- `LogisticRegression(C=1.0, L2)`

The baseline is `price_sector_fe`, which keeps the same price/regime features and sector fixed effects but excludes the macro-interaction block.

Both candidate and baseline are frozen for 1, 2, 4, 8 and 12 week horizons.

RRG-style variables are not part of the Phase 4 primary predictive model. Phase 3 did not show robust incremental RRG value after macro interactions. RRG may remain a descriptive or visualization layer, but it cannot be credited as the predictive engine unless a separate future study establishes that claim.

## Target

For sector `s`, decision week `t`, and horizon `h`:

`future_relative_return = log(P_s[t+h] / P_s[t]) - log(P_SPY[t+h] / P_SPY[t])`

The binary model target is whether this fully future relative return is greater than zero. The ranking metrics use the continuous future relative return.

## Frozen training boundary

All fitted coefficients use only discovery/development rows whose target ends on or before 2025-08-15.

The final frozen fits use all eligible Phase 3 development rows for each horizon, with the same feature construction and common-week eligibility used in Phase 3. The Phase 3 out-of-sample prediction file was numerically replayed before the Phase 4 freeze. Candidate and baseline scores match the saved Phase 3 scores to floating-point precision.

No Phase 4 model is refit, retuned, replaced, horizon-selected, or feature-selected during the confirmation window.

## First prospective decision

The first official prospective decision week is **2026-08-28**.

The 2026-08-21 week is excluded because this contract and the final coefficient freeze occurred after that market week had closed. The 2026-08-14 dry run is a pipeline smoke test only and is never eligible for prospective scoring.

An official decision is recorded only after the Friday U.S. market close and only when all 12 price series have a completed observation for that Friday. A Friday market holiday requires an explicit protocol amendment before using a prior trading session.

## Information available at a decision

### Market inputs

A decision may use only the market-data snapshot retrieved after that week's close. The raw provider payload and processed panel should be archived and hashed. Historical provider restatements must not silently replace the snapshot used to create the prediction.

### Macro inputs

For a decision in calendar month `M`, the newest macro month permitted is `M-1`.

The macro block remains:

- 10-year Treasury yield level
- 2-year Treasury yield level
- 10y minus 2y curve slope
- 3-month change in 10-year yield
- 3-month change in 2-year yield
- 3-month change in curve slope
- 3-month and 12-month WTI log return
- 3-month and 12-month broad-dollar log return

Each weekly macro snapshot is archived and hashed because FRED histories can be revised.

## Immutable prediction ledger

Every official weekly run records, before any target outcome is known:

- prediction creation timestamp
- decision week
- market snapshot hash
- macro snapshot hash
- frozen model hash
- model name and horizon
- sector
- score and within-week rank
- target maturity date

An existing prediction key `(decision_week, model, horizon, sector)` is never overwritten. Corrections require a new record with an explicit correction status and reason; the original remains preserved.

Outcome fields stay blank until the horizon matures.

## Outcome snapshot rule

At target maturity, compute the return from a newly archived maturity-date market snapshot, using both endpoint adjusted prices from that same maturity snapshot. This prevents later corporate-action restatements from silently rewriting the scored outcome. The maturity snapshot and outcome calculation are hashed and retained.

## Primary confirmatory endpoint

The preselected primary horizon is **4 weeks**. It is selected for prospective confirmation after Phase 3 discovery and is not an untouched historical choice.

Primary comparison:

`delta_rank_IC = weekly_rank_IC(candidate) - weekly_rank_IC(baseline)`

Primary claim bar after at least 52 official weekly decisions and after their 4-week outcomes have matured:

1. candidate 4-week mean weekly Spearman rank IC is positive with a 95% moving-block bootstrap interval whose lower bound is above zero; and
2. candidate-minus-baseline 4-week rank-IC improvement is positive with a 95% paired moving-block bootstrap interval whose lower bound is above zero.

The bootstrap resamples complete decision weeks with block length 4 or longer.

If this bar is not met, Phase 3's macro-sector result is not prospectively confirmed, even if another horizon looks attractive after the fact.

## Secondary endpoints

Always report all pre-frozen horizons: 1, 2, 4, 8 and 12 weeks.

Secondary metrics:

- mean weekly Spearman rank IC
- mean weekly top-3 minus bottom-3 future relative-return spread
- candidate-minus-baseline paired differences
- pooled ROC AUC
- balanced accuracy

Secondary horizons can describe generalization but cannot rescue a failed 4-week primary endpoint without a separately registered future study.

## Timing and interim views

- Minimum confirmatory sample: 52 official weekly decisions.
- Primary 4-week evaluation occurs only after all 52 primary outcomes mature.
- Full 12-week secondary evaluation waits until the corresponding outcomes mature.
- Interim tables may be inspected for pipeline health, but they remain descriptive and do not trigger model changes.

## No trading claim yet

Phase 4 tests predictive ranking, not profitability. No new portfolio rule, execution rule, transaction-cost optimization, or investable claim is introduced during the confirmation window. Economic-value research starts only after the predictive endpoint is evaluated.

## Interpretation rule

Before prospective confirmation, the strongest allowed wording remains:

> Exploratory development evidence suggests that lagged macro regimes may improve cross-sectional sector ranking. A genuine forward-looking edge has not yet been confirmed.

A successful Phase 4 primary endpoint would support a narrower claim of prospective 4-week sector-ranking evidence. It would still not establish exact price prediction or guaranteed trading alpha.
