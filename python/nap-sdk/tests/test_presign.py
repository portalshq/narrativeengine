from __future__ import annotations

import json

import nap_sdk


def test_presign_representation_forwards_options(monkeypatch) -> None:
    captured: tuple[object, ...] = ()

    def fake_presign(*args: object) -> str:
        nonlocal captured
        captured = args
        return json.dumps(
            {
                "url": "http://localhost/file?token=opaque",
                "expires_at": 123,
                "revision": "rev",
                "repository_id": "repo",
                "address": "address",
                "representation": "reference_image",
                "format": "png",
            }
        )

    monkeypatch.setattr(nap_sdk._native, "presign_representation", fake_presign)
    result = nap_sdk.presign_representation(
        "nap://test/character/hero",
        "reference_image",
        repo_path="/tmp/nap",
        branch="main",
        ttl_seconds=90,
        http_url="http://127.0.0.1:41339",
        bearer_token="secret",
    )

    assert result["expires_at"] == 123
    assert captured == (
        "nap://test/character/hero",
        "reference_image",
        "/tmp/nap",
        "main",
        None,
        90,
        "http://127.0.0.1:41339",
        "secret",
    )
