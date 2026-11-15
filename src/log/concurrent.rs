use super::{AppendLog, tests::Fake};

#[test]
fn concurrent_writers_preserve_contiguous_slots() {
    let store = Fake::default();
    let log = AppendLog::open(&store, "concurrent.log").unwrap();
    log.initialize().unwrap();
    std::thread::scope(|scope| {
        for i in 1..=8u64 {
            let log = &log;
            scope.spawn(move || {
                let id = format!("{i:032x}");
                log.append(&id, id.as_bytes()).unwrap()
            });
        }
    });
    let entries = log.read_from(1, 9).unwrap();
    assert_eq!(entries.len(), 8);
    for (index, (position, _, _)) in entries.iter().enumerate() {
        assert_eq!(*position, index as u64 + 1);
    }
}
