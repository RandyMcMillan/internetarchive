# Copilot instructions for `internetarchive`

## Project overview

This repository is a Python library plus the `ia` CLI for Archive.org. The main
user-facing API lives in `internetarchive/api.py`, while the heavier lifting is
in `session.py`, `item.py`, `files.py`, and `search.py`. The CLI entrypoint is
`internetarchive/cli/ia.py`, which wires subcommands from `internetarchive/cli/`.

## Build, test, and lint

- Install for development: `pip install -e '.[all]'`
- Run the repo’s check target: `make test`
- Run the full local check set: `ruff check && ruff format --check && pytest`
- Run a single test file: `pytest tests/test_api.py`
- Run a single test: `pytest tests/test_api.py::test_get_item`
- Run the full multi-version suite: `tox`
- Build docs with the Make target: `make docs`
- Build docs: `pip install -r docs/requirements.txt && cd docs && make html`

Test selection notes:

- Networked tests are marked `network`; use `pytest -m 'not network'` when you
  want to skip live Archive.org calls.
- CLI tests use `tests/ia.ini` and the `responses` mock library for HTTP
  isolation.

## Architecture

- `internetarchive.api` exposes convenience wrappers such as `get_item()`,
  `search_items()`, `upload()`, `download()`, `modify_metadata()`, and
  `get_session()`.
- `ArchiveSession` in `session.py` owns config loading, auth state, headers,
  retries, and request/session behavior. Most higher-level operations are
  reached through a session.
- `Item` in `item.py` models Archive.org items and collections, including
  metadata loading, downloads, uploads, and metadata edits.
- `File` in `files.py` models a single file inside an item and handles download
  and delete operations.
- `Search` in `search.py` handles advanced search, scrape, and full-text search
  flows.
- `config.py`, `auth.py`, `iarequest.py`, and `catalog.py` support config,
  authentication, request construction, and catalog task APIs.
- `internetarchive/cli/ia.py` is the CLI entrypoint and dispatches to the
  subcommand modules under `internetarchive/cli/`.

## Key conventions

- Preserve the project’s formatting style: 88-character lines, Ruff linting,
  and Ruff formatting with quote normalization disabled.
- Keep docstrings in Sphinx style with `:param:`, `:returns:`, and
  `:raises:` sections when you touch public APIs.
- Configuration is merged from file, environment, and explicit config. The
  config file search order is `IA_CONFIG_FILE`, XDG config, `~/.config/ia.ini`,
  then `~/.ia`. `IA_ACCESS_KEY_ID` and `IA_SECRET_ACCESS_KEY` must both be set
  together.
- Versions should be development suffixed outside releases (for example,
  `5.7.3.dev0`).
- New user-facing behavior should include docs updates under `docs/source/`.
- Avoid introducing new dependencies unless absolutely necessary.
