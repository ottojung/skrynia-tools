use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

fn echo_wasm() -> Vec<u8> {
    wat::parse_str(
        r#"(module
(import "skrynia" "read" (func $read(param i32 i32 i32)(result i32)))
(import "skrynia" "write" (func $write(param i32 i32 i32)(result i32)))
(memory(export "memory") 1)
(func(export "skrynia_main")(param $input i32)(param $output i32)(result i32)
 (local $n i32)(local $sent i32)(local $wrote i32)
 (block $eof (loop $read_more
  (local.set $n (call $read(local.get $input)(i32.const 0)(i32.const 65536)))
  (if (i32.lt_s (local.get $n) (i32.const 0))
   (then(return(local.get $n))))
  (br_if $eof (i32.eqz(local.get $n)))
  (local.set $sent (i32.const 0))
  (block $done (loop $write_more
   (br_if $done (i32.ge_s(local.get $sent)(local.get $n)))
   (local.set $wrote(call $write(local.get $output)(local.get $sent)
    (i32.sub(local.get $n)(local.get $sent))))
   (if (i32.le_s(local.get $wrote)(i32.const 0))
    (then(return(i32.const -5))))
   (local.set $sent(i32.add(local.get $sent)(local.get $wrote)))
   (br $write_more)))
  (br $read_more)))
 (i32.const 0)))"#,
    )
    .unwrap()
}

fn kit() -> Option<(String, String)> {
    let bin = std::env::var("SKRYNIA_V3_SERVER_BIN").ok()?;
    let p0 = std::env::var("SKRYNIA_V3_PROGRAM0_WASM").ok()?;
    if !Path::new(&bin).exists() || !Path::new(&p0).exists() {
        return None;
    }
    Some((bin, p0))
}

fn free_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
}

fn wait_ready(addr: SocketAddr, child: &mut Child) {
    for _ in 0..500 {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        assert!(child.try_wait().unwrap().is_none(), "process exited early");
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("did not become ready: {addr}");
}

struct Kernel {
    child: Child,
    _dir: tempfile::TempDir,
    addr: SocketAddr,
}

impl Kernel {
    fn start(bin: &str, p0_wasm: &str, token: [u8; 32]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("programs")).unwrap();
        std::fs::create_dir_all(dir.path().join("security")).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                dir.path().join("security"),
                std::fs::Permissions::from_mode(0o700),
            )
            .unwrap();
        }
        let digest = <sha2::Sha256 as sha2::Digest>::digest(token);
        let sec = dir.path().join("security/program0.sha256");
        std::fs::write(&sec, digest).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&sec, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        std::fs::write(dir.path().join("programs/root.wasm"), echo_wasm()).unwrap();

        let port = free_port();
        let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
        let mut child = Command::new(bin)
            .env("SKRYNIA_ROOT_WASM", p0_wasm)
            .env("SKRYNIA_BIND", addr.to_string())
            .env("SKRYNIA_DATA_DIR", dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start kernel");
        wait_ready(addr, &mut child);
        Kernel {
            child,
            _dir: dir,
            addr,
        }
    }
    fn url(&self) -> String {
        format!("http://{}/", self.addr)
    }
}
impl Drop for Kernel {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Gateway {
    child: Child,
    _dir: tempfile::TempDir,
    addr: SocketAddr,
}

impl Gateway {
    fn start(kernel_url: &str, token: [u8; 32], apps: serde_json::Value, max_body: usize) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let token_path = dir.path().join("operator.token");
        std::fs::write(&token_path, token).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&token_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        let port = free_port();
        let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
        let cfg = serde_json::json!({
            "bind": addr.to_string(),
            "upstream": kernel_url,
            "operator_token_file": token_path.to_string_lossy(),
            "max_body_bytes": max_body,
            "apps": apps
        });
        let cfg_path = dir.path().join("config.json");
        std::fs::write(&cfg_path, serde_json::to_vec(&cfg).unwrap()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&cfg_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        let bin = env!("CARGO_BIN_EXE_skrynia-v3-trusted-edge-gateway");
        let mut child = Command::new(bin)
            .env("SKRYNIA_EDGE_GATEWAY_CONFIG", &cfg_path)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start gateway");
        wait_ready(addr, &mut child);
        Gateway {
            child,
            _dir: dir,
            addr,
        }
    }
    fn stderr_text(&mut self) -> String {
        // A running gateway keeps stderr open; wait for EOF only after stopping it.
        let _ = self.child.kill();
        let _ = self.child.wait();
        let mut s = String::new();
        if let Some(mut e) = self.child.stderr.take() {
            let _ = e.read_to_string(&mut s);
        }
        s
    }
}
impl Drop for Gateway {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn post_raw(
    addr: SocketAddr,
    host: &str,
    origin: Option<&str>,
    capability: Option<&str>,
    body: &[u8],
) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(120))).ok();
    stream
        .set_write_timeout(Some(Duration::from_secs(120)))
        .ok();
    let mut req = format!(
        "POST /_skrynia/ HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    if let Some(o) = origin {
        req.push_str(&format!("Origin: {o}\r\n"));
    }
    if let Some(c) = capability {
        req.push_str(&format!("X-Skrynia-Edge-Capability: {c}\r\n"));
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes()).unwrap();
    stream.write_all(body).unwrap();
    let mut resp = Vec::new();
    let _ = stream.read_to_end(&mut resp);
    parse_http(&resp)
}

// Test genuine full duplex: send request and drain response concurrently.
fn post_raw_duplex(
    addr: SocketAddr,
    host: &str,
    origin: &str,
    capability: &str,
    body: &[u8],
) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let mut reader = TcpStream::connect(addr).unwrap();
    reader
        .set_read_timeout(Some(Duration::from_secs(40)))
        .unwrap();
    reader
        .set_write_timeout(Some(Duration::from_secs(40)))
        .unwrap();
    let mut sender = reader.try_clone().unwrap();
    let header = format!(
        "POST /_skrynia/ HTTP/1.1\r\nHost: {host}\r\nOrigin: {origin}\r\nX-Skrynia-Edge-Capability: {capability}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let mut response = Vec::new();
    std::thread::scope(|scope| {
        let writer = scope.spawn(move || {
            sender.write_all(header.as_bytes()).unwrap();
            sender.write_all(body).unwrap();
            // Do not half-close HTTP/1.1 before the streaming response completes.
            // Content-Length already delimits the request body.
        });
        reader.read_to_end(&mut response).unwrap();
        writer.join().unwrap();
    });
    parse_http(&response)
}

fn parse_http(resp: &[u8]) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let sep = resp
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("http header end");
    let head = std::str::from_utf8(&resp[..sep]).unwrap();
    let mut lines = head.lines();
    let status = lines
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    let mut headers = Vec::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
        }
    }
    let mut body = resp[sep + 4..].to_vec();
    let chunked = headers
        .iter()
        .any(|(k, v)| k == "transfer-encoding" && v.to_ascii_lowercase().contains("chunked"));
    if chunked {
        body = dechunk(&body);
    }
    (status, headers, body)
}

fn dechunk(data: &[u8]) -> Vec<u8> {
    let mut i = 0;
    let mut out = Vec::new();
    loop {
        // read chunk size line
        let mut j = i;
        while j + 1 < data.len() && &data[j..j + 2] != b"\r\n" {
            j += 1;
        }
        if j + 1 > data.len() {
            break;
        }
        let line = std::str::from_utf8(&data[i..j]).unwrap_or("");
        let size_str = line.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_str, 16).unwrap_or(0);
        i = j + 2;
        if size == 0 {
            break;
        }
        if i + size > data.len() {
            out.extend_from_slice(&data[i..]);
            break;
        }
        out.extend_from_slice(&data[i..i + size]);
        i += size;
        if i + 2 <= data.len() && &data[i..i + 2] == b"\r\n" {
            i += 2;
        }
    }
    out
}

fn five_apps(cap: &str) -> serde_json::Value {
    serde_json::json!([
        {"name":"along","hosts":["long.a.vau.place"],"origins":["https://long.a.vau.place"],"edge_capabilities":[cap]},
        {"name":"kawunapp","hosts":["kawun.a.vau.place"],"origins":["https://kawun.a.vau.place"],"edge_capabilities":[cap]},
        {"name":"tesicca","hosts":["tesicca.a.vau.place"],"origins":["https://tesicca.a.vau.place"],"edge_capabilities":[cap]},
        {"name":"theorem-lab","hosts":["theorem-lab.a.vau.place"],"origins":["https://theorem-lab.a.vau.place"],"edge_capabilities":[cap]},
        {"name":"sbp","hosts":["sbp.a.vau.place"],"origins":["https://sbp.a.vau.place"],"edge_capabilities":[cap]}
    ])
}

#[test]
fn live_kernel_p0_guest_gateway_negative_and_positive() {
    let Some((bin, p0)) = kit() else {
        eprintln!("SKIP: SKRYNIA_V3_SERVER_BIN and SKRYNIA_V3_PROGRAM0_WASM required");
        return;
    };
    let token = [0x42u8; 32];
    let edge_cap = "edge-cap-secret-long-0001";
    let kernel = Kernel::start(&bin, &p0, token);
    let mut gw = Gateway::start(&kernel.url(), token, five_apps(edge_cap), 16 * 1024 * 1024);

    // Positive
    let (st, _, body) = post_raw(
        gw.addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(edge_cap),
        b"hello-opaque",
    );
    assert_eq!(st, 200, "positive");
    assert_eq!(body, b"hello-opaque");

    // Negative: unauthenticated
    let (st, _, _) = post_raw(
        gw.addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        None,
        b"x",
    );
    assert!(st == 401 || st == 403, "missing capability");
    let (st, _, _) = post_raw(
        gw.addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some("wrong"),
        b"x",
    );
    assert_eq!(st, 401, "wrong capability");

    // Negative: wrong host / wrong origin / sibling cross-origin
    let (st, _, _) = post_raw(
        gw.addr,
        "evil.example",
        Some("https://long.a.vau.place"),
        Some(edge_cap),
        b"x",
    );
    assert_eq!(st, 403, "wrong host");
    let (st, _, _) = post_raw(
        gw.addr,
        "long.a.vau.place",
        Some("https://evil.example"),
        Some(edge_cap),
        b"x",
    );
    assert_eq!(st, 403, "wrong origin");
    let (st, _, _) = post_raw(
        gw.addr,
        "kawun.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(edge_cap),
        b"x",
    );
    assert_eq!(st, 403, "sibling mismatch");

    // No secret leakage in logs (stderr)
    // give a moment for any buffered stderr
    std::thread::sleep(Duration::from_millis(50));
    let stderr = gw.stderr_text();
    let token_hex: String = token.iter().map(|b| format!("{b:02x}")).collect();
    assert!(
        !stderr.contains(edge_cap),
        "edge capability leaked to stderr: {stderr}"
    );
    assert!(
        !stderr.contains(&token_hex),
        "operator token hex leaked to stderr"
    );
    // raw token bytes unlikely printable, but ensure obvious marker not present
    assert!(
        !stderr.contains("operator.token"),
        "token file path leakage check context: {stderr}"
    );
}

#[test]
fn live_kernel_p0_guest_gateway_13mib_full_duplex() {
    let Some((bin, p0)) = kit() else {
        eprintln!("SKIP: SKRYNIA_V3_SERVER_BIN and SKRYNIA_V3_PROGRAM0_WASM required");
        return;
    };
    let token = [0x42u8; 32];
    let edge_cap = "edge-cap-secret-13m";
    let kernel = Kernel::start(&bin, &p0, token);
    let gw = Gateway::start(&kernel.url(), token, five_apps(edge_cap), 16 * 1024 * 1024);

    let size = 13 * 1024 * 1024;
    let mut payload = vec![0u8; size];
    for (i, b) in payload.iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
    let (st, _, body) = post_raw_duplex(
        gw.addr,
        "long.a.vau.place",
        "https://long.a.vau.place",
        edge_cap,
        &payload,
    );
    assert_eq!(st, 200, "13MiB positive");
    assert_eq!(body.len(), payload.len(), "13MiB length");
    assert_eq!(body, payload, "13MiB full-duplex echo integrity");
    let _ = (gw, kernel);
}

#[test]
fn live_kernel_p0_guest_gateway_cancellation() {
    let Some((bin, p0)) = kit() else {
        eprintln!("SKIP: SKRYNIA_V3_SERVER_BIN and SKRYNIA_V3_PROGRAM0_WASM required");
        return;
    };
    let token = [0x42u8; 32];
    let edge_cap = "edge-cap-secret-cancel";
    let kernel = Kernel::start(&bin, &p0, token);
    let gw = Gateway::start(&kernel.url(), token, five_apps(edge_cap), 16 * 1024 * 1024);

    // Start a large request and disconnect mid-body (cancellation)
    let mut stream = TcpStream::connect(gw.addr).unwrap();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
    let body_len = 8 * 1024 * 1024;
    let req = format!(
        "POST /_skrynia/ HTTP/1.1\r\nHost: long.a.vau.place\r\nOrigin: https://long.a.vau.place\r\nX-Skrynia-Edge-Capability: {edge_cap}\r\nContent-Type: application/octet-stream\r\nContent-Length: {body_len}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).unwrap();
    // write partial body then drop
    let chunk = vec![7u8; 64 * 1024];
    let _ = stream.write_all(&chunk);
    drop(stream); // client disconnect/cancellation

    std::thread::sleep(Duration::from_millis(200));
    // Gateway and kernel must remain healthy and serve subsequent requests
    let (st, _, body) = post_raw(
        gw.addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(edge_cap),
        b"after-cancel",
    );
    assert_eq!(st, 200, "gateway healthy after cancellation");
    assert_eq!(body, b"after-cancel");
    let _ = (gw, kernel);
}
