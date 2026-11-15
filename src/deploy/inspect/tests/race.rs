use super::*;

#[test]
fn delayed_deployment_must_not_override_newer_activation() {
    let fixture = Fixture::new();
    let before = fixture.add(1);
    let manually_restored = fixture.add(2);
    let prepared = fixture.add(3);
    fixture.active(&before.id);
    let expected = inspect(&fixture.root, "hello").unwrap().unwrap().id;
    fixture.active(&manually_restored.id);
    assert!(publish::activate(&fixture.root, "hello", Some(&expected), &prepared.id).is_err());
    assert_eq!(
        inspect(&fixture.root, "hello").unwrap().unwrap().id,
        manually_restored.id
    );
}
