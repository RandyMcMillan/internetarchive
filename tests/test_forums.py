import json
from pathlib import Path

import pytest
import responses

from internetarchive import forums, get_session
from internetarchive.exceptions import ForumError, ForumNotFoundError

DATA = Path(__file__).parent / "data" / "forums"
OFFSHOOT_URL = "https://archive.org/services/offshoot/forum-posts.php"
TEST_CONFIG = str(Path(__file__).parent / "ia.ini")


def _session(cookies=True):
    config = {}
    if cookies:
        config["cookies"] = {
            "logged-in-user": "test%40example.com",
            "logged-in-sig": "test-sig",
        }
    return get_session(config=config, config_file=TEST_CONFIG)


def _backend(cookies=True):
    return forums._IathreadsBackend(_session(cookies=cookies))


def _listing_html():
    return json.loads((DATA / "offshoot_listing.json").read_text())["value"]["html"]


def test_parse_date():
    d = forums._parse_date("Jul 23, 2026 7:26am")
    assert d.isoformat() == "2026-07-23T07:26:00"
    d = forums._parse_date(" Dec 1, 2025 11:59pm ")
    assert d.isoformat() == "2025-12-01T23:59:00"


def test_parse_listing_groups_threads():
    threads = forums._parse_listing(_listing_html())
    assert [t.id for t in threads] == ["2445301", "2445295"]
    t = threads[0]
    assert t.subject == "First and Last GD Song You Saw"
    assert t.poster == "wlg3"
    assert t.replies == 1
    assert t.date.isoformat() == "2026-07-23T07:26:00"
    # last_post_date comes from the indented "Re:" row that follows the root
    assert t.last_post_date > t.date


def test_parse_listing_unescapes_entities():
    threads = forums._parse_listing(_listing_html())
    assert threads[1].subject == "BERTHA don’t you come a round here"


def test_parse_thread_posts():
    thread = forums._parse_thread((DATA / "thread.html").read_text())
    assert thread.id == "2445301"
    assert thread.forum_id == "GratefulDead"
    assert thread.subject == "First and Last GD Song You Saw"
    assert len(thread.posts) == 2
    root, reply = thread.posts
    assert (root.id, root.depth, root.parent_id) == ("2445301", 0, None)
    assert (reply.id, reply.depth, reply.parent_id) == ("2445303", 1, "2445301")
    assert root.thread_id == "2445301"
    assert root.poster == "wlg3"
    assert root.date.isoformat() == "2026-07-23T07:26:00"
    assert root.subject == "First and Last GD Song You Saw"
    assert "Days Between" in root.body
    assert "<br" not in root.body
    assert "<br" in root.body_html


def test_parse_thread_body_newlines_and_entities():
    thread = forums._parse_thread((DATA / "thread.html").read_text())
    root = thread.posts[0]
    # <br /><br /> becomes newlines; &amp; is unescaped
    assert "\n" in root.body
    assert "Dead & Co" in root.body


@responses.activate
def test_backend_exists():
    responses.add(
        responses.GET,
        OFFSHOOT_URL,
        json={"success": True, "value": {"exists": True, "html": ""}},
    )
    assert _backend().exists("GratefulDead") is True


@responses.activate
def test_backend_exists_false():
    responses.add(
        responses.GET,
        OFFSHOOT_URL,
        json={"success": True, "value": {"exists": False, "html": None}},
    )
    assert _backend().exists("nosuchforum") is False


@responses.activate
def test_backend_offshoot_envelope_error():
    responses.add(
        responses.GET,
        OFFSHOOT_URL,
        json={"success": False, "error": '"forum_id" not found'},
    )
    with pytest.raises(ForumError, match="forum_id"):
        _backend().list_posts("GratefulDead")


@responses.activate
def test_backend_list_posts():
    responses.add(
        responses.GET,
        OFFSHOOT_URL,
        body=(DATA / "offshoot_listing.json").read_text(),
        content_type="application/json",
    )
    threads = _backend().list_posts("GratefulDead")
    assert [t.id for t in threads] == ["2445301", "2445295"]


@responses.activate
def test_backend_list_posts_not_found():
    responses.add(
        responses.GET,
        OFFSHOOT_URL,
        json={"success": True, "value": {"exists": False, "html": None}},
    )
    with pytest.raises(ForumNotFoundError, match="ia forum create"):
        _backend().list_posts("nosuchforum")


@responses.activate
def test_backend_list_posts_format_changed():
    responses.add(
        responses.GET,
        OFFSHOOT_URL,
        json={
            "success": True,
            "value": {"exists": True, "html": "<div>redesigned!</div>"},
        },
    )
    with pytest.raises(ForumError, match="format may have changed"):
        _backend().list_posts("GratefulDead")


@responses.activate
def test_backend_get_thread():
    responses.add(
        responses.GET,
        "https://archive.org/post/2445301",
        body=(DATA / "thread.html").read_text(),
    )
    thread = _backend().get_thread("2445301")
    assert thread.id == "2445301"
    assert thread.forum_id == "GratefulDead"
    assert len(thread.posts) == 2


@responses.activate
def test_backend_get_thread_not_found():
    responses.add(responses.GET, "https://archive.org/post/999999999", status=404)
    with pytest.raises(ForumNotFoundError, match="999999999"):
        _backend().get_thread("999999999")


@responses.activate
def test_backend_get_thread_format_changed():
    responses.add(
        responses.GET,
        "https://archive.org/post/2445301",
        body="<html><body>redesigned!</body></html>",
    )
    with pytest.raises(ForumError, match="format may have changed"):
        _backend().get_thread("2445301")
