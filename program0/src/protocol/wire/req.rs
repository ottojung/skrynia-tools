use super::V2Kind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Register {
        code: Vec<u8>,
    },
    Set {
        id: Vec<u8>,
        code: Vec<u8>,
    },
    Invoke {
        id: Vec<u8>,
        input: Vec<u8>,
    },
    Lookup {
        id: Vec<u8>,
    },
    Revoke {
        id: Vec<u8>,
    },
    V2Bootstrap {
        token: [u8; 32],
    },
    V2Mutate {
        token: [u8; 32],
        key: Vec<u8>,
        kind: V2Kind,
        expected: Option<[u8; 32]>,
        body: Vec<u8>,
    },
}
