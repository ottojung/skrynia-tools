use super::*;

#[test]
fn even_an_a_b_a_activation_blocks_stale_build() {
    let fixture = Fixture::new();
    let initial = fixture.add(1);
    let other = fixture.add(2);
    let prepared = fixture.add(3);
    fixture.active(&initial.id);
    let (expected, generation) = publish::observe(&fixture.root, "hello").unwrap();
    fixture.active(&other.id);
    fixture.active(&initial.id);
    assert_eq!(
        files::generation::current(&fixture.root, "hello").unwrap(),
        generation + 2
    );
    let current = inspect(&fixture.root, "hello").unwrap().unwrap().id;
    assert_eq!(current, initial.id);
    assert!(
        publish::activate(
            &fixture.root,
            "hello",
            expected.as_deref(),
            generation,
            &prepared.id
        )
        .is_err()
    );
    assert_eq!(
        inspect(&fixture.root, "hello").unwrap().unwrap().id,
        initial.id
    );
}
