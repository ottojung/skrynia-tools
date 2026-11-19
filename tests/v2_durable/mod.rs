mod admin_http;
mod disk;
mod failure;
mod fakes;
mod faultstore;
mod push;
mod run;
mod setup;
mod verify;

#[test]
fn durable_restart() {
    run::go();
}
