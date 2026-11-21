#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminRequest {
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
}
