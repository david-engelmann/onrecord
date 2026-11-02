"""OR-11. A thin command line over the store: add, get,
subject, check. Failures are named errors with exit 1."""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from .store import Store, StoreError


def _parse_evidence(items) -> list[dict]:
    evidence = []
    for item in items or []:
        source, sep, reference = item.partition(":")
        if not sep or not source or not reference:
            raise StoreError(
                f"evidence must name a source and a reference, "
                f"{item}")
        evidence.append({"source": source, "reference": reference})
    return evidence


def _emit_text(text: str) -> int:
    try:
        sys.stdout.write(text)
        sys.stdout.flush()
    except (OSError, UnicodeEncodeError):
        print("error, cannot write to stdout", file=sys.stderr)
        # The broken stream would fail again at interpreter
        # shutdown and mask this refusal with exit 120 and
        # an ignored-exception dump, so it is replaced
        # with a sink before returning.
        try:
            sys.stdout.close()
        except OSError:
            pass
        import os
        sys.stdout = open(os.devnull, "w", encoding="utf-8")
        return 1
    return 0


def _emit(document) -> int:
    text = json.dumps(document, indent=2, ensure_ascii=False) + "\n"
    return _emit_text(text)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="onrecord")
    sub = parser.add_subparsers(dest="command", required=True)
    add = sub.add_parser("add", help="append a claim")
    add.add_argument("record")
    add.add_argument("--text", required=True)
    add.add_argument("--asserter", required=True)
    add.add_argument("--at", required=True)
    add.add_argument("--evidence", action="append")
    add.add_argument("--supersedes")
    add.add_argument("--subject")
    add.add_argument("--due")
    get = sub.add_parser("get", help="retrieve a claim by id")
    get.add_argument("record")
    get.add_argument("id")
    subj = sub.add_parser("subject", help="retrieve a subject")
    subj.add_argument("record")
    subj.add_argument("key")
    chk = sub.add_parser("check", help="name every defect")
    chk.add_argument("record")
    args = parser.parse_args(argv)
    store = Store(Path(args.record))
    try:
        if args.command == "add":
            claim_id = store.add(
                text=args.text, asserter=args.asserter,
                asserted_at=args.at,
                evidence=_parse_evidence(args.evidence),
                supersedes=args.supersedes,
                subject=args.subject, due=args.due)
            return _emit_text(claim_id + "\n")
        if args.command == "get":
            return _emit(store.get(args.id))
        if args.command == "subject":
            return _emit(store.subject(args.key))
        if args.command == "check":
            defects = store.check()
            for defect in defects:
                print(defect)
            if defects:
                return 1
            print("the record is sound")
            return 0
    except StoreError as exc:
        print(f"error, {exc}", file=sys.stderr)
        return 1
    except OSError as exc:
        print(f"error, {exc}", file=sys.stderr)
        return 1
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
