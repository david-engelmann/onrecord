//! OR-5 to OR-23. The claim store and its due view,
//! ported file for file from the Python suite, plus
//! the port's own pins: the loader's depth caps, the
//! digest budgets at the read site, and the
//! duplicate key's close order.

use std::path::{Path, PathBuf};
use std::process::Command;

use flighttape::value::{Text, Value};
use onrecord::digest::{canonical_bytes, digest_of};
use onrecord::emit::{dumps_line, Mode};
use onrecord::store::{AddClaim, RecPath, Store, StoreError, BUDGET_ADD_DERIVE, BUDGET_READ};
use onrecord::{claim_defects, derive_id, load_record};

const GOLDEN: &str = "cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3";

struct Tmp(PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let dir = std::env::temp_dir().join(format!("onrecord-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        Tmp(dir)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn store_at(path: &Path) -> Store {
    Store::new(RecPath::new(path.to_str().expect("utf8 path")))
}

fn text(value: &str) -> Text {
    Text::from_str(value)
}

fn evidence() -> Value {
    Value::Array(vec![Value::Object(vec![
        (text("source"), Value::Str(text("trace"))),
        (text("reference"), Value::Str(text("run-17"))),
    ])])
}

fn add(store: &Store, tweak: impl Fn(&mut AddClaim)) -> Result<String, StoreError> {
    let mut request = AddClaim {
        text: text("The staging deploy is broken"),
        asserter: text("agent-a"),
        asserted_at: text("2026-01-01T09:00:00Z"),
        evidence: evidence(),
        supersedes: None,
        subject: None,
        due: None,
        extra: Vec::new(),
    };
    tweak(&mut request);
    store.add(request)
}

fn add_ok(store: &Store, tweak: impl Fn(&mut AddClaim)) -> String {
    add(store, tweak).expect("add succeeds")
}

fn field<'a>(value: &'a Value, name: &str) -> &'a Value {
    onrecord::store::field(value, name).expect("field present")
}

fn as_str(value: &Value) -> String {
    match value {
        Value::Str(text) => text.as_str().expect("plain text"),
        _ => panic!("not a string"),
    }
}

fn write(path: &Path, content: &str) {
    std::fs::write(path, content).expect("write record");
}

fn lines(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .expect("read record")
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect()
}

fn run_cli(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_onrecord"))
        .args(args)
        .output()
        .expect("run the binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn the_digest_is_pinned_to_the_family_golden() {
    let tape_text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tape_synthetic.json"),
    )
    .expect("fixture");
    let tape = flighttape::json::parse(&tape_text).expect("tape parses");
    assert_eq!(digest_of(&tape, BUDGET_READ).expect("digest"), GOLDEN);
}

#[test]
fn add_appends_one_line_and_the_id_is_deterministic() {
    let tmp = Tmp::new("deterministic");
    let path = tmp.path("record.jsonl");
    let first = add_ok(&store_at(&path), |_| {});
    assert!(first.starts_with("claim-"));
    assert_eq!(lines(&path).len(), 1);
    let other = tmp.path("other.jsonl");
    let again = add_ok(&store_at(&other), |_| {});
    assert_eq!(again, first);
    let second = add_ok(&store_at(&path), |r| r.text = text("A different claim"));
    assert_ne!(second, first);
    assert_eq!(lines(&path).len(), 2);
}

#[test]
fn the_store_never_invents_a_time() {
    let tmp = Tmp::new("never-invents");
    let path = tmp.path("r.jsonl");
    let store = store_at(&path);
    assert!(add(&store, |r| r.asserted_at = text("last Tuesday")).is_err());
    assert!(!path.exists());
}

#[test]
fn supersession_refusals_are_named() {
    let tmp = Tmp::new("supersession");
    let store = store_at(&tmp.path("r.jsonl"));
    let err = add(&store, |r| r.supersedes = Some(text("claim-missing"))).unwrap_err();
    assert!(err.text().contains("names no claim"), "{}", err.text());
    let first = add_ok(&store, |_| {});
    let second = add_ok(&store, |r| {
        r.text = text("The staging deploy is fixed");
        r.supersedes = Some(text(&first));
    });
    assert_ne!(second, first);
    let err = add(&store, |r| {
        r.text = text("A fork attempt");
        r.supersedes = Some(text(&first));
    })
    .unwrap_err();
    assert!(
        err.text().contains("already been superseded"),
        "{}",
        err.text()
    );
}

#[test]
fn get_returns_the_claim_and_its_chain_oldest_first() {
    let tmp = Tmp::new("chain");
    let store = store_at(&tmp.path("r.jsonl"));
    let first = add_ok(&store, |_| {});
    let second = add_ok(&store, |r| {
        r.text = text("v2");
        r.supersedes = Some(text(&first));
    });
    let third = add_ok(&store, |r| {
        r.text = text("v3");
        r.supersedes = Some(text(&second));
    });
    let document = store.get(&text(&third)).expect("get");
    assert_eq!(as_str(field(field(&document, "claim"), "id")), third);
    let chain = match field(&document, "chain") {
        Value::Array(items) => items,
        _ => panic!("chain"),
    };
    let ids: Vec<String> = chain.iter().map(|c| as_str(field(c, "id"))).collect();
    assert_eq!(ids, vec![first.clone(), second.clone(), third.clone()]);
    let middle = store.get(&text(&second)).expect("get middle");
    let chain = match field(&middle, "chain") {
        Value::Array(items) => items,
        _ => panic!("chain"),
    };
    assert_eq!(chain.len(), 2);
    let err = store.get(&text("claim-nope")).unwrap_err();
    assert!(err.text().contains("no claim with id"), "{}", err.text());
}

#[test]
fn subject_retrieval_and_the_conflict_answer() {
    let tmp = Tmp::new("subject");
    let store = store_at(&tmp.path("r.jsonl"));
    add_ok(&store, |r| r.subject = Some(text("deploy")));
    let single = store.subject(&text("deploy")).expect("subject");
    assert_eq!(as_str(field(field(&single, "claim"), "subject")), "deploy");
    add_ok(&store, |r| {
        r.text = text("An independent reading");
        r.subject = Some(text("deploy"));
    });
    let conflict = store.subject(&text("deploy")).expect("conflict");
    assert_eq!(field(&conflict, "conflict"), &Value::Bool(true));
    assert_eq!(as_str(field(&conflict, "subject")), "deploy");
    match field(&conflict, "claims") {
        Value::Array(items) => {
            assert_eq!(items.len(), 2);
            assert!(items
                .iter()
                .all(|c| onrecord::store::field(c, "chain").is_some()));
        }
        _ => panic!("claims"),
    }
    let err = store.subject(&text("nothing-here")).unwrap_err();
    assert!(err.text().contains("no live claim"), "{}", err.text());
}

#[test]
fn superseding_one_side_leaves_the_other_claim_live() {
    let tmp = Tmp::new("one-side");
    let store = store_at(&tmp.path("r.jsonl"));
    let first = add_ok(&store, |r| r.subject = Some(text("deploy")));
    add_ok(&store, |r| {
        r.text = text("An independent reading");
        r.subject = Some(text("deploy"));
    });
    add_ok(&store, |r| {
        r.text = text("The merged reading");
        r.supersedes = Some(text(&first));
        r.subject = Some(text("deploy"));
    });
    let conflict = store.subject(&text("deploy")).expect("conflict");
    assert_eq!(field(&conflict, "conflict"), &Value::Bool(true));
    let heads: Vec<String> = match field(&conflict, "claims") {
        Value::Array(items) => items
            .iter()
            .map(|c| as_str(field(field(c, "claim"), "text")))
            .collect(),
        _ => panic!("claims"),
    };
    assert!(heads.contains(&"An independent reading".to_string()));
    assert!(heads.contains(&"The merged reading".to_string()));
    assert_eq!(heads.len(), 2);
}

#[test]
fn due_dates_are_stored_and_retrieved() {
    let tmp = Tmp::new("due-stored");
    let store = store_at(&tmp.path("r.jsonl"));
    let cid = add_ok(&store, |r| r.due = Some(text("2026-02-01")));
    let document = store.get(&text(&cid)).expect("get");
    assert_eq!(
        as_str(field(field(&document, "claim"), "due")),
        "2026-02-01"
    );
    let err = add(&store, |r| r.due = Some(text("next Friday"))).unwrap_err();
    assert!(err.text().contains("not an ISO date"), "{}", err.text());
}

#[test]
fn check_names_every_defect() {
    let tmp = Tmp::new("check-defects");
    let path = tmp.path("r.jsonl");
    add_ok(&store_at(&path), |_| {});
    let mut content = lines(&path);
    content.push("{\"id\": \"claim-x\", \"text\": 5}".to_string());
    content.push("not json at all".to_string());
    content.push(
        "{\"id\":\"claim-y\",\"text\":\"t\",\"asserter\":\"a\",\"asserted_at\":\"2026-01-01T00:00:00Z\",\"evidence\":[],\"supersedes\":\"claim-ghost\"}"
            .to_string(),
    );
    write(&path, &(content.join("\n") + "\n"));
    let defects = store_at(&path).check().expect("check");
    let joined = defects
        .iter()
        .map(|d| d.render_stderr())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("line 2") && joined.contains("text"),
        "{joined}"
    );
    assert!(joined.contains("line 3: not JSON"), "{joined}");
    assert!(joined.contains("claim-ghost"), "{joined}");
    assert!(
        joined.contains("id does not match its derivation"),
        "{joined}"
    );
}

#[test]
fn a_crash_mid_append_is_a_detectable_final_line() {
    let tmp = Tmp::new("crash-append");
    let path = tmp.path("r.jsonl");
    let store = store_at(&path);
    add_ok(&store, |_| {});
    let mut bytes = std::fs::read(&path).expect("read");
    bytes.extend_from_slice(b"{\"id\": \"claim-partial\", \"text\": \"hal");
    std::fs::write(&path, bytes).expect("append partial");
    let defects = store.check().expect("check");
    assert!(defects
        .iter()
        .any(|d| d.render_stderr().contains("not JSON")));
    let error = add(&store, |r| {
        r.text = text("Cannot append to a broken record")
    })
    .expect_err("the add refuses a defective record");
    assert!(error.text().contains("defects"), "{}", error.text());
}

#[test]
fn a_fork_in_the_file_is_a_defect() {
    let tmp = Tmp::new("fork");
    let path = tmp.path("r.jsonl");
    let store = store_at(&path);
    let first = add_ok(&store, |_| {});
    add_ok(&store, |r| {
        r.text = text("v2");
        r.supersedes = Some(text(&first));
    });
    let mut content = lines(&path);
    content.push(format!(
        "{{\"id\":\"claim-fork\",\"text\":\"fork\",\"asserter\":\"b\",\"asserted_at\":\"2026-01-01T10:00:00Z\",\"evidence\":[],\"supersedes\":\"{first}\"}}"
    ));
    write(&path, &(content.join("\n") + "\n"));
    let defects = store.check().expect("check");
    assert!(defects.iter().any(|d| d.render_stderr().contains("a fork")));
}

#[test]
fn the_cli_round_trip() {
    let tmp = Tmp::new("cli");
    let path = tmp.path("r.jsonl");
    let path = path.to_str().expect("utf8");
    let (code, out, _) = run_cli(&[
        "add",
        path,
        "--text",
        "The deploy is broken",
        "--asserter",
        "agent-a",
        "--at",
        "2026-01-01T09:00:00Z",
        "--evidence",
        "trace:run-17",
        "--subject",
        "deploy",
    ]);
    assert_eq!(code, 0);
    let cid = out.trim().to_string();
    let (code, out, _) = run_cli(&["get", path, &cid]);
    assert_eq!(code, 0);
    let document = flighttape::json::parse(&out).expect("document parses");
    assert_eq!(as_str(field(field(&document, "claim"), "id")), cid);
    assert_eq!(run_cli(&["subject", path, "deploy"]).0, 0);
    assert_eq!(run_cli(&["check", path]).0, 0);
    let (code, _, err) = run_cli(&[
        "add",
        path,
        "--text",
        "x",
        "--asserter",
        "a",
        "--at",
        "2026-01-01T09:00:00Z",
        "--evidence",
        "nocolon",
    ]);
    assert_eq!(code, 1);
    assert!(err.contains("source and a reference"), "{err}");
}

#[test]
fn derive_id_pins_both_factors() {
    let claim = Value::Object(vec![
        (text("text"), Value::Str(text("x"))),
        (text("asserter"), Value::Str(text("a"))),
        (
            text("asserted_at"),
            Value::Str(text("2026-01-01T00:00:00Z")),
        ),
        (
            text("evidence"),
            Value::Array(vec![Value::Object(vec![
                (text("source"), Value::Str(text("s"))),
                (text("reference"), Value::Str(text("r"))),
            ])]),
        ),
        (text("supersedes"), Value::Null),
    ]);
    let at_zero = derive_id(&claim, 0, BUDGET_ADD_DERIVE).expect("derive");
    let at_one = derive_id(&claim, 1, BUDGET_ADD_DERIVE).expect("derive");
    assert_ne!(at_zero, at_one);
    let mut changed = match claim.clone() {
        Value::Object(entries) => entries,
        _ => panic!("object"),
    };
    changed[0].1 = Value::Str(text("y"));
    let changed = derive_id(&Value::Object(changed), 0, BUDGET_ADD_DERIVE).expect("derive");
    assert_ne!(at_zero, changed);
}

#[test]
fn twin_appends_get_distinct_ids_and_a_sound_record() {
    let tmp = Tmp::new("twins");
    let store = store_at(&tmp.path("r.jsonl"));
    let first = add_ok(&store, |_| {});
    let second = add_ok(&store, |_| {});
    assert_ne!(first, second);
    assert_eq!(store.check().expect("check"), Vec::new());
}

#[test]
fn a_refused_add_leaves_the_file_byte_identical() {
    let tmp = Tmp::new("refused-add");
    let path = tmp.path("r.jsonl");
    let store = store_at(&path);
    add_ok(&store, |_| {});
    let before = std::fs::read(&path).expect("read");
    assert!(add(&store, |r| r.text = text("")).is_err());
    assert!(add(&store, |r| r.asserter = text("")).is_err());
    assert!(add(&store, |r| r.evidence = Value::Array(vec![])).is_err());
    assert!(add(&store, |r| {
        r.evidence = Value::Array(vec![Value::Object(vec![
            (text("source"), Value::Str(text(""))),
            (text("reference"), Value::Str(text("r"))),
        ])]);
    })
    .is_err());
    assert!(add(&store, |r| r.supersedes = Some(text("claim-ghost"))).is_err());
    assert_eq!(std::fs::read(&path).expect("read"), before);
}

#[test]
fn an_unterminated_final_line_is_completed_never_glued() {
    let tmp = Tmp::new("unterminated");
    let path = tmp.path("r.jsonl");
    let store = store_at(&path);
    let first = add_ok(&store, |_| {});
    let bytes = std::fs::read(&path).expect("read");
    std::fs::write(&path, bytes.strip_suffix(b"\n").expect("newline")).expect("truncate");
    assert_eq!(store.check().expect("check"), Vec::new());
    let second = add_ok(&store, |r| r.text = text("The next claim"));
    assert_eq!(lines(&path).len(), 2);
    assert_eq!(
        as_str(field(
            field(&store.get(&text(&first)).expect("get"), "claim"),
            "id"
        )),
        first
    );
    assert_eq!(
        as_str(field(
            field(&store.get(&text(&second)).expect("get"), "claim"),
            "id"
        )),
        second
    );
}

#[test]
fn a_forged_id_hiding_behind_nan_is_a_named_defect() {
    let tmp = Tmp::new("nan");
    let path = tmp.path("r.jsonl");
    add_ok(&store_at(&path), |_| {});
    let mut content = lines(&path);
    content.push(
        "{\"id\":\"claim-attackerchosen\",\"text\":\"forged\",\"asserter\":\"mallory\",\"asserted_at\":\"2026-01-01T00:00:00Z\",\"evidence\":[{\"source\":\"s\",\"reference\":\"r\"}],\"supersedes\":null,\"reading\":NaN}"
            .to_string(),
    );
    write(&path, &(content.join("\n") + "\n"));
    let store = store_at(&path);
    let defects = store.check().expect("check");
    assert!(defects
        .iter()
        .any(|d| d.render_stderr().contains("cannot be verified")));
    let error = store
        .get(&text("claim-attackerchosen"))
        .expect_err("the get refuses a defective record");
    assert!(error.text().contains("defects"), "{}", error.text());
}

#[test]
fn check_names_deep_nesting_instead_of_dying() {
    let tmp = Tmp::new("deep");
    let path = tmp.path("r.jsonl");
    let deep = "[".repeat(30000) + "1" + &"]".repeat(30000);
    write(&path, &format!("{{\"id\": {deep}}}\n"));
    let defects = store_at(&path).check().expect("check");
    assert!(defects
        .iter()
        .any(|d| d.render_stderr().contains("nesting too deep")));
}

#[test]
fn a_duplicate_key_in_a_line_is_a_defect() {
    let tmp = Tmp::new("dup");
    let path = tmp.path("r.jsonl");
    write(
        &path,
        "{\"id\": \"claim-a\", \"id\": \"claim-b\", \"text\": \"t\"}\n",
    );
    let defects = store_at(&path).check().expect("check");
    assert!(defects
        .iter()
        .any(|d| d.render_stderr().contains("duplicate key")));
}

#[test]
fn the_duplicate_that_counts_is_the_first_object_to_close() {
    // The Python's hook raises as an object closes:
    // the nested object's duplicate fires ahead of
    // the enclosing object's own repeated key.
    let tmp = Tmp::new("dup-order");
    let path = tmp.path("r.jsonl");
    write(&path, "{\"a\": 1, \"a\": 2, \"b\": {\"x\": 1, \"x\": 2}}\n");
    let defects = store_at(&path).check().expect("check");
    assert!(
        defects
            .iter()
            .any(|d| d.render_stderr().contains("duplicate key x")),
        "{:?}",
        defects
            .iter()
            .map(|d| d.render_stderr())
            .collect::<Vec<_>>()
    );
    // And a duplicate whose object never closes
    // behind a syntax error stays a parse failure.
    write(&path, "{\"a\": 1, \"a\": 2, \"b\": }\n");
    let defects = store_at(&path).check().expect("check");
    assert!(defects
        .iter()
        .any(|d| d.render_stderr().contains("not JSON")));
}

#[test]
fn a_record_with_a_bom_reads() {
    let tmp = Tmp::new("bom");
    let path = tmp.path("r.jsonl");
    let store = store_at(&path);
    let cid = add_ok(&store, |_| {});
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(&std::fs::read(&path).expect("read"));
    std::fs::write(&path, bytes).expect("rewrite with bom");
    let document = store.get(&text(&cid)).expect("get");
    assert_eq!(as_str(field(field(&document, "claim"), "id")), cid);
}

#[test]
fn check_on_a_missing_record_is_a_refusal() {
    let tmp = Tmp::new("missing");
    let missing = tmp.path("nope.jsonl");
    let (code, _, err) = run_cli(&["check", missing.to_str().expect("utf8")]);
    assert_eq!(code, 1);
    assert!(err.contains("no such record"), "{err}");
}

#[test]
fn a_chain_head_answers_for_its_chains_subject() {
    let tmp = Tmp::new("answers");
    let store = store_at(&tmp.path("r.jsonl"));
    let first = add_ok(&store, |r| r.subject = Some(text("deploy")));
    add_ok(&store, |r| {
        r.text = text("The head carries no subject");
        r.supersedes = Some(text(&first));
    });
    let document = store.subject(&text("deploy")).expect("subject");
    assert_eq!(
        as_str(field(field(&document, "claim"), "text")),
        "The head carries no subject"
    );
}

#[test]
fn the_canonical_bytes_battery() {
    let obj = |pairs: Vec<(&str, Value)>| {
        Value::Object(pairs.into_iter().map(|(k, v)| (text(k), v)).collect())
    };
    let cases: Vec<(Value, &[u8])> = vec![
        (
            obj(vec![
                ("b", Value::Int("1".into())),
                ("a", Value::Int("2".into())),
            ]),
            b"{\"a\":2,\"b\":1}",
        ),
        (obj(vec![("x", Value::Float(1.5))]), b"{\"x\":1.5}"),
        (obj(vec![("x", Value::Float(1e21))]), b"{\"x\":1e+21}"),
        (
            obj(vec![("x", Value::Int("1152921504606846977".into()))]),
            b"{\"x\":1152921504606846977}",
        ),
        (
            obj(vec![
                ("é", Value::Int("1".into())),
                ("e", Value::Int("2".into())),
            ]),
            "{\"e\":2,\"é\":1}".as_bytes(),
        ),
    ];
    for (value, expected) in cases {
        assert_eq!(
            canonical_bytes(&value, BUDGET_READ).expect("render"),
            expected
        );
    }
}

#[test]
fn the_due_view_answers_live_claims_by_date() {
    let tmp = Tmp::new("due-view");
    let store = store_at(&tmp.path("r.jsonl"));
    let early = add_ok(&store, |r| {
        r.text = text("Due early");
        r.due = Some(text("2026-02-01"));
    });
    add_ok(&store, |r| {
        r.text = text("Due later");
        r.due = Some(text("2026-03-01"));
    });
    add_ok(&store, |r| r.text = text("No date at all"));
    let document = store.due(&text("2026-02-01")).expect("due");
    assert_eq!(as_str(field(&document, "as_of")), "2026-02-01");
    let due = match field(&document, "due") {
        Value::Array(items) => items,
        _ => panic!("due"),
    };
    assert_eq!(due.len(), 1);
    assert_eq!(as_str(field(field(&due[0], "claim"), "id")), early);
    let chain = match field(&due[0], "chain") {
        Value::Array(items) => items,
        _ => panic!("chain"),
    };
    assert_eq!(as_str(field(&chain[0], "id")), early);
    assert!(onrecord::store::field(&document, "conflicts").is_none());
    let document = store.due(&text("2026-01-01")).expect("due");
    assert_eq!(field(&document, "due"), &Value::Array(vec![]));
}

#[test]
fn the_due_view_orders_by_date_then_position() {
    let tmp = Tmp::new("due-order");
    let store = store_at(&tmp.path("r.jsonl"));
    let third = add_ok(&store, |r| {
        r.text = text("Third");
        r.due = Some(text("2026-02-10"));
    });
    let first = add_ok(&store, |r| {
        r.text = text("First");
        r.due = Some(text("2026-02-01"));
    });
    let second = add_ok(&store, |r| {
        r.text = text("Second");
        r.due = Some(text("2026-02-10"));
    });
    let document = store.due(&text("2026-02-10")).expect("due");
    let ids: Vec<String> = match field(&document, "due") {
        Value::Array(items) => items
            .iter()
            .map(|d| as_str(field(field(d, "claim"), "id")))
            .collect(),
        _ => panic!("due"),
    };
    assert_eq!(ids, vec![first, third, second]);
}

#[test]
fn a_superseded_claims_due_date_dies_with_it() {
    let tmp = Tmp::new("due-dies");
    let store = store_at(&tmp.path("r.jsonl"));
    let old = add_ok(&store, |r| {
        r.text = text("Old promise");
        r.due = Some(text("2026-02-01"));
    });
    let new = add_ok(&store, |r| {
        r.text = text("New promise");
        r.supersedes = Some(text(&old));
        r.due = Some(text("2026-04-01"));
    });
    let document = store.due(&text("2026-03-01")).expect("due");
    assert_eq!(field(&document, "due"), &Value::Array(vec![]));
    let document = store.due(&text("2026-04-01")).expect("due");
    let due = match field(&document, "due") {
        Value::Array(items) => items,
        _ => panic!("due"),
    };
    assert_eq!(as_str(field(field(&due[0], "claim"), "id")), new);
    let chain = match field(&due[0], "chain") {
        Value::Array(items) => items,
        _ => panic!("chain"),
    };
    let ids: Vec<String> = chain.iter().map(|c| as_str(field(c, "id"))).collect();
    assert_eq!(ids, vec![old, new]);
}

#[test]
fn the_due_view_names_its_conflicts() {
    let tmp = Tmp::new("due-conflicts");
    let store = store_at(&tmp.path("r.jsonl"));
    for (label, subject) in [
        ("Side one", "deploy"),
        ("Side two", "deploy"),
        ("Unrelated", "other"),
    ] {
        add_ok(&store, |r| {
            r.text = text(label);
            r.subject = Some(text(subject));
            r.due = Some(text("2026-02-01"));
        });
    }
    let document = store.due(&text("2026-02-01")).expect("due");
    match field(&document, "due") {
        Value::Array(items) => assert_eq!(items.len(), 3),
        _ => panic!("due"),
    }
    assert_eq!(
        field(&document, "conflicts"),
        &Value::Array(vec![Value::Str(text("deploy"))])
    );
}

#[test]
fn the_due_view_refuses_a_bad_date_by_name() {
    let tmp = Tmp::new("due-bad-date");
    let store = store_at(&tmp.path("r.jsonl"));
    add_ok(&store, |r| r.due = Some(text("2026-02-01")));
    for bad in ["next Friday", "2026-02-30", "2026-02-01T09:00:00Z", ""] {
        let err = store.due(&text(bad)).unwrap_err();
        assert!(
            err.text().contains("not an ISO date"),
            "{} for {bad}",
            err.text()
        );
    }
}

#[test]
fn the_due_command_prints_the_document() {
    let tmp = Tmp::new("due-command");
    let record = tmp.path("r.jsonl");
    let store = store_at(&record);
    let dated = add_ok(&store, |r| {
        r.text = text("Dated");
        r.due = Some(text("2026-02-01"));
    });
    add_ok(&store, |r| r.text = text("Undated"));
    let record = record.to_str().expect("utf8");
    let (code, out, _) = run_cli(&["due", record, "2026-02-15"]);
    assert_eq!(code, 0);
    let document = flighttape::json::parse(&out).expect("document");
    assert_eq!(as_str(field(&document, "as_of")), "2026-02-15");
    let due = match field(&document, "due") {
        Value::Array(items) => items,
        _ => panic!("due"),
    };
    assert_eq!(as_str(field(field(&due[0], "claim"), "id")), dated);
    let (code, _, err) = run_cli(&["due", record, "whenever"]);
    assert_eq!(code, 1);
    assert!(err.contains("not an ISO date"), "{err}");
}

#[test]
fn an_undated_successor_silences_the_chain() {
    let tmp = Tmp::new("silences");
    let store = store_at(&tmp.path("r.jsonl"));
    let old = add_ok(&store, |r| {
        r.text = text("Promised");
        r.due = Some(text("2026-02-01"));
    });
    add_ok(&store, |r| {
        r.text = text("Withdrawn");
        r.supersedes = Some(text(&old));
    });
    let document = store.due(&text("2026-06-01")).expect("due");
    assert_eq!(field(&document, "due"), &Value::Array(vec![]));
}

#[test]
fn a_due_claims_document_is_gets_document() {
    let tmp = Tmp::new("due-is-get");
    let store = store_at(&tmp.path("r.jsonl"));
    let dated = add_ok(&store, |r| {
        r.text = text("Dated");
        r.due = Some(text("2026-02-01"));
    });
    let document = store.due(&text("2026-02-01")).expect("due");
    let due = match field(&document, "due") {
        Value::Array(items) => items,
        _ => panic!("due"),
    };
    assert_eq!(due[0], store.get(&text(&dated)).expect("get"));
}

#[test]
fn a_conflict_needs_both_sides_due() {
    let tmp = Tmp::new("both-due");
    let store = store_at(&tmp.path("r.jsonl"));
    add_ok(&store, |r| {
        r.text = text("Side one");
        r.subject = Some(text("deploy"));
        r.due = Some(text("2026-02-01"));
    });
    add_ok(&store, |r| {
        r.text = text("Side two");
        r.subject = Some(text("deploy"));
        r.due = Some(text("2026-09-01"));
    });
    let document = store.due(&text("2026-03-01")).expect("due");
    match field(&document, "due") {
        Value::Array(items) => assert_eq!(items.len(), 1),
        _ => panic!("due"),
    }
    assert!(onrecord::store::field(&document, "conflicts").is_none());
}

#[test]
fn inherited_subjects_conflict_in_the_due_view() {
    let tmp = Tmp::new("inherited");
    let store = store_at(&tmp.path("r.jsonl"));
    let root = add_ok(&store, |r| {
        r.text = text("Root");
        r.subject = Some(text("deploy"));
    });
    add_ok(&store, |r| {
        r.text = text("Head without a subject");
        r.supersedes = Some(text(&root));
        r.due = Some(text("2026-02-01"));
    });
    add_ok(&store, |r| {
        r.text = text("Direct");
        r.subject = Some(text("deploy"));
        r.due = Some(text("2026-02-01"));
    });
    let document = store.due(&text("2026-02-01")).expect("due");
    assert_eq!(
        field(&document, "conflicts"),
        &Value::Array(vec![Value::Str(text("deploy"))])
    );
    let subject = store.subject(&text("deploy")).expect("subject");
    assert_eq!(field(&subject, "conflict"), &Value::Bool(true));
}

#[test]
fn answering_never_self_conflicts() {
    let tmp = Tmp::new("self-conflict");
    let store = store_at(&tmp.path("r.jsonl"));
    let root = add_ok(&store, |r| {
        r.text = text("Root");
        r.subject = Some(text("deploy"));
    });
    let mid = add_ok(&store, |r| {
        r.text = text("Mid");
        r.supersedes = Some(text(&root));
        r.subject = Some(text("deploy"));
    });
    add_ok(&store, |r| {
        r.text = text("Head");
        r.supersedes = Some(text(&mid));
        r.due = Some(text("2026-02-01"));
    });
    add_ok(&store, |r| {
        r.text = text("Quiet one");
        r.due = Some(text("2026-02-01"));
    });
    add_ok(&store, |r| {
        r.text = text("Quiet two");
        r.due = Some(text("2026-02-01"));
    });
    let document = store.due(&text("2026-02-01")).expect("due");
    match field(&document, "due") {
        Value::Array(items) => assert_eq!(items.len(), 3),
        _ => panic!("due"),
    }
    assert!(onrecord::store::field(&document, "conflicts").is_none());
}

#[test]
fn a_head_with_its_own_subject_does_not_inherit() {
    let tmp = Tmp::new("own-subject");
    let store = store_at(&tmp.path("r.jsonl"));
    let root = add_ok(&store, |r| {
        r.text = text("Root");
        r.subject = Some(text("deploy"));
    });
    add_ok(&store, |r| {
        r.text = text("Head on another subject");
        r.supersedes = Some(text(&root));
        r.subject = Some(text("billing"));
        r.due = Some(text("2026-02-01"));
    });
    add_ok(&store, |r| {
        r.text = text("Direct");
        r.subject = Some(text("deploy"));
        r.due = Some(text("2026-02-01"));
    });
    let document = store.due(&text("2026-02-01")).expect("due");
    assert!(onrecord::store::field(&document, "conflicts").is_none());
}

#[test]
fn dates_are_exactly_one_form() {
    let tmp = Tmp::new("one-form");
    let store = store_at(&tmp.path("r.jsonl"));
    for bad in ["20260201", "2026-W05-1", "2026-W05"] {
        assert!(add(&store, |r| r.due = Some(text(bad))).is_err());
        let err = store.due(&text(bad)).unwrap_err();
        assert!(err.text().contains("not an ISO date"), "{}", err.text());
    }
    let dated = add_ok(&store, |r| r.due = Some(text("2026-02-01")));
    let document = store.due(&text("2026-02-01")).expect("due");
    let due = match field(&document, "due") {
        Value::Array(items) => items,
        _ => panic!("due"),
    };
    assert_eq!(as_str(field(field(&due[0], "claim"), "id")), dated);
}

#[test]
fn the_due_view_refuses_a_missing_record() {
    let tmp = Tmp::new("due-missing");
    let missing = tmp.path("missing.jsonl");
    let err = store_at(&missing).due(&text("2026-02-01")).unwrap_err();
    assert!(
        err.text().contains("cannot read the record"),
        "{}",
        err.text()
    );
    let err = store_at(&tmp.0).due(&text("2026-02-01")).unwrap_err();
    assert!(
        err.text().contains("cannot read the record"),
        "{}",
        err.text()
    );
    let (code, _, _) = run_cli(&["due", missing.to_str().expect("utf8"), "2026-02-01"]);
    assert_eq!(code, 1);
}

#[test]
fn check_names_an_alien_due_date_at_read_time() {
    let tmp = Tmp::new("alien-due");
    let path = tmp.path("r.jsonl");
    let claim = Value::Object(vec![
        (text("text"), Value::Str(text("Hand written"))),
        (text("asserter"), Value::Str(text("agent-a"))),
        (
            text("asserted_at"),
            Value::Str(text("2026-01-01T09:00:00Z")),
        ),
        (text("evidence"), evidence()),
        (text("supersedes"), Value::Null),
        (text("due"), Value::Str(text("20260201"))),
    ]);
    let id = derive_id(&claim, 0, BUDGET_ADD_DERIVE).expect("derive");
    let mut entries = vec![(text("id"), Value::Str(text(&id)))];
    if let Value::Object(pairs) = claim {
        entries.extend(pairs);
    }
    let stored = Value::Object(entries);
    let line = onrecord::emit::dumps_line(&stored, onrecord::emit::Mode::Strict).expect("render");
    let mut bytes = line;
    bytes.push(b'\n');
    std::fs::write(&path, bytes).expect("write");
    let store = store_at(&path);
    let defects = store.check().expect("check");
    assert!(defects.iter().any(|d| d.render_stderr().contains("due")));
    assert!(store.due(&text("2026-12-31")).is_err());
}

#[test]
fn the_loader_depth_caps_are_the_probed_pair() {
    // Deep values drop recursively; run on the same
    // generous stack the CLI uses.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(loader_depth_caps_body)
        .expect("thread");
    child.join().expect("the caps hold");
}

fn loader_depth_caps_body() {
    // A bare array line: parse depth 9994 reads (and
    // is then not a claim object); 9995 is refused
    // with the nesting phrase. Objects close the
    // hook's stack sooner: 9991 reads, 9992 refuses.
    let tmp = Tmp::new("loader-caps");
    let path = tmp.path("r.jsonl");
    let defects_at = |content: &str| {
        write(&path, content);
        let (_, defects) = load_record(&RecPath::new(path.to_str().expect("utf8")), BUDGET_READ);
        defects
            .iter()
            .map(|d| d.render_stderr())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let arrays = |n: usize| "[".repeat(n) + &"]".repeat(n) + "\n";
    let objects = |n: usize| "{\"a\":".repeat(n) + "1" + &"}".repeat(n) + "\n";
    assert!(defects_at(&arrays(9994)).contains("not a claim object"));
    assert!(defects_at(&arrays(9995)).contains("nesting too deep to read"));
    assert!(defects_at(&objects(9991)).contains("missing field id"));
    assert!(defects_at(&objects(9992)).contains("nesting too deep to read"));
}

#[test]
fn the_digest_budget_at_read_is_the_probed_pair() {
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(digest_budget_body)
        .expect("thread");
    child.join().expect("the budget holds");
}

fn digest_budget_body() {
    // A claim whose extra field nests arrays: the
    // read site's budget refuses the derive past
    // node depth 990 (payload counted), which in
    // this shape is 987 nested arrays accepted (the
    // refusal then is the id mismatch) and 988
    // refused as unverifiable.
    let _tmp = Tmp::new("digest-caps");
    let verdict = |n: usize| {
        let mut deep = Value::Int("1".into());
        for _ in 0..n {
            deep = Value::Array(vec![deep]);
        }
        let claim = Value::Object(vec![
            (text("id"), Value::Str(text("claim-0000000000000000"))),
            (text("text"), Value::Str(text("t"))),
            (text("asserter"), Value::Str(text("a"))),
            (
                text("asserted_at"),
                Value::Str(text("2026-01-01T00:00:00Z")),
            ),
            (text("evidence"), evidence()),
            (text("supersedes"), Value::Null),
            (text("deep"), deep),
        ]);
        let defects = claim_defects(&claim, 1, 0, BUDGET_READ);
        defects
            .iter()
            .map(|d| d.render_stderr())
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert!(
        verdict(987).contains("id does not match"),
        "{}",
        verdict(987)
    );
    assert!(
        verdict(988).contains("cannot be rendered"),
        "{}",
        verdict(988)
    );
}

#[test]
fn the_line_scan_arbitrates_in_the_parsers_order() {
    let tmp = Tmp::new("scan-order");
    let path = tmp.path("r.jsonl");
    let store = store_at(&path);
    let defects_at = |content: &str| {
        write(&path, content);
        store
            .check()
            .expect("check")
            .iter()
            .map(|d| d.render_stderr())
            .collect::<Vec<_>>()
            .join("\n")
    };
    // A lexical error ahead of the close hides a
    // pending duplicate behind the parse failure:
    // an invalid escape, a bad literal, a raw
    // control character in a string.
    assert!(defects_at("{\"a\": 1, \"a\": 2, \"b\": \"\\x\"}\n").contains("not JSON"));
    assert!(defects_at("{\"a\": tru, \"b\": {\"x\": 1, \"x\": 2}}\n").contains("not JSON"));
    assert!(defects_at("{\"a\": 1, \"a\": 2, \"b\": \"x\u{1}y\"}\n").contains("not JSON"));
    // The object that closes first names its
    // duplicate, ahead of an enclosing object's.
    assert!(
        defects_at("{\"a\": 1, \"a\": 2, \"b\": {\"x\": 1, \"x\": 2}}\n")
            .contains("duplicate key x")
    );
    assert!(defects_at("{\"a\": {\"x\": 1, \"x\": 2}, \"a\": 3}\n").contains("duplicate key x"));
    // Depth is measured on the value being read:
    // trailing depth the parser never reaches does
    // not decide the phrase.
    let trailing = "[".repeat(10000) + &"]".repeat(10000);
    assert!(defects_at(&format!("[1] {trailing}\n")).contains("not JSON"));
    assert!(defects_at(&format!("{{\"a\":1,\"a\":2}} {trailing}\n")).contains("duplicate key a"));
    // A negated NaN is not a constant this loader
    // accepts.
    assert!(defects_at("{\"reading\": -NaN}\n").contains("not JSON"));
}

#[test]
fn the_read_budget_splits_by_entry() {
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(read_budget_split_body)
        .expect("thread");
    child.join().expect("the split holds");
}

fn read_budget_split_body() {
    // At 987 nested arrays the record verifies
    // through check and due, which load directly,
    // and refuses through get and subject, whose
    // loads pass through the store's gate one frame
    // deeper (probed at the shipped CLI).
    let tmp = Tmp::new("budget-split");
    let path = tmp.path("r.jsonl");
    let mut deep = Value::Int("1".into());
    for _ in 0..987 {
        deep = Value::Array(vec![deep]);
    }
    let claim = Value::Object(vec![
        (text("text"), Value::Str(text("t"))),
        (text("asserter"), Value::Str(text("a"))),
        (
            text("asserted_at"),
            Value::Str(text("2026-01-01T00:00:00Z")),
        ),
        (text("evidence"), evidence()),
        (text("supersedes"), Value::Null),
        (text("reading"), deep),
    ]);
    let id = derive_id(&claim, 0, BUDGET_READ).expect("derive");
    let mut entries = vec![(text("id"), Value::Str(text(&id)))];
    if let Value::Object(fields) = &claim {
        entries.extend(fields.iter().cloned());
    }
    let stored = Value::Object(entries);
    let line = dumps_line(&stored, Mode::Strict).expect("render");
    std::fs::write(&path, line).expect("write");
    let store = store_at(&path);
    assert!(store.check().expect("check").is_empty());
    assert!(store.due(&text("2027-01-01")).is_ok());
    let error = store.get(&text(&id)).expect_err("get refuses");
    assert!(
        error.text().contains("cannot be rendered"),
        "{}",
        error.text()
    );
    let error = store.subject(&text("k")).expect_err("subject refuses");
    assert!(
        error.text().contains("cannot be rendered"),
        "{}",
        error.text()
    );
}

#[test]
fn the_digest_budget_object_measure_at_read_is_the_probed_point() {
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(digest_object_budget_body)
        .expect("thread");
    child.join().expect("the budget holds");
}

fn digest_object_budget_body() {
    // A claim whose extra field nests objects: the
    // read budget's second measure (the deepest
    // object node, BUDGET_READ's 988) binds before
    // the node measure does. In this shape 986
    // nested objects render, so the refusal a wrong
    // id draws is the mismatch, and 987 are refused
    // as unverifiable, one nest before the array
    // shape's flip in
    // the_digest_budget_at_read_is_the_probed_pair.
    // The CLI probe of Round 9 measured the same
    // points on both binaries.
    let _tmp = Tmp::new("digest-object-caps");
    let verdict = |n: usize| {
        let mut deep = Value::Int("1".into());
        for _ in 0..n {
            deep = Value::Object(vec![(text("a"), deep)]);
        }
        let claim = Value::Object(vec![
            (text("id"), Value::Str(text("claim-0000000000000000"))),
            (text("text"), Value::Str(text("t"))),
            (text("asserter"), Value::Str(text("a"))),
            (
                text("asserted_at"),
                Value::Str(text("2026-01-01T09:00:00Z")),
            ),
            (text("evidence"), evidence()),
            (text("supersedes"), Value::Null),
            (text("deep"), deep),
        ]);
        let defects = claim_defects(&claim, 1, 0, BUDGET_READ);
        defects
            .iter()
            .map(|defect| defect.render_stderr())
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert!(
        verdict(986).contains("id does not match"),
        "{}",
        verdict(986)
    );
    assert!(
        verdict(987).contains("cannot be rendered"),
        "{}",
        verdict(987)
    );
}

#[test]
fn the_entry_split_holds_on_the_object_measure_too() {
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(object_split_body)
        .expect("thread");
    child.join().expect("the split holds");
}

fn object_split_body() {
    // The same split as the_read_budget_splits_by_entry,
    // on the object measure: a claim nesting 986
    // objects in its extra field renders under
    // BUDGET_READ, so check verifies it, and the
    // store gate's narrower budget refuses it at the
    // same nest, so get and subject refuse.
    let tmp = Tmp::new("object-split");
    let path = tmp.path("r.jsonl");
    let mut deep = Value::Int("1".into());
    for _ in 0..986 {
        deep = Value::Object(vec![(text("a"), deep)]);
    }
    let claim = Value::Object(vec![
        (text("text"), Value::Str(text("t"))),
        (text("asserter"), Value::Str(text("a"))),
        (
            text("asserted_at"),
            Value::Str(text("2026-01-01T09:00:00Z")),
        ),
        (text("evidence"), evidence()),
        (text("supersedes"), Value::Null),
        (text("reading"), deep),
    ]);
    let id = derive_id(&claim, 0, BUDGET_READ).expect("derive");
    let mut entries = vec![(text("id"), Value::Str(text(&id)))];
    if let Value::Object(fields) = &claim {
        entries.extend(fields.iter().cloned());
    }
    let stored = Value::Object(entries);
    let line = dumps_line(&stored, Mode::Strict).expect("render");
    std::fs::write(&path, line).expect("write");
    let store = store_at(&path);
    assert!(store.check().expect("check runs").is_empty());
    let error = store.get(&text(&id)).expect_err("get refuses");
    assert!(
        error.text().contains("cannot be rendered"),
        "{}",
        error.text()
    );
    let error = store.subject(&text("k")).expect_err("subject refuses");
    assert!(
        error.text().contains("cannot be rendered"),
        "{}",
        error.text()
    );
}

#[test]
fn a_duplicated_line_deep_in_a_record_is_named_at_its_own_line() {
    // A verbatim copy of the first line appended at
    // the tail of a 120-claim record draws exactly
    // two defects, both at the copy's own line: the
    // id no longer matches its derivation, because
    // the position moved, and the id appears twice.
    // The original line keeps no defect. Round 9's
    // CLI battery measured the same law at 3,000
    // claims (OR-53).
    let tmp = Tmp::new("dup-at-scale");
    let path = tmp.path("r.jsonl");
    let store = store_at(&path);
    let mut first_id = String::new();
    for i in 0..120 {
        let id = add_ok(&store, |claim| {
            claim.text = text(&format!("Synthetic claim {i}"));
        });
        if i == 0 {
            first_id = id;
        }
    }
    let bytes = std::fs::read(&path).expect("read the record");
    let first_line = bytes.split(|b| *b == b'\n').next().expect("first line");
    let mut grown = bytes.clone();
    grown.extend_from_slice(first_line);
    grown.push(b'\n');
    std::fs::write(&path, &grown).expect("append the copy");
    let (_, defects) = load_record(&RecPath::new(path.to_str().expect("utf8")), BUDGET_READ);
    let rendered: Vec<String> = defects
        .iter()
        .map(|defect| defect.render_stderr())
        .collect();
    assert_eq!(
        rendered,
        vec![
            "line 121: id does not match its derivation".to_string(),
            format!("line 121: id {first_id} appears twice"),
        ]
    );
}

#[test]
fn a_name_past_the_os_limit_surfaces_its_os_error() {
    let tmp = Tmp::new("long-name");
    let path = tmp.path(&"x".repeat(300));
    let store = store_at(&path);
    let error = store.check().expect_err("check surfaces the OS error");
    assert!(
        error.text().starts_with("[Errno 36] File name too long"),
        "{}",
        error.text()
    );
    let error = store
        .get(&text("claim-x"))
        .expect_err("get surfaces it too");
    assert!(error.text().starts_with("[Errno 36]"), "{}", error.text());
    // due never gates on exists: its load reports
    // the unreadable record as a defect instead.
    let error = store
        .due(&text("2027-01-01"))
        .expect_err("due refuses by defect");
    assert!(
        error.text().contains("cannot read the record"),
        "{}",
        error.text()
    );
}

#[test]
fn a_fork_names_a_non_string_source_id_by_its_repr() {
    let tmp = Tmp::new("fork-repr");
    let path = tmp.path("r.jsonl");
    let store = store_at(&path);
    let first = add_ok(&store, |_| {});
    let mut content = lines(&path);
    content.push(format!(
        "{{\"id\":[1,2],\"text\":\"t\",\"asserter\":\"a\",\"asserted_at\":\"2026-01-01T00:00:00Z\",\"evidence\":[{{\"source\":\"s\",\"reference\":\"r\"}}],\"supersedes\":\"{first}\"}}"
    ));
    content.push(format!(
        "{{\"id\":{{\"k\":1}},\"text\":\"t\",\"asserter\":\"a\",\"asserted_at\":\"2026-01-01T00:00:00Z\",\"evidence\":[{{\"source\":\"s\",\"reference\":\"r\"}}],\"supersedes\":\"{first}\"}}"
    ));
    write(&path, &(content.join("\n") + "\n"));
    let defects = store.check().expect("check");
    let joined = defects
        .iter()
        .map(|d| d.render_stderr())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(joined.contains("[1, 2] and {'k': 1}"), "{joined}");
}

// OR-36. The Round 6 append witnesses move from
// the probe record into the suite: an append
// writes in place, so the record keeps its inode
// and a hardlink taken before the append sees the
// appended line. An append through a temp and a
// rename would fail both assertions.
#[cfg(unix)]
#[test]
fn an_append_keeps_the_inode_and_the_hardlinks() {
    use std::os::unix::fs::MetadataExt;
    let tmp = Tmp::new("inode");
    let path = tmp.path("r.jsonl");
    let store = store_at(&path);
    add_ok(&store, |_| {});
    let before = std::fs::metadata(&path).expect("stat the record").ino();
    let witness = tmp.path("witness.jsonl");
    std::fs::hard_link(&path, &witness).expect("hardlink the record");
    add_ok(&store, |claim| {
        claim.text = text("A second claim");
    });
    let after = std::fs::metadata(&path).expect("stat the record").ino();
    assert_eq!(before, after, "the append replaced the file");
    assert_eq!(lines(&witness).len(), 2, "the hardlink missed the append");
    assert_eq!(lines(&path), lines(&witness));
}

// OR-36. The existence split in its absolute
// form: a symlink component popped by a following
// dotdot, its target missing, refuses the add,
// and no record appears at the lexical location
// a lexical absolutization would have written to.
#[cfg(unix)]
#[test]
fn an_append_through_a_popped_missing_link_is_refused() {
    let tmp = Tmp::new("split");
    let link = tmp.path("link");
    std::os::unix::fs::symlink("void", &link).expect("plant the link");
    let spelling = format!("{}/../record.jsonl", link.display());
    let store = Store::new(RecPath::new(&spelling));
    let refused = add(&store, |_| {});
    assert!(refused.is_err(), "the add through the split was accepted");
    assert!(
        !tmp.path("record.jsonl").exists(),
        "a record landed at the lexical location"
    );
}
