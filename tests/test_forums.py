import json
from pathlib import Path

from internetarchive import forums

DATA = Path(__file__).parent / "data" / "forums"


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
