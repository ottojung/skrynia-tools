use super::{AppendLog, CreateResult, Store};
use anyhow::Result;
use std::sync::Mutex;

#[derive(Default)]
pub(super) struct Fake(Mutex<std::collections::BTreeMap<String, Vec<u8>>>);
impl Store for Fake {
    fn get(&self, k: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.0.lock().unwrap().get(k).cloned())
    }
    fn create(&self, k: &str, v: &[u8]) -> Result<CreateResult> {
        let mut map = self.0.lock().unwrap();
        if map.contains_key(k) {
            return Ok(CreateResult::Exists);
        }
        map.insert(k.to_owned(), v.to_vec());
        Ok(CreateResult::Created)
    }
}
#[test]
fn append_contiguous_and_read() {
    let fake = Fake::default();
    let log = AppendLog::open(&fake, "test.log").unwrap();
    log.initialize().unwrap();
    assert_eq!(
        log.append("00000000000000000000000000000001", b"hello")
            .unwrap(),
        1
    );
    assert_eq!(
        log.append("00000000000000000000000000000002", b"world")
            .unwrap(),
        2
    );
    assert_eq!(
        log.append("00000000000000000000000000000001", b"hello")
            .unwrap(),
        1
    );
    assert!(
        log.append("00000000000000000000000000000001", b"changed")
            .is_err()
    );
    let all = log.read_from(1, 5).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[1].2, b"world");
}
