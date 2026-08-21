from __future__ import annotations

import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE_ROOT = ROOT / "historical" / "audited-evidence"

# These byte counts and SHA-256 values refer to the recovered archive bytes,
# which used CRLF line endings. The public Git repository stores the same 12
# text files with LF line endings. Verification therefore canonicalizes the
# repository checkout to CRLF before comparing with the recovered-archive
# manifest. This tests content equivalence under the documented line-ending
# transformation; it is not a claim that the raw Git blob bytes are identical.
EXPECTED = {
    "README.md": (6381, "46fb1e63dac583d7cbaa4a32fd94059dcb7877df552f11eec82dbcbf7613046b"),
    "config.toml": (1718, "4356831f3531caf312c423b9b821dd9f030f8900fe24b21da0822d848ea458cf"),
    "crates/rrg-calc/src/calculator.rs": (22341, "afebb1e98531495ed833a281437ffd55e8d0cfa2b0a60fb5595b9a608dba251d"),
    "crates/rrg-calc/src/quadrant.rs": (13243, "bb0e5df0ae335a14f77dbf3b3981c177265271df77a0b359c29babf0f9a7dc87"),
    "crates/ml/src/lstm_predictor.rs": (21468, "8a01d6cb1d4c4e2648ab0253ff987fa35b7c2cead483057a7c881d8289807f9c"),
    "crates/ml/src/feature_engineering.rs": (40171, "56ea893b3331df6d0350b3e7e3f726c9ce52056c4420a28919b3b03b8c0bb6ed"),
    "crates/ml/src/training_pipeline.rs": (17526, "c547877e9115c8c0a598a24b5cb7ea284e9e81313a0bd08b09bd6efcaaeb876d"),
    "crates/ml/src/prediction_engine.rs": (26517, "536b6d7a3528799bf8c81dd94725a742f2cf594ddda193d8c33d0b03ce0b7fec"),
    "crates/ml/src/backtesting.rs": (25464, "023863d7727b691517872a6fd519f11af5fb6e381421eb1ade1be8874adccd4c"),
    "crates/web/Cargo.toml": (1043, "8c6ebc8c8a2329888e1e2d2929c2ee16448eccc6e2c05fa1d68f29cac45baf28"),
    "crates/web/src/api.rs": (43865, "a5661d8339e566102721c0ef7203d0df88c295143dea0b79d7f42980e40f37e9"),
    "app-standalone/app.js": (19602, "5325b05cad5599691e08e20608b7ac5593b19b9a34f792a9cc37130d568a91c1"),
}


def archive_crlf_bytes(data: bytes) -> bytes:
    """Convert text checkout bytes to the recovered archive's CRLF convention."""
    normalized = data.replace(b"\r\n", b"\n").replace(b"\r", b"\n")
    return normalized.replace(b"\n", b"\r\n")


def main() -> int:
    failures: list[str] = []
    for relative, (expected_size, expected_hash) in EXPECTED.items():
        path = EVIDENCE_ROOT / relative
        if not path.is_file():
            failures.append(f"missing: {relative}")
            continue

        canonical = archive_crlf_bytes(path.read_bytes())
        actual_size = len(canonical)
        actual_hash = hashlib.sha256(canonical).hexdigest()
        if actual_size != expected_size or actual_hash != expected_hash:
            failures.append(
                f"mismatch: {relative} size={actual_size}/{expected_size} "
                f"sha256={actual_hash}/{expected_hash}"
            )
        else:
            print(f"ok {relative} {actual_hash}")

    if failures:
        for failure in failures:
            print(failure)
        return 1

    print(f"verified {len(EXPECTED)}/{len(EXPECTED)} public evidence files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
