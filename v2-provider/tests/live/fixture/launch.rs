use std::{
    net::TcpListener,
    process::{Child, Command, Stdio},
    time::Duration,
};
pub struct Server {
    pub url: String,
    pub child: Child,
    pub dir: std::path::PathBuf,
}
impl Server {
    pub fn new() -> Self {
        let bin = std::env::var("SKRYNIA_V3_SERVER_BIN").unwrap();
        let guest = std::env::var("SKRYNIA_V2_PROVIDER_WASM").unwrap();
        let dir = std::env::temp_dir().join(format!(
            "skrynia-v2guest-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        super::seed::data(&dir, &guest);
        let listen = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listen.local_addr().unwrap();
        drop(listen);
        let child = Command::new(bin)
            .env("SKRYNIA_ROOT_WASM", dir.join("root.wasm"))
            .env("SKRYNIA_DATA_DIR", dir.join("data"))
            .env("SKRYNIA_BIND", addr.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut s = Self {
            url: format!("http://{addr}/"),
            child,
            dir,
        };
        for _ in 0..500 {
            if std::net::TcpStream::connect(addr).is_ok() {
                return s;
            }
            assert!(s.child.try_wait().unwrap().is_none(), "kernel exited");
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("kernel did not become ready")
    }
}
