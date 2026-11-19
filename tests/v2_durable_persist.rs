#[path = "v2_durable_fakes.rs"]
mod fakes;
#[path = "v2_durable_run.rs"]
mod run;
#[path = "v2_durable_setup.rs"]
mod setup;
#[path = "v2_durable_verify.rs"]
mod verify;

#[test]
fn durable_restart() {
    run::go();
}
