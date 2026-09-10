import pytest

from px_sdk import parse_uri, uri_identity


def test_px_uri_round_trip_is_canonical() -> None:
    assert uri_identity("px://toystory/character/woody") == "px://toystory/character/woody"
    assert parse_uri("px://toystory/character/woody")["repository"] == "toystory"


def test_legacy_uri_input_is_canonicalized() -> None:
    assert uri_identity("nap://toystory/character/woody") == "px://toystory/character/woody"


def test_unsupported_uri_scheme_is_rejected() -> None:
    with pytest.raises(Exception, match="unsupported URI scheme"):
        parse_uri("other://toystory/character/woody")
