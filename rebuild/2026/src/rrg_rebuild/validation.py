from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class WalkForwardConfig:
    """Configuration for non-overlapping, time-ordered validation folds.

    All sizes are expressed in rows of the model-ready dataset. `gap` is the
    purged region between the last training row and the first validation row.
    For an h-step-ahead target, callers should use gap >= h so that the target
    attached to the last training row is known before validation begins.
    """

    initial_train_size: int
    validation_size: int
    gap: int = 0
    expanding: bool = True

    def __post_init__(self) -> None:
        if self.initial_train_size <= 0:
            raise ValueError("initial_train_size must be > 0")
        if self.validation_size <= 0:
            raise ValueError("validation_size must be > 0")
        if self.gap < 0:
            raise ValueError("gap must be >= 0")


@dataclass(frozen=True)
class WalkForwardFold:
    fold_id: int
    train_start: int
    train_end: int
    validation_start: int
    validation_end: int

    @property
    def train_slice(self) -> slice:
        return slice(self.train_start, self.train_end)

    @property
    def validation_slice(self) -> slice:
        return slice(self.validation_start, self.validation_end)

    def verify(self) -> None:
        if not (0 <= self.train_start < self.train_end):
            raise AssertionError("invalid training bounds")
        if self.train_end > self.validation_start:
            raise AssertionError("training and validation overlap")
        if self.validation_start >= self.validation_end:
            raise AssertionError("invalid validation bounds")


def generate_walk_forward_folds(
    n_samples: int,
    config: WalkForwardConfig,
) -> list[WalkForwardFold]:
    """Generate full, non-overlapping validation windows in chronological order.

    Partial final validation windows are deliberately omitted so fold metrics
    remain comparable. Validation windows never overlap, so an observation can
    be scored at most once for a given horizon/model evaluation run.
    """
    if n_samples <= 0:
        raise ValueError("n_samples must be > 0")

    folds: list[WalkForwardFold] = []
    train_end = config.initial_train_size
    fold_id = 0

    while True:
        validation_start = train_end + config.gap
        validation_end = validation_start + config.validation_size
        if validation_end > n_samples:
            break

        train_start = 0 if config.expanding else train_end - config.initial_train_size
        fold = WalkForwardFold(
            fold_id=fold_id,
            train_start=train_start,
            train_end=train_end,
            validation_start=validation_start,
            validation_end=validation_end,
        )
        fold.verify()
        folds.append(fold)

        train_end += config.validation_size
        fold_id += 1

    if not folds:
        raise ValueError("not enough samples for one complete walk-forward fold")

    for previous, current in zip(folds, folds[1:], strict=False):
        if previous.validation_end > current.validation_start:
            raise AssertionError("validation windows overlap")

    return folds
