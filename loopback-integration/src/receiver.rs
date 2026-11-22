//! Controlled local test receiver: disposable-CA loopback HTTPS push endpoint.
//! Adapted verbatim from the sender crate's proven `tests/common/tls` helpers
//! into one disposable harness file. Binds 127.0.0.1 only; no real DNS/TLS.
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::{ClientHello, ResolvesServerCert, ServerConfig};
use rustls::sign::CertifiedKey;
use std::fs;
use std::io;
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener as TokioListener;
use tokio::runtime::Builder;
use tokio_rustls::TlsAcceptor;

pub const HOST: &str = "push.example.test";

#[derive(Debug, Clone, Default)]
pub struct Seen {
    pub method: String,
    pub target: String,
    pub host: String,
    pub sni: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Seen {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

#[derive(Clone)]
pub struct ReplyPlan {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub delay_ms: u64,
}

impl ReplyPlan {
    pub fn status(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: b"ok".to_vec(),
            delay_ms: 0,
        }
    }
    pub fn retry(mut self, seconds: u64) -> Self {
        self.headers
            .push(("Retry-After".into(), seconds.to_string()));
        self
    }
}

static SEQ: AtomicU64 = AtomicU64::new(0);

struct Materials {
    ca_der: Vec<u8>,
    cert_der: Vec<u8>,
    key_pk8: Vec<u8>,
    dir: PathBuf,
}

fn openssl(args: &[&str]) {
    let status = Command::new("openssl")
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("openssl");
    assert!(status.success(), "openssl failed: {}", args.join(" "));
}

fn path(dir: &Path, name: &str) -> String {
    dir.join(name).to_str().unwrap().to_owned()
}

impl Materials {
    fn new(host: &str) -> Self {
        let id = SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("wps-loopback-{}-{}", std::process::id(), id));
        fs::create_dir_all(&dir).unwrap();
        let (ca_key, ca_pem, ca_der) = (
            path(&dir, "ca.key"),
            path(&dir, "ca.pem"),
            path(&dir, "ca.der"),
        );
        openssl(&[
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-nodes",
            "-days",
            "1",
            "-keyout",
            &ca_key,
            "-out",
            &ca_pem,
            "-subj",
            "/CN=Loopback Test CA",
            "-addext",
            "basicConstraints=critical,CA:TRUE",
            "-addext",
            "keyUsage=critical,keyCertSign,cRLSign",
        ]);
        openssl(&["x509", "-in", &ca_pem, "-outform", "DER", "-out", &ca_der]);
        let (s_key, s_csr, s_pem, s_der, s_pk8, s_ext) = (
            path(&dir, "s.key"),
            path(&dir, "s.csr"),
            path(&dir, "s.pem"),
            path(&dir, "s.der"),
            path(&dir, "s.pk8"),
            path(&dir, "s.ext"),
        );
        let san = format!(
            "subjectAltName=DNS:{host}\nbasicConstraints=CA:FALSE\n\
             keyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n"
        );
        fs::write(&s_ext, san).unwrap();
        let subject = format!("/CN={host}");
        openssl(&[
            "req", "-new", "-newkey", "rsa:2048", "-nodes", "-keyout", &s_key, "-out", &s_csr,
            "-subj", &subject,
        ]);
        openssl(&[
            "x509",
            "-req",
            "-in",
            &s_csr,
            "-CA",
            &ca_pem,
            "-CAkey",
            &ca_key,
            "-CAcreateserial",
            "-days",
            "1",
            "-extfile",
            &s_ext,
            "-out",
            &s_pem,
        ]);
        openssl(&["x509", "-in", &s_pem, "-outform", "DER", "-out", &s_der]);
        openssl(&[
            "pkcs8", "-topk8", "-nocrypt", "-in", &s_key, "-outform", "DER", "-out", &s_pk8,
        ]);
        Self {
            ca_der: fs::read(&ca_der).unwrap(),
            cert_der: fs::read(&s_der).unwrap(),
            key_pk8: fs::read(&s_pk8).unwrap(),
            dir,
        }
    }
}

impl Drop for Materials {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[derive(Debug)]
struct Sni {
    key: CertifiedKey,
    seen: Arc<Mutex<Option<String>>>,
}

impl ResolvesServerCert for Sni {
    fn resolve(&self, hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        *self.seen.lock().unwrap() = hello.server_name().map(str::to_owned);
        Some(Arc::new(self.key.clone()))
    }
}

fn server_config(m: &Materials, sni: Arc<Mutex<Option<String>>>) -> ServerConfig {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let chain = vec![CertificateDer::from(m.cert_der.clone())];
    let key = PrivateKeyDer::Pkcs8(m.key_pk8.clone().into());
    let key = CertifiedKey::from_der(chain, key, &provider).expect("valid test key");
    ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("tls versions")
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(Sni { key, seen: sni }))
}

async fn read_request<S: AsyncRead + Unpin>(stream: &mut S) -> io::Result<(String, Vec<u8>)> {
    let mut data = Vec::new();
    let mut byte = [0u8; 1];
    while data.len() < 8192 {
        if stream.read(&mut byte).await? == 0 {
            return Err(io::Error::other("empty request"));
        }
        data.push(byte[0]);
        if data.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let end = data
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| io::Error::other("no header terminator"))?;
    let head = String::from_utf8_lossy(&data[..end]).into_owned();
    let want = head
        .lines()
        .find_map(|line| {
            let (n, v) = line.split_once(':')?;
            n.trim()
                .eq_ignore_ascii_case("content-length")
                .then(|| v.trim().parse::<usize>().unwrap_or(0))
        })
        .unwrap_or(0);
    let mut body = data[end + 4..].to_vec();
    while body.len() < want && body.len() < 16384 {
        let mut buf = [0u8; 1024];
        let n = (want - body.len()).min(buf.len()).min(16384 - body.len());
        let got = stream.read(&mut buf[..n]).await?;
        if got == 0 {
            break;
        }
        body.extend_from_slice(&buf[..got]);
    }
    Ok((head, body))
}

fn parse(head: &str, body: Vec<u8>, sni: Option<String>) -> Seen {
    let mut lines = head.lines();
    let mut parts = lines.next().unwrap_or_default().split_whitespace();
    let method = parts.next().unwrap_or_default().to_owned();
    let target = parts.next().unwrap_or_default().to_owned();
    let mut headers = Vec::new();
    for line in lines {
        if let Some((n, v)) = line.split_once(':') {
            headers.push((n.trim().to_ascii_lowercase(), v.trim().to_owned()));
        }
    }
    let host = headers
        .iter()
        .find(|(n, _)| n == "host")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    Seen {
        method,
        target,
        host,
        sni,
        headers,
        body,
    }
}

async fn write_reply<S: AsyncWrite + Unpin>(stream: &mut S, plan: &ReplyPlan) -> io::Result<()> {
    if plan.delay_ms > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(plan.delay_ms)).await;
    }
    let mut out = format!(
        "HTTP/1.1 {} X\r\nContent-Length: {}\r\nConnection: close\r\n",
        plan.status,
        plan.body.len()
    )
    .into_bytes();
    for (n, v) in &plan.headers {
        out.extend_from_slice(format!("{n}: {v}\r\n").as_bytes());
    }
    out.extend_from_slice(b"\r\n");
    out.extend_from_slice(&plan.body);
    stream.write_all(&out).await?;
    stream.flush().await
}

async fn serve(
    listener: std::net::TcpListener,
    materials: Arc<Materials>,
    plan: Arc<ReplyPlan>,
    log: Arc<Mutex<Vec<Seen>>>,
    stop: Arc<AtomicBool>,
) {
    let listener = TokioListener::from_std(listener).unwrap();
    while !stop.load(Ordering::Relaxed) {
        let Ok(Ok((stream, _))) =
            tokio::time::timeout(std::time::Duration::from_millis(200), listener.accept()).await
        else {
            continue;
        };
        let (m, p, l) = (materials.clone(), plan.clone(), log.clone());
        tokio::spawn(async move {
            let sni = Arc::new(Mutex::new(None));
            let cfg = server_config(&m, sni.clone());
            let Ok(mut tls) = TlsAcceptor::from(Arc::new(cfg)).accept(stream).await else {
                return;
            };
            let Ok((head, body)) = read_request(&mut tls).await else {
                return;
            };
            let _ = write_reply(&mut tls, &p).await;
            let mut seen = parse(&head, body, sni.lock().unwrap().clone());
            seen.sni = sni.lock().unwrap().clone();
            l.lock().unwrap().push(seen);
        });
    }
}

pub struct FakePushService {
    pub addr: SocketAddr,
    pub ca_der: Vec<u8>,
    pub seen: Arc<Mutex<Vec<Seen>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    _materials: Arc<Materials>,
}

impl FakePushService {
    pub fn start(plan: ReplyPlan) -> Self {
        let materials = Arc::new(Materials::new(HOST));
        let ca_der = materials.ca_der.clone();
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (m, p, s, t) = (
            materials.clone(),
            Arc::new(plan),
            seen.clone(),
            stop.clone(),
        );
        let worker = std::thread::spawn(move || {
            let rt = Builder::new_current_thread().enable_all().build().unwrap();
            rt.block_on(serve(listener, m, p, s, t));
        });
        Self {
            addr,
            ca_der,
            seen,
            stop,
            worker: Some(worker),
            _materials: materials,
        }
    }
    pub fn seen(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }
}

impl Drop for FakePushService {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}
