//! Real HTTP test: P1 opcodes 20..=23 over the pinned loopback edge.
use super::cases::{CAP, HOST, ORIGIN, cfg, edge, res};
use super::{HttpP1, MockEdge, Reply, Seen};

#[test]
fn opcodes_20_through_23_over_real_http() {
    let mock = MockEdge::start(Reply::Ok(res(0, b"ok")), HOST, ORIGIN, CAP, 4);
    let io = HttpP1::new(edge(mock.port, HOST, ORIGIN, CAP)).unwrap();
    let config = cfg();
    let client = super::Client::new(&config, &io).unwrap();
    client.hint_list().unwrap();
    client.hint_read(7).unwrap();
    client.hint_ack(7).unwrap();
    client.sub_fetch().unwrap();
    let seen = mock.seen();
    assert_eq!(seen.len(), 4);
    let ops: Vec<u8> = seen.iter().map(|s| s.body[0]).collect();
    assert_eq!(ops, vec![20, 21, 22, 23]);
    for request in &seen {
        assert_eq!(request.request_line, "POST /_skrynia/ HTTP/1.1");
        assert!(!has_root_token_marker(request));
    }
}

fn has_root_token_marker(s: &Seen) -> bool {
    let planted = s
        .raw
        .windows(33)
        .any(|w| w[0] == 1 && w[1..] == super::cases::SENTINEL);
    let labeled = s
        .headers
        .iter()
        .any(|(k, _)| k.contains("operator") || k.contains("p0-token"));
    planted || labeled
}
