use super::*;
#[test]
fn known_mutations_delegate_to_authorized_backend() {
    let memory = Memory::default();
    for (method, status, pending) in [
        ("POST", 201, true),
        ("PUT", 200, false),
        ("DELETE", 200, false),
    ] {
        let mut r = req(method);
        if method == "DELETE" {
            r.body.clear();
        }
        let res = handle(&memory, &r);
        assert_eq!(res.status, status);
        assert!(
            std::str::from_utf8(&res.body)
                .unwrap()
                .contains(if pending {
                    "notificationPending"
                } else {
                    "\"ok\":true"
                })
        );
    }
    assert_eq!(memory.0.lock().unwrap().len(), 3);
}
#[test]
fn rejected_preconditions_never_mutate() {
    let memory = Memory::default();
    for (method, name, value) in [
        ("POST", "If-Match", "*"),
        ("POST", "X-Skrynia-If-SHA256", &"a".repeat(64)),
        ("PUT", "X-Skrynia-If-SHA256", "xyz"),
    ] {
        let mut r = req(method);
        r.headers.insert(name.into(), value.into());
        assert_eq!(handle(&memory, &r).status, 400);
    }
    assert!(memory.0.lock().unwrap().is_empty());
}
