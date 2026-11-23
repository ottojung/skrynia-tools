use super::super::p1::must;
use super::super::wire::{blb, idb, req};
use super::ctx::Ctx;
use crate::provider;
use webpush_sender::pump::Report;

pub fn verify(c: &Ctx, rep: &Report, sent: usize) -> Vec<String> {
    assert_eq!(
        *rep,
        Report {
            hints: 1,
            delivered: 1,
            acked: 1,
            skipped: 0
        }
    );
    assert_eq!(sent, 1);
    let seen = c.push.seen();
    assert_eq!(seen.len(), 1, "provider saw exactly one push");
    let v =
        provider::check(&seen[0], &c.sub_keys, "https://push.example.test").expect("vapid+decrypt");
    assert_eq!(v.plaintext.len(), 32);
    assert_eq!(seen[0].header("content-encoding"), Some("aes128gcm"));
    assert_eq!(seen[0].header("ttl"), Some("60"));
    let list = must(
        &c.io,
        &req(20, &[&idb(&c.sender_cap), &idb(&c.ns), &blb(&c.key)]),
        "list",
    );
    assert_eq!(list, 0u32.to_le_bytes(), "hints acked");
    vec![
        format!(
            "report={}/{}/{}/{}",
            rep.hints, rep.delivered, rep.acked, rep.skipped
        ),
        format!("plaintext_hex={}", hex(&v.plaintext)),
        format!("vapid_sub={}", v.vapid_sub),
        format!("push_seen={}", seen.len()),
        "hint_list_empty=true".into(),
    ]
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
