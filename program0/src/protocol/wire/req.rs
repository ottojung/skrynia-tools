#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppPrecond {
    Absent,
    Present(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Register {
        admin_cap: Vec<u8>,
        code: Vec<u8>,
    },
    Set {
        admin_cap: Vec<u8>,
        expect: AppPrecond,
        code: Vec<u8>,
    },
    Invoke {
        invoke_id: Vec<u8>,
        input: Vec<u8>,
    },
    Lookup {
        invoke_id: Vec<u8>,
    },
    Revoke {
        admin_cap: Vec<u8>,
        expect: AppPrecond,
    },
    StageCode {
        admin_cap: Vec<u8>,
        code: Vec<u8>,
    },
    SealCode {
        admin_cap: Vec<u8>,
        hash: Vec<u8>,
    },
    ActivateCode {
        admin_cap: Vec<u8>,
        expected_revision: u64,
        target: Vec<u8>,
    },
    GetActive {
        invoke_id: Vec<u8>,
    },
    ListVersions {
        invoke_id: Vec<u8>,
        after: Option<Vec<u8>>,
        limit: u16,
    },
    ListPrograms {
        after: Option<Vec<u8>>,
        limit: u16,
    },
    AppGet {
        read_cap: Vec<u8>,
        key: Vec<u8>,
    },
    AppPut {
        write_cap: Vec<u8>,
        key: Vec<u8>,
        expect: AppPrecond,
        value: Vec<u8>,
    },
    AppDelete {
        write_cap: Vec<u8>,
        key: Vec<u8>,
        expect: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u8,
    pub body: Vec<u8>,
}
