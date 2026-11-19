mod bridge;
mod failure;
mod fakes;
mod faultstore;
mod run;
mod setup;
mod verify;

#[test]
fn durable_restart() {
    run::go();
}
