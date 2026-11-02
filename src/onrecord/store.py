"""OR-5 to OR-12. The append-only claim store."""
from __future__ import annotations

import json
from datetime import date, datetime
from pathlib import Path

from .digest import DigestError, digest_of

REQUIRED = ("id", "text", "asserter", "asserted_at",
            "evidence", "supersedes")


class StoreError(ValueError):
    """A named refusal."""


def derive_id(claim_without_id: dict, position: int) -> str:
    payload = {"claim": claim_without_id, "position": position}
    return "claim-" + digest_of(payload)[:16]


def _is_iso_timestamp(value) -> bool:
    if not isinstance(value, str):
        return False
    try:
        datetime.fromisoformat(value.replace("Z", "+00:00"))
        return True
    except ValueError:
        return False


def _is_iso_date(value) -> bool:
    if not isinstance(value, str):
        return False
    try:
        date.fromisoformat(value)
        return True
    except ValueError:
        return False


def claim_defects(claim, line_no: int, position: int) -> list[str]:
    """Shape defects of one parsed line (OR-5, OR-5a, OR-11a).
    Every defect is named; one bad field never hides another."""
    where = f"line {line_no}"
    if not isinstance(claim, dict):
        return [f"{where}: not a claim object"]
    defects = []
    for field in REQUIRED:
        if field not in claim:
            defects.append(f"{where}: missing field {field}")
    if "id" in claim and (not isinstance(claim["id"], str)
                          or not claim["id"]):
        defects.append(f"{where}: id must be a non-empty string")
    if "text" in claim and (not isinstance(claim["text"], str)
                            or not claim["text"]):
        defects.append(f"{where}: text must be a non-empty string")
    if "asserter" in claim and (
            not isinstance(claim["asserter"], str)
            or not claim["asserter"]):
        defects.append(
            f"{where}: asserter must be a non-empty string")
    if "asserted_at" in claim \
            and not _is_iso_timestamp(claim["asserted_at"]):
        defects.append(f"{where}: asserted_at is not an ISO timestamp")
    if "evidence" in claim:
        evidence = claim["evidence"]
        if not isinstance(evidence, list):
            defects.append(f"{where}: evidence must be a list")
        else:
            for i, entry in enumerate(evidence):
                if not isinstance(entry, dict) \
                        or not isinstance(entry.get("source"), str) \
                        or not isinstance(entry.get("reference"), str):
                    defects.append(
                        f"{where}: evidence[{i}] must name a source "
                        f"and a reference")
    if "supersedes" in claim and claim["supersedes"] is not None \
            and not isinstance(claim["supersedes"], str):
        defects.append(f"{where}: supersedes must be an id or null")
    if "subject" in claim and claim["subject"] is not None \
            and not isinstance(claim["subject"], str):
        defects.append(f"{where}: subject must be a string")
    if "due" in claim and claim["due"] is not None \
            and not _is_iso_date(claim["due"]):
        defects.append(f"{where}: due is not an ISO date")
    if all(field in claim for field in REQUIRED) \
            and isinstance(claim.get("id"), str):
        without = {k: v for k, v in claim.items() if k != "id"}
        try:
            expected = derive_id(without, position)
        except (DigestError, RecursionError):
            expected = None
        if expected is not None and claim["id"] != expected:
            defects.append(
                f"{where}: id does not match its derivation")
    return defects


def load_record(path: Path):
    """Return (claims, defects): every claim in order and
    every defect in the record, line numbers attached."""
    claims: list[dict] = []
    defects: list[str] = []
    try:
        raw = path.read_bytes()
    except OSError:
        return [], [f"cannot read the record, {path}"]
    if not raw:
        return [], []
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError:
        return [], ["the record is not valid UTF-8"]
    lines = text.split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    parsed: list = []
    for index, line in enumerate(lines):
        line_no = index + 1
        if not line.strip():
            defects.append(f"line {line_no}: blank line")
            parsed.append(None)
            continue
        try:
            claim = json.loads(line)
        except ValueError:
            defects.append(f"line {line_no}: not JSON")
            parsed.append(None)
            continue
        parsed.append(claim)
        defects.extend(claim_defects(claim, line_no, index))
    # Chain level defects need the whole record.
    claims = [c for c in parsed if isinstance(c, dict)]
    ids = {}
    for claim in claims:
        cid = claim.get("id")
        if isinstance(cid, str):
            if cid in ids:
                defects.append(f"id {cid} appears twice")
            ids[cid] = claim
    superseded_by: dict[str, list[str]] = {}
    for claim in claims:
        target = claim.get("supersedes")
        if isinstance(target, str):
            if target not in ids:
                defects.append(
                    f"claim {claim.get('id')}: supersedes names "
                    f"no claim in the record, {target}")
            superseded_by.setdefault(target, []).append(
                claim.get("id"))
    for target, sources in superseded_by.items():
        if len(sources) > 1:
            defects.append(
                f"claim {target} is superseded by two claims, "
                f"{sources[0]} and {sources[1]}, a fork")
    return claims, defects


def _require_sound(claims, defects) -> None:
    if defects:
        raise StoreError("the record has defects: "
                         + "; ".join(defects))


class Store:
    def __init__(self, path):
        self.path = Path(path)

    def _load(self):
        claims, defects = load_record(self.path) \
            if self.path.exists() else ([], [])
        _require_sound(claims, defects)
        return claims

    def add(self, *, text, asserter, asserted_at, evidence,
            supersedes=None, subject=None, due=None,
            extra=None) -> str:
        claims = self._load()
        if not _is_iso_timestamp(asserted_at):
            raise StoreError(
                f"asserted_at is not an ISO timestamp, {asserted_at}")
        if due is not None and not _is_iso_date(due):
            raise StoreError(f"due is not an ISO date, {due}")
        if supersedes is not None:
            by_id = {c["id"]: c for c in claims}
            if supersedes not in by_id:
                raise StoreError(
                    f"supersedes names no claim in the record, "
                    f"{supersedes}")
            if any(c.get("supersedes") == supersedes for c in claims):
                raise StoreError(
                    f"claim {supersedes} has already been superseded")
        claim = {"text": text, "asserter": asserter,
                 "asserted_at": asserted_at, "evidence": evidence,
                 "supersedes": supersedes}
        if subject is not None:
            claim["subject"] = subject
        if due is not None:
            claim["due"] = due
        if extra:
            for key, value in extra.items():
                if key not in claim and key != "id":
                    claim[key] = value
        position = len(claims)
        claim_id = derive_id(claim, position)
        stored = {"id": claim_id, **claim}
        line = json.dumps(stored, ensure_ascii=False,
                          separators=(",", ":")) + "\n"
        with open(self.path, "a", encoding="utf-8") as handle:
            handle.write(line)
            handle.flush()
        return claim_id

    def _chain(self, claims, claim) -> list[dict]:
        by_id = {c["id"]: c for c in claims}
        chain = [claim]
        seen = {claim["id"]}
        current = claim
        while current.get("supersedes") is not None:
            parent = by_id[current["supersedes"]]
            if parent["id"] in seen:
                break
            chain.append(parent)
            seen.add(parent["id"])
            current = parent
        chain.reverse()
        return chain

    def get(self, claim_id: str) -> dict:
        claims = self._load()
        by_id = {c["id"]: c for c in claims}
        if claim_id not in by_id:
            raise StoreError(f"no claim with id {claim_id}")
        claim = by_id[claim_id]
        return {"claim": claim, "chain": self._chain(claims, claim)}

    def subject(self, key: str) -> dict:
        claims = self._load()
        superseded = {c["supersedes"] for c in claims
                      if c.get("supersedes")}
        live = [c for c in claims
                if c.get("subject") == key
                and c["id"] not in superseded]
        if not live:
            raise StoreError(f"no live claim on subject {key}")
        if len(live) == 1:
            return {"claim": live[0],
                    "chain": self._chain(claims, live[0])}
        return {"conflict": True, "subject": key,
                "claims": [{"claim": c,
                            "chain": self._chain(claims, c)}
                           for c in live]}

    def check(self) -> list[str]:
        _, defects = load_record(self.path) \
            if self.path.exists() else ([], [])
        return defects
