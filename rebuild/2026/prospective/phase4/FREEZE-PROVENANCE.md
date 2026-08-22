# Phase 4 freeze provenance

The Phase 4 fixed models were created from the same local 2026 market archive and Phase 3 contract used for the exploratory study.

- Market archive: `HKSI-RRG-2026-market-data.zip`
  - SHA-256: `3838be08d18b238675ea02b9addce983799820abdf74eb40ac0e4ce8481b82bf`
- Phase 3 final pre-run contract:
  - SHA-256: `5354e282c7804cd9ee21b28b08298daccbe6c663e63f2093588a325a2b7cc1a3`
- Saved Phase 3 OOS predictions:
  - SHA-256: `594dae9f752b1d8b195651a62cb6c9fd23579edda8af7a81e3bf58e238bcd71d`
- Phase 4 macro snapshot through 2026-07:
  - SHA-256: `245ee0ac05d3ad045958f46693f89182770db8237ae23a7662c59eaa6bd900af`

The A/B Phase 3 prediction scores were independently replayed by `build_phase4_freeze.py` and matched the saved OOS scores to less than `1e-12` absolute difference before fixed Phase 4 coefficients were written.

The Phase 3 RRG-augmented sidecar is not part of the Phase 4 predictive freeze. Its replay differences were numerical and below `2e-6`; it is retained only in the reproduction diagnostic.
