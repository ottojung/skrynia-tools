use super::super::{State, client, upstream};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
#[tokio::test]
async fn cannot_forward_root_bearer_to_untrusted_redirect_destination() {
    let leaked = Arc::new(AtomicBool::new(false));
    let flag = leaked.clone();
    let target = axum::Router::new().fallback(move || async move {
        flag.store(true, Ordering::SeqCst);
        "exfiltrated"
    });
    let listen = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_url = format!("http://{}/", listen.local_addr().unwrap());
    let task_target = tokio::spawn(axum::serve(listen, target).into_future());
    let redirect = axum::Router::new().fallback(move || async move {
        (
            axum::http::StatusCode::TEMPORARY_REDIRECT,
            [("location", target_url)],
            "redirect",
        )
    });
    let listen = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}/", listen.local_addr().unwrap());
    let task_origin = tokio::spawn(axum::serve(listen, redirect).into_future());
    let state = State {
        bind: "127.0.0.1:1234".parse().unwrap(),
        endpoint: origin.parse().unwrap(),
        token_path: PathBuf::new(),
        client: client::build().unwrap(),
    };
    let payload = vec![0x42u8; 33];
    let answer = upstream::post(&state, payload).await;
    assert_eq!(answer, Err(axum::http::StatusCode::BAD_GATEWAY));
    assert!(!leaked.load(Ordering::SeqCst));
    task_origin.abort();
    task_target.abort();
}
