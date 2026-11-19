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
