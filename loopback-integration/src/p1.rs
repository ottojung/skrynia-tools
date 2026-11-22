//! Ordinary-P1 loopback fake. Executes the exact native-service push opcodes
//! 16/17/20-23 that the pinned P1 WASM does NOT contain (see BLOCKER.md), so
//! the harness can verify the caller/transport contract while the untested
//! real-WASM boundary is stated explicitly.
use std::sync::Mutex;
use webpush_sender::error::IoFailure;
use webpush_sender::p1::P1Transport;

pub const OP_SUBSCRIBE: u8 = 16;
pub const OP_UNSUBSCRIBE: u8 = 17;
pub const OP_HINT_LIST: u8 = 20;
pub const OP_HINT_READ: u8 = 21;
pub const OP_HINT_ACK: u8 = 22;
pub const OP_SUB_FETCH: u8 = 23;

fn res(body: &[u8]) -> Vec<u8> {
    let mut out = vec![1u8, 0];
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(body);
    out
}

fn err(code: u8) -> Vec<u8> {
    vec![1u8, 1, 1, 0, 0, 0, code]
}

fn id(out: &mut Vec<u8>, v: &[u8]) {
    out.extend_from_slice(&(v.len() as u16).to_le_bytes());
    out.extend_from_slice(v);
}

fn blob(out: &mut Vec<u8>, v: &[u8]) {
    out.extend_from_slice(&(v.len() as u32).to_le_bytes());
    out.extend_from_slice(v);
}

pub fn frame(cap: &[u8], ns: &[u8], key: &[u8], op: u8, tail: &[u8]) -> Vec<u8> {
    let mut out = vec![op];
    id(&mut out, cap);
    id(&mut out, ns);
    blob(&mut out, key);
    out.extend_from_slice(tail);
    out
}

pub struct LoopbackP1 {
    pub calls: Mutex<Vec<Vec<u8>>>,
    pub hints: Mutex<Vec<u64>>,
    pub subs: Mutex<Vec<Vec<u8>>>,
    pub hint: [u8; 32],
    pub acked: Mutex<Vec<u64>>,
    pub required_cap: [u8; 32],
}

impl LoopbackP1 {
    pub fn new(cap: [u8; 32], hints: Vec<u64>, subs: Vec<Vec<u8>>) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            hints: Mutex::new(hints),
            subs: Mutex::new(subs),
            hint: [0x5a; 32],
            acked: Mutex::new(Vec::new()),
            required_cap: cap,
        }
    }

    pub fn subscribe(&self, cap: &[u8], ns: &[u8], key: &[u8], who: &[u8], sub: &[u8]) {
        let mut tail = Vec::new();
        id(&mut tail, who);
        blob(&mut tail, sub);
        let req = frame(cap, ns, key, OP_SUBSCRIBE, &tail);
        assert_eq!(self.call(&req).unwrap(), res(&[]));
        self.subs.lock().unwrap().push(sub.to_vec());
    }

    pub fn revoke(&self, cap: &[u8], ns: &[u8], key: &[u8], who: &[u8]) {
        let mut tail = Vec::new();
        id(&mut tail, who);
        let req = frame(cap, ns, key, OP_UNSUBSCRIBE, &tail);
        assert_eq!(self.call(&req).unwrap(), res(&[]));
        self.subs.lock().unwrap().clear();
    }

    pub fn opcodes(&self) -> Vec<u8> {
        self.calls.lock().unwrap().iter().map(|c| c[0]).collect()
    }

    fn cap_of(req: &[u8]) -> Option<&[u8]> {
        if req.len() < 3 {
            return None;
        }
        let n = u16::from_le_bytes([req[1], req[2]]) as usize;
        req.get(3..3 + n)
    }
}

impl P1Transport for LoopbackP1 {
    fn call(&self, req: &[u8]) -> core::result::Result<Vec<u8>, IoFailure> {
        self.calls.lock().unwrap().push(req.to_vec());
        if Self::cap_of(req) != Some(self.required_cap.as_slice()) {
            return Ok(err(5)); // unauthorized: wrong scoped bearer
        }
        let body = match req.first().copied() {
            Some(OP_HINT_LIST) => {
                let seqs = self.hints.lock().unwrap();
                let mut out = (seqs.len() as u32).to_le_bytes().to_vec();
                for s in seqs.iter() {
                    out.extend_from_slice(&s.to_le_bytes());
                }
                out
            }
            Some(OP_HINT_READ) => self.hint.to_vec(),
            Some(OP_HINT_ACK) => {
                self.acked.lock().unwrap().push(u64::from_le_bytes(
                    req[req.len() - 8..].try_into().map_err(|_| IoFailure)?,
                ));
                Vec::new()
            }
            Some(OP_SUB_FETCH) => {
                let subs = self.subs.lock().unwrap();
                let mut out = (subs.len() as u32).to_le_bytes().to_vec();
                for s in subs.iter() {
                    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
                    out.extend_from_slice(s);
                }
                out
            }
            Some(OP_SUBSCRIBE) | Some(OP_UNSUBSCRIBE) => Vec::new(),
            _ => return Err(IoFailure),
        };
        Ok(res(&body))
    }
}
