import pytest

from rrg_rebuild.validation import WalkForwardConfig, generate_walk_forward_folds


def test_walk_forward_validation_windows_are_non_overlapping() -> None:
    config = WalkForwardConfig(
        initial_train_size=20,
        validation_size=5,
        gap=2,
        expanding=True,
    )
    folds = generate_walk_forward_folds(42, config)

    assert [(f.train_start, f.train_end, f.validation_start, f.validation_end) for f in folds] == [
        (0, 20, 22, 27),
        (0, 25, 27, 32),
        (0, 30, 32, 37),
        (0, 35, 37, 42),
    ]
    for previous, current in zip(folds, folds[1:], strict=False):
        assert previous.validation_end <= current.validation_start


def test_fixed_window_moves_training_start_forward() -> None:
    config = WalkForwardConfig(12, 4, gap=1, expanding=False)
    folds = generate_walk_forward_folds(30, config)
    assert folds[0].train_start == 0
    assert folds[1].train_start == 4
    assert folds[1].train_end == 16


def test_requires_enough_rows_for_complete_fold() -> None:
    with pytest.raises(ValueError, match="not enough samples"):
        generate_walk_forward_folds(10, WalkForwardConfig(8, 3, gap=1))
