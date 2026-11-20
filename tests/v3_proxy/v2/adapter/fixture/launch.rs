use super::Gateway;
use std::{
    fs,
    net::{TcpListener, TcpStream},
    process::{Command, Stdio},
    time::Duration,
};
impl Gateway {
    pub fn start(upstream: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "skrynia-v2-gateway-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        fs::create_dir_all(&dir).unwrap();
        let token_path = super::token::create(&dir);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let child = Command::new(std::env::var("SKRYNIA_V2_GATEWAY_BIN").unwrap())
            .env("SKRYNIA_V2_GATEWAY_BIND", addr.to_string())
            .env("SKRYNIA_V3_URL", upstream)
            .env("SKRYNIA_OPERATOR_TOKEN_FILE", &token_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut gateway = Self {
            url: format!("http://{addr}"),
            token_path,
            child,
            dir,
        };
        for _ in 0..500 {
            if TcpStream::connect(addr).is_ok() {
                return gateway;
            }
            assert!(
                gateway.child.try_wait().unwrap().is_none(),
                "gateway crashed"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("gateway did not start")
    }
}
