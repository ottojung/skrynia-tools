//! Spawns the real service binary with private loopback roots.

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use tempfile::TempDir;

pub struct Fixture {
    pub _dir: TempDir,
    pub child: Child,
    pub base: String,
    pub requests: PathBuf,
    pub disk: PathBuf,
    pub nginx: PathBuf,
}

impl Fixture {
    pub fn start() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let requests = dir.path().join("requests");
        let disk = dir.path().join("disk");
        let nginx = dir.path().join("nginx");
        for path in [&requests, &disk, &nginx] {
            std::fs::create_dir(path).expect("root dir");
        }
        let probe = TcpListener::bind("127.0.0.1:0").expect("probe");
        let port = probe.local_addr().expect("addr").port();
        drop(probe);
        let child = Command::new(env!("CARGO_BIN_EXE_skrynia-publish-provider"))
            .env("PUBLICATION_PROVIDER_BIND", format!("127.0.0.1:{port}"))
            .env("PATH_TO_PUBLICATION_REQUESTS", &requests)
            .env("PATH_TO_SKRYNIA_DISK", &disk)
            .env("PATH_TO_NGINX_SKRYNIA_ROOT", &nginx)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn provider");
        let fixture = Self {
            _dir: dir,
            child,
            base: format!("http://127.0.0.1:{port}"),
            requests,
            disk,
            nginx,
        };
        super::wait_ready(&fixture.base);
        fixture
    }
}
