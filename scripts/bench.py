#!/usr/bin/env python3
"""Reproducible scale benchmark harness for onrecord.

Usage:
    python3 scripts/bench.py [--bin PATH] [--claims 1000] [--json]
"""
import argparse
import hashlib
import json
import pathlib
import subprocess
import tempfile
import time


def derive_claim_id(claim_wo_id: dict, position: int) -> str:
    payload = json.dumps(
        {"claim": claim_wo_id, "position": position},
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return "claim-" + hashlib.sha256(payload).hexdigest()[:16]


def measure(cmd: list[str], runs: int = 5) -> float:
    samples = []
    for _ in range(runs):
        t0 = time.perf_counter()
        subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        samples.append((time.perf_counter() - t0) * 1000.0)
    samples.sort()
    return round(samples[len(samples) // 2], 2)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--bin", default="target/release/onrecord")
    ap.add_argument("--claims", type=int, default=5000)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    bin_path = pathlib.Path(args.bin)
    if not bin_path.exists():
        subprocess.run(["cargo", "build", "--release", "--locked"], check=True)

    with tempfile.TemporaryDirectory() as tmp:
        rec = pathlib.Path(tmp) / "bench.jsonl"
        prev_id = None
        lines = []
        for pos in range(args.claims):
            body = {
                "asserted_at": "2026-03-01T00:00:00Z",
                "asserter": "bench-agent",
                "due": "2026-06-01",
                "evidence": [{"reference": f"ref-{pos}", "source": "bench"}],
                "subject": f"subj-{pos % 50}",
                "supersedes": prev_id if pos % 50 != 0 else None,
                "text": f"Claim statement at position {pos}",
            }
            cid = derive_claim_id(body, pos)
            prev_id = None if (pos + 1) % 50 == 0 else cid
            stored = {"id": cid, **body}
            lines.append(json.dumps(stored, separators=(",", ":")))
        rec.write_text("\n".join(lines) + "\n", encoding="utf-8")

        results = {
            "claims": args.claims,
            "check_ms": measure([str(bin_path), "check", str(rec)], runs=3),
            "subject_ms": measure([str(bin_path), "subject", str(rec), "subj-49"], runs=3),
            "due_ms": measure([str(bin_path), "due", str(rec), "2026-12-31"], runs=3),
        }

    if args.json:
        print(json.dumps(results, indent=2))
    else:
        for k, v in results.items():
            print(f"{k}: {v}")


if __name__ == "__main__":
    main()
