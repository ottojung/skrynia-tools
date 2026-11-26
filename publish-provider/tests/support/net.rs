//! Real HTTP client helpers shared by integration tests.

use reqwest::blocking::Client;
use std::time::Duration;

pub fn client() -> Client {
    Client::builder().no_proxy().build().expect("http client")
}

pub fn wait_ready(base: &str) {
    for _ in 0..100 {
        if client().get(format!("{base}/syscalls_list")).send().is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("provider did not start");
}

pub fn call_at(base: &str, name: &str, ticket: u32) -> u32 {
    let response = client()
        .post(format!("{base}/{name}"))
        .body(skrynia_publish_provider::abi::encode(ticket))
        .send()
        .expect("post syscall");
    assert_eq!(response.status(), 200);
    let body = response.bytes().expect("response body");
    skrynia_publish_provider::abi::decode(&body).expect("typed i32 result")
}
