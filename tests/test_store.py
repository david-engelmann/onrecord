"""OR-5 to OR-12. The claim store."""
import json
from pathlib import Path

import pytest

from onrecord.cli import main
from onrecord.digest import digest_of
from onrecord.store import Store, StoreError

GOLDEN = "cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3"
FIX = Path(__file__).parent / "fixtures"


def add(store, **kw):
    base = {"text": "The staging deploy is broken",
            "asserter": "agent-a",
            "asserted_at": "2026-01-01T09:00:00Z",
            "evidence": [{"source": "trace",
                          "reference": "run-17"}]}
    base.update(kw)
    return store.add(**base)


def test_the_digest_is_pinned_to_the_family_golden():
    tape = json.loads((FIX / "tape_synthetic.json").read_text())
    assert digest_of(tape) == GOLDEN


def test_add_appends_one_line_and_the_id_is_deterministic(tmp_path):
    path = tmp_path / "record.jsonl"
    first = add(Store(path))
    assert first.startswith("claim-")
    assert len(path.read_text().splitlines()) == 1
    again = add(Store(tmp_path / "other.jsonl"))
    assert again == first
    second = add(Store(path), text="A different claim")
    assert second != first
    assert len(path.read_text().splitlines()) == 2


def test_the_store_never_invents_a_time(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    with pytest.raises(StoreError):
        add(store, asserted_at="last Tuesday")
    assert not (tmp_path / "r.jsonl").exists()


def test_supersession_refusals_are_named(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    with pytest.raises(StoreError, match="names no claim"):
        add(store, supersedes="claim-missing")
    first = add(store)
    second = add(store, text="The staging deploy is fixed",
                 supersedes=first)
    assert second != first
    with pytest.raises(StoreError, match="already been superseded"):
        add(store, text="A fork attempt", supersedes=first)


def test_get_returns_the_claim_and_its_chain_oldest_first(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    first = add(store)
    second = add(store, text="v2", supersedes=first)
    third = add(store, text="v3", supersedes=second)
    document = store.get(third)
    assert document["claim"]["id"] == third
    assert [c["id"] for c in document["chain"]] == [first, second, third]
    middle = store.get(second)
    assert [c["id"] for c in middle["chain"]] == [first, second]
    with pytest.raises(StoreError, match="no claim with id"):
        store.get("claim-nope")


def test_subject_retrieval_and_the_conflict_answer(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    add(store, subject="deploy")
    single = store.subject("deploy")
    assert single["claim"]["subject"] == "deploy"
    add(store, text="An independent reading", subject="deploy")
    conflict = store.subject("deploy")
    assert conflict["conflict"] is True
    assert conflict["subject"] == "deploy"
    assert len(conflict["claims"]) == 2
    assert all("chain" in c for c in conflict["claims"])
    with pytest.raises(StoreError, match="no live claim"):
        store.subject("nothing-here")


def test_superseding_one_side_resolves_the_conflict(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    first = add(store, subject="deploy")
    add(store, text="An independent reading", subject="deploy")
    add(store, text="The merged reading", supersedes=first,
        subject="deploy")
    conflict = store.subject("deploy")
    # Two live claims remain (the independent reading and the
    # merged one); the conflict is still the answer, and the
    # superseded claim is in neither live chain head.
    assert conflict["conflict"] is True
    heads = {c["claim"]["text"] for c in conflict["claims"]}
    assert heads == {"An independent reading", "The merged reading"}


def test_due_dates_are_stored_and_retrieved(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    cid = add(store, due="2026-02-01")
    assert store.get(cid)["claim"]["due"] == "2026-02-01"
    with pytest.raises(StoreError, match="not an ISO date"):
        add(store, due="next Friday")


def test_check_names_every_defect(tmp_path):
    path = tmp_path / "r.jsonl"
    good = add(Store(path))
    lines = path.read_text().splitlines()
    lines.append('{"id": "claim-x", "text": 5}')
    lines.append("not json at all")
    lines.append(json.dumps({
        "id": "claim-y", "text": "t", "asserter": "a",
        "asserted_at": "2026-01-01T00:00:00Z", "evidence": [],
        "supersedes": "claim-ghost"}))
    path.write_text("\n".join(lines) + "\n")
    defects = Store(path).check()
    joined = "\n".join(defects)
    assert "line 2" in joined and "text" in joined
    assert "line 3: not JSON" in joined
    assert "claim-ghost" in joined
    assert "id does not match its derivation" in joined


def test_a_crash_mid_append_is_a_detectable_final_line(tmp_path):
    path = tmp_path / "r.jsonl"
    add(Store(path))
    with open(path, "a", encoding="utf-8") as handle:
        handle.write('{"id": "claim-partial", "text": "hal')
    defects = Store(path).check()
    assert any("not JSON" in d for d in defects)
    with pytest.raises(StoreError, match="defects"):
        add(Store(path), text="Cannot append to a broken record")


def test_a_fork_in_the_file_is_a_defect(tmp_path):
    path = tmp_path / "r.jsonl"
    first = add(Store(path))
    add(Store(path), text="v2", supersedes=first)
    # A second supersession of the same claim, written by hand
    # as a foreign writer could, is a fork at read time. Its id
    # cannot match the derivation, which check also names.
    lines = path.read_text().splitlines()
    lines.append(json.dumps({
        "id": "claim-fork", "text": "fork", "asserter": "b",
        "asserted_at": "2026-01-01T10:00:00Z", "evidence": [],
        "supersedes": first}))
    path.write_text("\n".join(lines) + "\n")
    defects = Store(path).check()
    assert any("a fork" in d for d in defects)


def test_the_cli_round_trip(tmp_path, capsys):
    path = str(tmp_path / "r.jsonl")
    assert main(["add", path, "--text", "The deploy is broken",
                 "--asserter", "agent-a",
                 "--at", "2026-01-01T09:00:00Z",
                 "--evidence", "trace:run-17",
                 "--subject", "deploy"]) == 0
    cid = capsys.readouterr().out.strip()
    assert main(["get", path, cid]) == 0
    document = json.loads(capsys.readouterr().out)
    assert document["claim"]["id"] == cid
    assert main(["subject", path, "deploy"]) == 0
    assert main(["check", path]) == 0
    assert main(["add", path, "--text", "x", "--asserter", "a",
                 "--at", "2026-01-01T09:00:00Z",
                 "--evidence", "nocolon"]) == 1
    assert "source and a reference" in capsys.readouterr().err
