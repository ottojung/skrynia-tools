//! Test-only sender bridge: attach the real sender to an EXTERNAL scoped edge.
//! No kernel/edge is spawned here; the caller owns the single shared P1 stack.
mod config;
mod edge;
mod private_output;
mod pump;

use std::io::Write;

pub fn execute() {
    let cfg = config::load();
    // Reserve before any sender effects: a conflicting output must not
    // consume/acknowledge the hint needed by the positive test.
    let mut output = private_output::reserve(&cfg.out).expect("reserve private hint output");
    let io = webpush_sender::p1::HttpP1::new(edge::build(&cfg)).expect("external edge io");
    let push = crate::FakePush::start(crate::ReplyPlan::status(201));
    let out = crate::LoopbackPush::new(push.ca_der.clone(), push.addr.port());
    let rep = pump::run(&cfg, &io, &out);
    let seen = push.seen();
    assert_eq!(seen.len(), 1, "controlled endpoint saw exactly one push");
    let (_, keys) = crate::fixture();
    let v = crate::check(&seen[0], &keys, "https://push.example.test").expect("vapid+decrypt");
    assert_eq!(v.plaintext.len(), 32, "opaque hint must be 32 bytes");
    let hex = v
        .plaintext
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let body = format!(
        "plaintext_hex={hex}\nvapid_sub={}\nreport={}/{}/{}/{}\n",
        v.vapid_sub, rep.hints, rep.delivered, rep.acked, rep.skipped
    );
    output
        .write_all(body.as_bytes())
        .expect("write private decrypted hint");
    // The plaintext is written only to the new private 0600 temp output.
    // Never echo the raw hint to stdout or CI logs; digests/verdicts only.
    println!("bridge_vapid_sub={}", v.vapid_sub);
    println!("bridge_vapid_es256_verify=ok");
    println!("bridge_aes128gcm_decrypt=ok");
    println!(
        "bridge_report={}/{}/{}/{}",
        rep.hints, rep.delivered, rep.acked, rep.skipped
    );
    println!("BRIDGE_OK");
}
