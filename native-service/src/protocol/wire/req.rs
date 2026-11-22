use super::AppPrecond;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Register {
        operator_cap: Vec<u8>,
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
    Admin(super::AdminRequest),
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
    PushSubscribe(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>),
    PushUnsubscribe(Vec<u8>, Vec<u8>, Vec<u8>),
    PushGrant(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>),
    PushSenderGrant(Vec<u8>, Vec<u8>, Vec<u8>),
    HintList(Vec<u8>, Vec<u8>, Vec<u8>),
    HintRead(Vec<u8>, Vec<u8>, Vec<u8>, u64),
    HintAck(Vec<u8>, Vec<u8>, Vec<u8>, u64),
    SubFetch(Vec<u8>, Vec<u8>, Vec<u8>),
}
