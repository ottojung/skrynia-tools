mod create;
mod replace;
use super::{cases, parent_fixture::Server};
use skrynia_tools::wire::v2compat::{Request, Response};
use std::{
    sync::{Arc, Barrier},
    thread,
};
fn race(s: &Server, requests: [Request; 2]) -> Vec<u16> {
    let gate = Arc::new(Barrier::new(3));
    let handles: Vec<_> = requests
        .into_iter()
        .map(|request| {
            let gate = gate.clone();
            let url = s.url.clone();
            thread::spawn(move || {
                let body = serde_json::to_vec(&request).unwrap();
                gate.wait();
                let http = cases::post(&url, &cases::authenticated(&body, &[0x42; 32]));
                assert_eq!(http.status(), 200);
                let answer: Response = serde_json::from_slice(&http.bytes().unwrap()).unwrap();
                answer.status
            })
        })
        .collect();
    gate.wait();
    handles.into_iter().map(|h| h.join().unwrap()).collect()
}
