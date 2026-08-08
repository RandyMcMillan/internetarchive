import importlib
import sys
import types


def test_rust_bridge_uses_extension(monkeypatch):
    dummy = types.SimpleNamespace(
        validate_s3_identifier=lambda identifier: None,
        sanitize_windows_filename=lambda name: (f"rust:{name}", True),
    )
    monkeypatch.setitem(sys.modules, "internetarchive_rust", dummy)

    bridge = importlib.reload(importlib.import_module("internetarchive._rust"))

    assert bridge.RUST_AVAILABLE is True
    assert bridge.validate_s3_identifier("nasa") is True
    assert bridge.sanitize_windows_filename("name") == ("rust:name", True)
