use super::parity_sup::{G, I, sreq};
use super::{Mem, api};
#[test]
fn subscribe_rejects_scope_and_bad_input() {
    let mut s = Mem::default();
    let good = serde_json::json!({"locations":"apps/x/*","endpoint":"https://p/1","keys":{"p256dh":"k","auth":"a"}});
    let r = api::subscribe(
        &mut s,
        &G,
        &I(1),
        &sreq(
            &"b".repeat(64),
            serde_json::json!({"locations":"other/*","endpoint":"https://p/1","keys":{"p256dh":"k","auth":"a"}}),
        ),
    );
    assert_eq!(r.status, 403);
    let bad = serde_json::json!({"locations":"apps/x/*","endpoint":"http://p/1","keys":{"p256dh":"k","auth":"a"}});
    assert_eq!(
        api::subscribe(&mut s, &G, &I(1), &sreq(&"b".repeat(64), bad)).status,
        400
    );
    assert_eq!(
        api::subscribe(&mut s, &G, &I(1), &sreq(&"b".repeat(64), good)).status,
        201
    );
}
