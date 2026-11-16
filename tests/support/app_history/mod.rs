use super::Fixture;
mod audit;
#[cfg(unix)]
mod broken_active;
mod checks;
mod compare;
mod diff;
mod doctor;
mod generation;
mod recovery;
mod verify_all;

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
    diff::verify(&fixture, &first);
    generation::verify(&fixture, &first);
    recovery::verify(&fixture, &first);
    verify_all::verify(&fixture, &first);
    audit::verify(&fixture, &first);
    #[cfg(unix)]
    broken_active::verify(&fixture);
}
