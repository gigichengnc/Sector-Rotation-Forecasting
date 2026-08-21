from __future__ import annotations

import argparse
from pathlib import Path

from rrg_rebuild.null_benchmark import (
    SyntheticNullConfig,
    run_synthetic_null_benchmark,
    summarize_synthetic_null,
)


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Run the legacy RRG-coordinate forecast pipeline on no-signal "
            "synthetic data."
        )
    )
    parser.add_argument("--trials", type=int, default=12)
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("outputs/synthetic-null"),
    )
    args = parser.parse_args()

    config = SyntheticNullConfig(n_trials=args.trials)
    results = run_synthetic_null_benchmark(config)
    summary = summarize_synthetic_null(results)

    args.output_dir.mkdir(parents=True, exist_ok=True)
    results.to_csv(args.output_dir / "trial_results.csv", index=False)
    summary.to_csv(args.output_dir / "summary.csv", index=False)
    print(summary.to_string(index=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
