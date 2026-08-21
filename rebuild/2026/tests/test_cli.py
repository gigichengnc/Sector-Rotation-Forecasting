from pathlib import Path
from types import SimpleNamespace

import pandas as pd

from rrg_rebuild import DeploymentConfig, RRGConfig, __version__
from rrg_rebuild import cli


def test_public_package_surface_has_version_and_core_types() -> None:
    assert __version__ == "0.1.0"
    assert RRGConfig().ratio_period == 10
    assert DeploymentConfig().horizons == (1, 2, 4, 8)


def test_cli_uses_generic_public_name() -> None:
    parser = cli.build_parser()
    assert parser.prog == "sector-rotation"


def test_cli_version(capsys) -> None:
    assert cli.main(["version"]) == 0
    assert capsys.readouterr().out.strip() == "0.1.0"


def test_cli_forecast_dispatches_without_network(monkeypatch, tmp_path: Path, capsys) -> None:
    output_dir = tmp_path / "forecast-run"
    table = pd.DataFrame(
        {
            "symbol": ["XLK"],
            "current_quadrant": ["Improving"],
            "1w_quadrant": ["Leading"],
        }
    )
    fake = SimpleNamespace(
        table=table,
        forecast_table_path=output_dir / "forecast_table.csv",
        forecast_long_path=output_dir / "forecast_long.csv",
        run_marker_path=output_dir / "RUN-MARKER.json",
        market_data=SimpleNamespace(archive_path=output_dir / "market-data.zip"),
    )
    calls: dict[str, object] = {}

    def fake_run(path: Path, **kwargs):
        calls["path"] = path
        calls.update(kwargs)
        return fake

    monkeypatch.setattr(cli, "run_fresh_forecast", fake_run)

    assert cli.main(
        [
            "forecast",
            "--output-dir",
            str(output_dir),
            "--start",
            "2020-01-01",
            "--end",
            "2026-01-01",
            "--sleep",
            "0",
        ]
    ) == 0

    assert calls["path"] == output_dir
    assert calls["start_date"] == "2020-01-01"
    assert calls["end_date"] == "2026-01-01"
    assert calls["sleep_seconds"] == 0.0
    output = capsys.readouterr().out
    assert "XLK" in output
    assert "does not estimate expected returns or issue trading signals" in output
