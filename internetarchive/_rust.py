"""Optional Rust extension bridge for the Python package."""

from __future__ import annotations

try:
    import internetarchive_rust as _rust  # type: ignore
except ImportError:  # pragma: no cover - extension is optional in Python tests
    _rust = None

RUST_AVAILABLE = _rust is not None


def validate_s3_identifier(identifier: str) -> bool:
    """Validate an identifier with the Rust extension when available."""
    if not _rust:
        raise ImportError("Rust extension is not available")
    _rust.validate_s3_identifier(identifier)
    return True


def sanitize_windows_filename(name: str) -> tuple[str, bool]:
    """Sanitize a filename with the Rust extension when available."""
    if not _rust:
        raise ImportError("Rust extension is not available")
    return _rust.sanitize_windows_filename(name)
