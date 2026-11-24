use super::hlp::pw;
use super::setup::{hints, wake_setup};
use skrynia_native_service::read_cap;
use skrynia_native_service::registry::appdata::push;

#[test]
fn offline_sender_does_not_block_over_a_thousand_crossns_mutations() {
    let (disk, wa, wb, ga, gb) = wake_setup();
    let key = push::WAKE_KEY;
    for i in 0..(push::MAX_HINTS + 64) {
        let (writer, grant) = if i % 2 == 0 { (&wa, &ga) } else { (&wb, &gb) };
        let object = format!("thread-{i:04}-tes-v1");
        pw(&disk, writer, object.as_bytes(), b"message", grant, key).unwrap();
    }
    let wake_ns = read_cap(&[1u8; 32]);
    assert_eq!(hints(&disk, &wake_ns, key), push::MAX_HINTS + 64);
    assert!(
        !disk
            .0
            .lock()
            .unwrap()
            .keys()
            .any(|path| { path.starts_with(&push::hints_root("d", &wake_ns, key)) })
    );
}
