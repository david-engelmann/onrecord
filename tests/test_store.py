"""OR-5 to OR-16. The claim store and its due view."""
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


def test_superseding_one_side_leaves_the_other_claim_live(tmp_path):
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


def test_derive_id_pins_both_factors():
    from onrecord.store import derive_id
    claim = {"text": "x", "asserter": "a",
             "asserted_at": "2026-01-01T00:00:00Z",
             "evidence": [{"source": "s", "reference": "r"}],
             "supersedes": None}
    assert derive_id(claim, 0) != derive_id(claim, 1)
    changed = dict(claim, text="y")
    assert derive_id(claim, 0) != derive_id(changed, 0)


def test_twin_appends_get_distinct_ids_and_a_sound_record(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    first = add(store)
    second = add(store)
    assert first != second
    assert store.check() == []


def test_a_refused_add_leaves_the_file_byte_identical(tmp_path):
    path = tmp_path / "r.jsonl"
    store = Store(path)
    add(store)
    before = path.read_bytes()
    for kwargs in ({"text": ""}, {"asserter": ""},
                   {"evidence": []},
                   {"evidence": [{"source": "", "reference": "r"}]},
                   {"supersedes": "claim-ghost"}):
        with pytest.raises(StoreError):
            add(store, **kwargs)
    assert path.read_bytes() == before


def test_an_unterminated_final_line_is_completed_never_glued(tmp_path):
    path = tmp_path / "r.jsonl"
    store = Store(path)
    first = add(store)
    path.write_bytes(path.read_bytes().rstrip(b"\n"))
    assert store.check() == []
    second = add(store, text="The next claim")
    lines = path.read_text().splitlines()
    assert len(lines) == 2
    assert store.get(first)["claim"]["id"] == first
    assert store.get(second)["claim"]["id"] == second


def test_a_forged_id_hiding_behind_nan_is_a_named_defect(tmp_path):
    path = tmp_path / "r.jsonl"
    add(Store(path))
    forged = {"id": "claim-attackerchosen", "text": "forged",
              "asserter": "mallory",
              "asserted_at": "2026-01-01T00:00:00Z",
              "evidence": [{"source": "s", "reference": "r"}],
              "supersedes": None, "reading": float("nan")}
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(json.dumps(forged) + "\n")
    defects = Store(path).check()
    assert any("cannot be verified" in d for d in defects)
    with pytest.raises(StoreError, match="defects"):
        Store(path).get("claim-attackerchosen")


def test_check_names_deep_nesting_instead_of_dying(tmp_path):
    path = tmp_path / "r.jsonl"
    deep = "1"
    for _ in range(30000):
        deep = "[" + deep + "]"
    path.write_text('{"id": ' + deep + "}\n")
    defects = Store(path).check()
    assert any("nesting too deep" in d for d in defects)


def test_a_duplicate_key_in_a_line_is_a_defect(tmp_path):
    path = tmp_path / "r.jsonl"
    path.write_text('{"id": "claim-a", "id": "claim-b", '
                    '"text": "t"}\n')
    defects = Store(path).check()
    assert any("duplicate key" in d for d in defects)


def test_a_record_with_a_bom_reads(tmp_path):
    path = tmp_path / "r.jsonl"
    store = Store(path)
    cid = add(store)
    path.write_bytes(b"\xef\xbb\xbf" + path.read_bytes())
    assert Store(path).get(cid)["claim"]["id"] == cid


def test_check_on_a_missing_record_is_a_refusal(tmp_path, capsys):
    assert main(["check", str(tmp_path / "nope.jsonl")]) == 1
    assert "no such record" in capsys.readouterr().err


def test_a_chain_head_answers_for_its_chains_subject(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    first = add(store, subject="deploy")
    add(store, text="The head carries no subject",
        supersedes=first)
    document = store.subject("deploy")
    assert document["claim"]["text"] == "The head carries no subject"


def test_the_canonical_bytes_battery():
    from onrecord.digest import canonical_bytes
    cases = [
        ({"b": 1, "a": 2}, b'{"a":2,"b":1}'),
        ({"x": 1.5}, b'{"x":1.5}'),
        ({"x": 1e21}, b'{"x":1e+21}'),
        ({"x": 2 ** 60 + 1}, b'{"x":1152921504606846977}'),
        ({"\u00e9": 1, "e": 2}, b'{"e":2,"\xc3\xa9":1}'),
    ]
    for value, expected in cases:
        assert canonical_bytes(value) == expected


def test_the_due_view_answers_live_claims_by_date(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    early = add(store, text="Due early", due="2026-02-01")
    add(store, text="Due later", due="2026-03-01")
    add(store, text="No date at all")
    document = store.due("2026-02-01")
    assert document["as_of"] == "2026-02-01"
    assert [d["claim"]["id"] for d in document["due"]] == [early]
    assert document["due"][0]["chain"][0]["id"] == early
    assert "conflicts" not in document
    document = store.due("2026-01-01")
    assert document["due"] == []


def test_the_due_view_orders_by_date_then_position(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    third = add(store, text="Third", due="2026-02-10")
    first = add(store, text="First", due="2026-02-01")
    second = add(store, text="Second", due="2026-02-10")
    document = store.due("2026-02-10")
    assert [d["claim"]["id"] for d in document["due"]] == [
        first, third, second]


def test_a_superseded_claims_due_date_dies_with_it(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    old = add(store, text="Old promise", due="2026-02-01")
    new = add(store, text="New promise", supersedes=old,
              due="2026-04-01")
    document = store.due("2026-03-01")
    assert document["due"] == []
    document = store.due("2026-04-01")
    assert [d["claim"]["id"] for d in document["due"]] == [new]
    assert [c["id"] for c in document["due"][0]["chain"]] == [
        old, new]


def test_the_due_view_names_its_conflicts(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    add(store, text="Side one", subject="deploy",
        due="2026-02-01")
    add(store, text="Side two", subject="deploy",
        due="2026-02-01")
    add(store, text="Unrelated", subject="other",
        due="2026-02-01")
    document = store.due("2026-02-01")
    assert len(document["due"]) == 3
    assert document["conflicts"] == ["deploy"]


def test_the_due_view_refuses_a_bad_date_by_name(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    add(store, due="2026-02-01")
    for bad in ("next Friday", "2026-02-30",
                "2026-02-01T09:00:00Z", ""):
        with pytest.raises(StoreError, match="not an ISO date"):
            store.due(bad)


def test_the_due_command_prints_the_document(
        tmp_path, capsys):
    record = tmp_path / "r.jsonl"
    store = Store(record)
    dated = add(store, text="Dated", due="2026-02-01")
    add(store, text="Undated")
    assert main(["due", str(record), "2026-02-15"]) == 0
    document = json.loads(capsys.readouterr().out)
    assert document["as_of"] == "2026-02-15"
    assert [d["claim"]["id"] for d in document["due"]] == [dated]
    assert main(["due", str(record), "whenever"]) == 1
    assert "not an ISO date" in capsys.readouterr().err


def test_an_undated_successor_silences_the_chain(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    old = add(store, text="Promised", due="2026-02-01")
    add(store, text="Withdrawn", supersedes=old)
    assert store.due("2026-06-01")["due"] == []


def test_a_due_claims_document_is_gets_document(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    dated = add(store, text="Dated", due="2026-02-01")
    document = store.due("2026-02-01")
    assert document["due"] == [store.get(dated)]


def test_a_conflict_needs_both_sides_due(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    add(store, text="Side one", subject="deploy",
        due="2026-02-01")
    add(store, text="Side two", subject="deploy",
        due="2026-09-01")
    document = store.due("2026-03-01")
    assert len(document["due"]) == 1
    assert "conflicts" not in document


def test_inherited_subjects_conflict_in_the_due_view(
        tmp_path):
    store = Store(tmp_path / "r.jsonl")
    root = add(store, text="Root", subject="deploy")
    add(store, text="Head without a subject",
        supersedes=root, due="2026-02-01")
    add(store, text="Direct", subject="deploy",
        due="2026-02-01")
    document = store.due("2026-02-01")
    assert document["conflicts"] == ["deploy"]
    assert store.subject("deploy")["conflict"] is True


def test_answering_never_self_conflicts(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    root = add(store, text="Root", subject="deploy")
    mid = add(store, text="Mid", supersedes=root,
              subject="deploy")
    add(store, text="Head", supersedes=mid,
        due="2026-02-01")
    add(store, text="Quiet one", due="2026-02-01")
    add(store, text="Quiet two", due="2026-02-01")
    document = store.due("2026-02-01")
    assert len(document["due"]) == 3
    assert "conflicts" not in document


def test_a_head_with_its_own_subject_does_not_inherit(
        tmp_path):
    store = Store(tmp_path / "r.jsonl")
    root = add(store, text="Root", subject="deploy")
    add(store, text="Head on another subject",
        supersedes=root, subject="billing",
        due="2026-02-01")
    add(store, text="Direct", subject="deploy",
        due="2026-02-01")
    document = store.due("2026-02-01")
    assert "conflicts" not in document


def test_dates_are_exactly_one_form(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    for bad in ("20260201", "2026-W05-1", "2026-W05"):
        with pytest.raises(StoreError):
            add(store, due=bad)
        with pytest.raises(StoreError,
                           match="not an ISO date"):
            store.due(bad)
    dated = add(store, due="2026-02-01")
    document = store.due("2026-02-01")
    assert [d["claim"]["id"] for d in document["due"]] == [
        dated]


def test_the_due_view_refuses_a_missing_record(tmp_path):
    store = Store(tmp_path / "missing.jsonl")
    with pytest.raises(StoreError,
                       match="cannot read the record"):
        store.due("2026-02-01")
    with pytest.raises(StoreError,
                       match="cannot read the record"):
        Store(tmp_path).due("2026-02-01")
    assert main(["due", str(tmp_path / "missing.jsonl"),
                 "2026-02-01"]) == 1


def test_check_names_an_alien_due_date_at_read_time(
        tmp_path):
    from onrecord.store import derive_id
    path = tmp_path / "r.jsonl"
    claim = {"text": "Hand written",
             "asserter": "agent-a",
             "asserted_at": "2026-01-01T09:00:00Z",
             "evidence": [{"source": "trace",
                           "reference": "run-17"}],
             "supersedes": None, "due": "20260201"}
    stored = {"id": derive_id(claim, 0), **claim}
    path.write_text(json.dumps(stored) + "\n",
                    encoding="utf-8")
    store = Store(path)
    defects = store.check()
    assert any("due" in defect for defect in defects)
    with pytest.raises(StoreError):
        store.due("2026-12-31")
