use skrynia_tools::wire::v2compat::Request;
use skrynia_v2_provider::{Callback, InvokeError, dispatch};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Mutex,
};
const TOKEN: &str = "0000000000000000000000000000000000000000000000000000000000000000";
struct Store(Mutex<HashMap<String, Vec<u8>>>);
impl Callback for Store {
    fn invoke(&self, bytes: &[u8]) -> Result<Vec<u8>, InvokeError> {
        if bytes.len() < 3 {
            return Err(InvokeError::Unavailable);
        }
        let len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
        if bytes.len() < 3 + len {
            return Err(InvokeError::Unavailable);
        }
        let key = std::str::from_utf8(&bytes[3..3 + len]).map_err(|_| InvokeError::Unavailable)?;
        let mut map = self.0.lock().unwrap();
        match bytes[0] {
            1 => Ok(match map.get(key) {
                Some(b) => [vec![1], b.clone()].concat(),
                None => vec![0],
            }),
            2 => {
                map.insert(key.into(), bytes[3 + len..].to_vec());
                Ok(vec![0])
            }
            3 => {
                map.remove(key);
                Ok(vec![0])
            }
            _ => Err(InvokeError::Unavailable),
        }
    }
}
fn req(method: &str, path: &str, body: Vec<u8>) -> Request {
    let mut h = BTreeMap::new();
    h.insert("authorization".into(), format!("Bearer {TOKEN}"));
    Request {
        method: method.into(),
        path: path.into(),
        query: None,
        headers: h,
        body,
    }
}
#[test]
fn delegated_positive_create_cas_replace_and_delete() {
    let store = Store(Mutex::new(HashMap::new()));
    let c = dispatch(
        &req("POST", "/store/test/doc.txt", b"alpha".to_vec()),
        &store,
    );
    assert_eq!(c.status, 201);
    let g = dispatch(
        &Request {
            method: "GET".into(),
            path: "/store/test/doc.txt".into(),
            query: None,
            headers: BTreeMap::new(),
            body: vec![],
        },
        &store,
    );
    assert_eq!(g.status, 200);
    assert_eq!(g.body, b"alpha");
    let etag = g.headers.get("etag").unwrap().trim_matches('"');
    let mut bad = req("PUT", "/store/test/doc.txt", b"beta".to_vec());
    bad.headers
        .insert("x-skrynia-if-sha256".into(), "0".repeat(64));
    assert_eq!(dispatch(&bad, &store).status, 412);
    let mut good = req("PUT", "/store/test/doc.txt", b"beta".to_vec());
    good.headers
        .insert("x-skrynia-if-sha256".into(), etag.into());
    assert_eq!(dispatch(&good, &store).status, 200);
    assert_eq!(
        dispatch(
            &Request {
                method: "GET".into(),
                path: "/store/test/doc.txt".into(),
                query: None,
                headers: BTreeMap::new(),
                body: vec![]
            },
            &store
        )
        .body,
        b"beta"
    );
    assert_eq!(
        dispatch(&req("DELETE", "/store/test/doc.txt", vec![]), &store).status,
        200
    );
    assert_eq!(
        dispatch(
            &Request {
                method: "GET".into(),
                path: "/store/test/doc.txt".into(),
                query: None,
                headers: BTreeMap::new(),
                body: vec![]
            },
            &store
        )
        .status,
        404
    );
}
