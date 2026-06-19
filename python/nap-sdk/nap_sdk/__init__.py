from __future__ import annotations

import json
import os
from typing import Any, cast

from . import _native


def parse_uri(uri: str) -> dict[str, Any]:
    return cast(dict[str, Any], json.loads(_native.parse_uri(uri)))


def parse_manifest(yaml_str: str) -> dict[str, Any]:
    return cast(dict[str, Any], json.loads(_native.parse_manifest(yaml_str)))


def resolve(uri: str, repo_path: str | None = None) -> dict[str, Any]:
    if repo_path is None:
        repo_path = os.environ.get("NAP_DIR", "~/.nap")
    repo_path = os.path.expanduser(repo_path)
    return cast(dict[str, Any], json.loads(_native.resolve(uri, repo_path)))


def ingest_media(data: bytes, format: str) -> str:
    """Ingest raw media bytes into the content-addressed storage engine.

    The storage backend is determined by the ``NAP_STORAGE_BACKEND``
    environment variable at the Rust layer (``local`` or ``s3``).

    Args:
        data: Raw bytes of the media asset (image, audio, mesh, etc.).
        format: File extension without a leading dot (e.g. ``"png"``,
            ``"jpg"``, ``"wav"``, ``"glb"``).

    Returns:
        The content-addressed hash ``sha256:<hex>``.
    """
    return _native.ingest_media(data, format)


def version() -> str:
    return _native.version()


__all__ = [
    "parse_uri",
    "parse_manifest",
    "resolve",
    "ingest_media",
    "version",
]
