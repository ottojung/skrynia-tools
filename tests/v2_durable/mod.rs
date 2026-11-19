mod admin_http;
mod bridge;
mod disk;
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
