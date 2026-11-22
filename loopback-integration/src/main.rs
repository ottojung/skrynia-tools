//! Disposable loopback integration harness (DRAFT, marceline-dev).
//! Exercises the REAL sender transport / VAPID / encryption / durable ledger
//! against a controlled local HTTPS receiver. P1 and the push service are
//! loopback fakes; the pinned P1 WASM lacks push opcodes (see BLOCKER.md), so
//! the real-WASM, kernel/P0/trusted-edge and browser-SW boundaries are
//! explicitly UNTESTED here. No production, main, DNS, TLS, secrets or real
//! subscriptions are touched.
mod p1;
mod receiver;

use base64ct::{Base64UrlUnpadded, Encoding};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use receiver::{FakePushService, HOST, ReplyPlan, Seen};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Mutex;
use webpush_sender::error::{IoFailure, Result as WResult};
use webpush_sender::ledger::FileLedgerStore;
use webpush_sender::p1::P1Transport;
use webpush_sender::pump::{Clock, Parts, Pump, Report};
use webpush_sender::push::{HttpsTransport, PushRequest, PushTransport, Reply, Rng};
use webpush_sender::sub::Resolver;
use webpush_sender::vapid::VapidKey;
use webpush_sender::{Config, Error};

const CAP: [u8; 32] = [7u8; 32];
const NS: [u8; 32] = [9u8; 32];
const KEY: &[u8] = b"notice/arrival";
const PRIV: [u8; 32] = [3u8; 32]; // disposable test scalar, never a live key

struct FixedResolver(HashMap<String, Vec<IpAddr>>);
impl Resolver for FixedResolver {
    fn resolve(&self, host: &str) -> core::result::Result<Vec<IpAddr>, IoFailure> {
        self.0.get(host).cloned().ok_or(IoFailure)
    }
}
fn resolver() -> FixedResolver {
    let mut m = HashMap::new();
    m.insert(HOST.to_owned(), vec!["93.184.216.34".parse().unwrap()]);
    FixedResolver(m)
}

struct FixedRng;
impl Rng for FixedRng {
    fn fill(&self, out: &mut [u8]) {
        for (i, b) in out.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(31).wrapping_add(4);
        }
    }
}

struct TestClock {
    now: Mutex<u64>,
    sleeps: Mutex<Vec<u64>>,
}
impl TestClock {
    fn new() -> Self {
        Self {
            now: Mutex::new(1_700_000_000_000),
            sleeps: Mutex::new(Vec::new()),
        }
    }
}
impl Clock for TestClock {
    fn now_ms(&self) -> u64 {
        *self.now.lock().unwrap()
    }
    fn sleep_ms(&self, ms: u64) {
        *self.now.lock().unwrap() += ms;
        self.sleeps.lock().unwrap().push(ms);
    }
}

/// Maps the SSRF-validated global pin onto the loopback receiver while keeping
/// the real pinned-IP HTTPS transport (TLS, SNI, VAPID, guard) in the path.
struct LoopbackTransport {
    ca_der: Vec<u8>,
    port: u16,
    sent: Mutex<Vec<PushRequest>>,
}
impl PushTransport for LoopbackTransport {
    fn send(&self, req: &PushRequest) -> core::result::Result<Reply, IoFailure> {
        self.sent.lock().unwrap().push(req.clone());
        let mut mapped = req.clone();
        mapped.ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        HttpsTransport::for_test(&self.ca_der, self.port, 4_000).send(&mapped)
    }
}

fn config(max_attempts: u8) -> Config {
    Config {
        sender_cap: CAP,
        ns: NS,
        key: KEY.to_vec(),
        max_payload: 4096,
        max_attempts,
        base_backoff_ms: 10,
        max_backoff_ms: 100,
        ttl_secs: 60,
        topic: [2u8; 16],
        vapid_sub: "mailto:ops@example.invalid".to_owned(),
        ledger_epoch: 0,
    }
}

fn sub_json(endpoint: &str) -> Vec<u8> {
    let secret = p256::SecretKey::from_slice(&[11u8; 32]).unwrap();
    let p256dh = secret.public_key().to_encoded_point(false);
    let auth = [42u8; 16];
    format!(
        r#"{{"endpoint":"{endpoint}","keys":{{"p256dh":"{}","auth":"{}"}}}}"#,
        Base64UrlUnpadded::encode_string(p256dh.as_bytes()),
        Base64UrlUnpadded::encode_string(&auth)
    )
    .into_bytes()
}

fn store_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("wps-loopback-ledger-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn evidence(name: &str, parts: &[&str]) -> String {
    let canonical = format!("{}|{}", name, parts.join("|"));
    let sha = hex(Sha256::digest(canonical.as_bytes()));
    println!("EVIDENCE {name} sha256={sha} canonical={canonical}");
    sha
}

fn hex<B: AsRef<[u8]>>(bytes: B) -> String {
    bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

fn assert_scoped(req: &[u8]) {
    assert_eq!(&req[3..35], CAP, "scoped sender cap present");
    assert_eq!(&req[37..69], NS, "namespace present");
}

fn auth_summary(s: &Seen) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        s.method,
        s.target,
        s.host,
        s.sni.as_deref().unwrap_or("-"),
        s.header("authorization").unwrap_or("-"),
        s.header("content-encoding").unwrap_or("-"),
        s.header("ttl").unwrap_or("-"),
        hex(Sha256::digest(&s.body))
    )
}

fn new_pump<'a>(
    cfg: &'a Config,
    p1: &'a p1::LoopbackP1,
    out: &'a LoopbackTransport,
    res: &'a FixedResolver,
    key: &'a VapidKey,
    rng: &'a FixedRng,
    clock: &'a TestClock,
    dir: &std::path::Path,
) -> WResult<Pump<'a, p1::LoopbackP1, LoopbackTransport, FixedResolver, FixedRng, TestClock>> {
    let binding = Sha256::digest(b"skrynia-webpush-ledger-v1-loopback-fixture");
    let mut b = [0u8; 32];
    b.copy_from_slice(&binding);
    let store = FileLedgerStore::new(dir, &b)?;
    Pump::with_ledger(Parts::new(cfg, p1, out, res, key, rng, clock), store)
}

fn report_str(r: &Report) -> String {
    format!("{}/{}/{}/{}", r.hints, r.delivered, r.acked, r.skipped)
}

fn main() {
    let mut results = Vec::new();

    // ---- Scenario A: loopback delivery, scoped bearer, subscribe/revoke,
    // durable hint/ACK and restart suppression. ----
    let svc = FakePushService::start(ReplyPlan::status(201));
    let out = LoopbackTransport {
        ca_der: svc.ca_der.clone(),
        port: svc.addr.port(),
        sent: Mutex::new(Vec::new()),
    };
    let p1 = p1::LoopbackP1::new(CAP, Vec::new(), Vec::new());
    let endpoint = format!("https://{HOST}/push/x");
    p1.subscribe(&CAP, &NS, KEY, b"subscriber-A", &sub_json(&endpoint));
    p1.hints.lock().unwrap().push(1);
    let cfg = config(3);
    let key = VapidKey::from_slice(&PRIV).unwrap();
    let rng = FixedRng;
    let clock = TestClock::new();
    let res = resolver();
    let dir = store_dir("A");
    let mut pump = new_pump(&cfg, &p1, &out, &res, &key, &rng, &clock, &dir).unwrap();
    let report = pump.run_once().unwrap();
    assert_eq!(
        report,
        Report {
            hints: 1,
            delivered: 1,
            acked: 1,
            skipped: 0
        }
    );
    let seen = svc.seen();
    assert_eq!(seen.len(), 1);
    assert!(
        seen[0]
            .header("authorization")
            .unwrap()
            .starts_with("vapid t=")
    );
    assert_eq!(seen[0].sni.as_deref(), Some(HOST));
    let p1calls = p1.calls.lock().unwrap();
    for c in p1calls.iter() {
        assert_scoped(c);
    }
    let opcodes: Vec<u8> = p1calls.iter().map(|c| c[0]).collect();
    drop(p1calls);
    assert_eq!(
        &opcodes[1..],
        &[
            p1::OP_HINT_LIST,
            p1::OP_HINT_READ,
            p1::OP_SUB_FETCH,
            p1::OP_HINT_ACK
        ]
    );
    assert_eq!(opcodes[0], p1::OP_SUBSCRIBE);
    assert!(p1.acked.lock().unwrap().contains(&1));
    let ledger_files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert!(!ledger_files.is_empty(), "durable ledger file persisted");
    let ledger_sha = hex(Sha256::digest(std::fs::read(&ledger_files[0]).unwrap()));
    results.push((
        "A_delivery_scoped_durable",
        evidence(
            "A_delivery_scoped_durable",
            &[
                &report_str(&report),
                &auth_summary(&seen[0]),
                &hex(&opcodes),
                &ledger_sha,
            ],
        ),
    ));

    // Restart: same grant/store, hint still queued -> ledger suppresses replay.
    let mut pump2 = new_pump(&cfg, &p1, &out, &res, &key, &rng, &clock, &dir).unwrap();
    let report2 = pump2.run_once().unwrap();
    assert_eq!(
        report2,
        Report {
            hints: 1,
            delivered: 0,
            acked: 0,
            skipped: 1
        }
    );
    results.push((
        "A2_restart_no_duplicate",
        evidence(
            "A2_restart_no_duplicate",
            &[&report_str(&report2), &svc.seen().len().to_string()],
        ),
    ));
    assert_eq!(svc.seen().len(), 1, "restart must not re-send");

    // Revoke: clear subscriptions, new seq -> no delivery, hint still ACKed.
    p1.revoke(&CAP, &NS, KEY, b"subscriber-A");
    p1.hints.lock().unwrap().push(2);
    let mut pump3 = new_pump(&cfg, &p1, &out, &res, &key, &rng, &clock, &dir).unwrap();
    let report3 = pump3.run_once().unwrap();
    assert_eq!(
        report3.delivered, 0,
        "revoked subscription must not be delivered"
    );
    assert!(p1.acked.lock().unwrap().contains(&2));
    results.push((
        "A3_revoke_stops_delivery",
        evidence(
            "A3_revoke_stops_delivery",
            &[&report_str(&report3), &hex(&p1.opcodes())],
        ),
    ));
    drop(svc);

    // ---- Scenario B: remote auth response rejection -> no ACK, no replay. ----
    let svc_b = FakePushService::start(ReplyPlan::status(401));
    let out_b = LoopbackTransport {
        ca_der: svc_b.ca_der.clone(),
        port: svc_b.addr.port(),
        sent: Mutex::new(Vec::new()),
    };
    let p1_b = p1::LoopbackP1::new(
        CAP,
        vec![7],
        vec![sub_json(&format!("https://{HOST}/push/b"))],
    );
    let dir_b = store_dir("B");
    let mut pump_b = new_pump(&cfg, &p1_b, &out_b, &res, &key, &rng, &clock, &dir_b).unwrap();
    let err_b = pump_b.run_once().unwrap_err();
    assert_eq!(err_b, Error::Transport("status"));
    assert!(
        !p1_b.acked.lock().unwrap().contains(&7),
        "rejected response must not be ACKed"
    );
    assert_eq!(svc_b.seen().len(), 1);
    results.push((
        "B_response_rejection",
        evidence(
            "B_response_rejection",
            &[
                &format!("{err_b:?}"),
                &svc_b.seen().len().to_string(),
                &hex(&p1_b.opcodes()),
            ],
        ),
    ));
    drop(svc_b);

    // ---- Scenario C: transient 503 + Retry-After -> bounded sender retry. ----
    let svc_c = FakePushService::start(ReplyPlan::status(503).retry(1));
    let out_c = LoopbackTransport {
        ca_der: svc_c.ca_der.clone(),
        port: svc_c.addr.port(),
        sent: Mutex::new(Vec::new()),
    };
    let p1_c = p1::LoopbackP1::new(
        CAP,
        vec![9],
        vec![sub_json(&format!("https://{HOST}/push/c"))],
    );
    let dir_c = store_dir("C");
    let mut pump_c = new_pump(&cfg, &p1_c, &out_c, &res, &key, &rng, &clock, &dir_c).unwrap();
    let err_c = pump_c.run_once().unwrap_err();
    assert_eq!(err_c, Error::Exhausted);
    assert_eq!(svc_c.seen().len(), 3, "3 bounded attempts");
    assert_eq!(
        clock.sleeps.lock().unwrap().len(),
        2,
        "backoff between 3 attempts"
    );
    results.push((
        "C_sender_retry",
        evidence(
            "C_sender_retry",
            &[
                &format!("{err_c:?}"),
                &svc_c.seen().len().to_string(),
                &hex(&p1_c.opcodes()),
            ],
        ),
    ));
    drop(svc_c);

    // ---- Scenario D: adversarial forged scoped bearer -> P1 rejects. ----
    let forged = p1::frame(&[0xEE; 32], &NS, KEY, p1::OP_HINT_LIST, &[]);
    let p1_d = p1::LoopbackP1::new(CAP, vec![1], Vec::new());
    let raw = p1_d.call(&forged).unwrap();
    assert_eq!(
        raw,
        vec![1, 1, 1, 0, 0, 0, 5],
        "wrong bearer rejected with P1 code 5"
    );
    let bad = webpush_sender::p1::decode(&raw);
    assert!(matches!(bad, Err(Error::P1(5))));
    results.push((
        "D_forged_bearer_rejected",
        evidence(
            "D_forged_bearer_rejected",
            &[&hex(&raw), &format!("{bad:?}")],
        ),
    ));

    // ---- Scenario E: no VAPID private key exposed to browser; test key disposable. ----
    let pub_bytes = key.public_bytes();
    assert_eq!(
        pub_bytes[0], 4,
        "public verification key is uncompressed point"
    );
    assert_ne!(
        &pub_bytes[1..33],
        &PRIV,
        "wire key is public, not private scalar"
    );
    let auth = svc_sent_auth(&out);
    assert!(auth.contains("k="), "authorization carries only public key");
    let priv_hex = hex(&PRIV);
    let priv_b64 = Base64UrlUnpadded::encode_string(&PRIV);
    let mut scanned = 0usize;
    for root in [
        "../src",
        "../webpush-sender/src",
        "../browser-package",
        "../v3-trusted-edge-gateway/src",
    ] {
        scan_tree(
            std::path::Path::new(root),
            &priv_hex,
            &priv_b64,
            &mut scanned,
        );
    }
    assert_eq!(
        scanned, 0,
        "no VAPID private key material in browser-facing sources"
    );
    results.push((
        "E_no_privkey_in_browser",
        evidence(
            "E_no_privkey_in_browser",
            &[&hex(&pub_bytes), &priv_hex, &scanned.to_string()],
        ),
    ));

    println!("RESULT disposability=loopback-only real_p1_wasm=UNTESTED browser_sw=UNTESTED");
    for (name, sha) in &results {
        println!("FINAL {name} {sha}");
    }
    println!("HARNESS_OK {}", results.len());
}

fn svc_sent_auth(out: &LoopbackTransport) -> String {
    out.sent
        .lock()
        .unwrap()
        .first()
        .map(|r| r.authorization.clone())
        .unwrap_or_default()
}

fn scan_tree(path: &std::path::Path, priv_hex: &str, priv_b64: &str, hits: &mut usize) {
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            scan_tree(&p, priv_hex, priv_b64, hits);
        } else if let Ok(bytes) = std::fs::read(&p) {
            let text = String::from_utf8_lossy(&bytes);
            if text.contains(priv_hex)
                || text.contains(priv_b64)
                || text.contains("PRIVATE KEY")
                || text.contains("vapid_private")
            {
                *hits += 1;
                eprintln!("PRIVATE KEY MATERIAL FOUND: {}", p.display());
            }
        }
    }
}
