use super::http::{B, call, json};
use serde_json::json;

fn post(b: &B, key: &str, body: Vec<u8>, t: &str) -> u16 {
    call(b, "POST", &format!("/store/{key}"), None, t, body, vec![]).status
}
pub fn seed(b: &B, t: &str, vault: &[u8]) {
    let vkey = "apps/tools/vault/00112233445566778899aabbccddeeff.json";
    assert_eq!(post(b, vkey, vault.to_vec(), t), 201);
    for app in ["alpha", "beta"] {
        for rel in [
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ] {
            assert_eq!(
                post(
                    b,
                    &format!("apps/{app}/versions/{rel}.ready"),
                    b"ready".to_vec(),
                    t
                ),
                201
            );
            let m = json!({"schema":1,"name":app,"id":rel,"files":[]});
            assert_eq!(
                post(b, &format!("apps/{app}/versions/{rel}.json"), json(&m), t),
                201
            );
        }
        let a = json!({"schema":1,"release":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","nonce":"n1"});
        assert_eq!(
            post(b, &format!("apps/{app}/active.json"), json(&a), t),
            201
        );
    }
}
