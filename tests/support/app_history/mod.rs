use super::Fixture;
mod checks;
mod compare;
mod doctor;
mod recovery;

pub fn verify() {
    let fixture = Fixture::new();
    let one = fixture.commit("version one");
    let first = fixture.deploy(&one);
    assert!(first.status.success());
    let first: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    let two = fixture.commit("version two");
    assert!(fixture.deploy(&two).status.success());
    let three = fixture.commit("version three");
    assert!(fixture.deploy(&three).status.success());
    checks::verify(&fixture, &first, &one, &three);
    compare::verify(&fixture, &first, &one, &three);
    doctor::verify(&fixture);
    recovery::verify(&fixture, &first);
}
