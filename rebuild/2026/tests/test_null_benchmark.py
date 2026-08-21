import numpy as np

from rrg_rebuild.null_benchmark import (
    SyntheticNullConfig,
    run_synthetic_null_benchmark,
    simulate_no_signal_prices,
    summarize_synthetic_null,
)


def test_synthetic_null_prices_are_deterministic_and_positive():
    config = SyntheticNullConfig(
        n_weeks=180,
        n_sectors=2,
        n_trials=1,
        holdout_weeks=20,
    )
    benchmark_a, sectors_a = simulate_no_signal_prices(seed=7, config=config)
    benchmark_b, sectors_b = simulate_no_signal_prices(seed=7, config=config)

    np.testing.assert_allclose(benchmark_a.to_numpy(), benchmark_b.to_numpy())
    assert (benchmark_a > 0).all()
    assert sectors_a.keys() == sectors_b.keys()
    for symbol in sectors_a:
        np.testing.assert_allclose(
            sectors_a[symbol].to_numpy(), sectors_b[symbol].to_numpy()
        )
        assert (sectors_a[symbol] > 0).all()


def test_null_benchmark_returns_trial_horizon_records():
    config = SyntheticNullConfig(
        n_weeks=220,
        n_sectors=2,
        n_trials=2,
        holdout_weeks=20,
        horizons=(1, 2),
        lookback=10,
    )
    results = run_synthetic_null_benchmark(config)
    assert len(results) == 4
    assert set(results["horizon"]) == {1, 2}
    assert set(results["trial"]) == {0, 1}
    assert (results["n"] > 0).all()

    summary = summarize_synthetic_null(results)
    assert set(summary["horizon"]) == {1, 2}
    assert (summary["trials"] == 2).all()
