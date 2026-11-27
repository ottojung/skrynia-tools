use crate::cases::support::{MemDisk, NoLock};
use skrynia_native_service::reg::Registry;
use skrynia_native_service::wire::{AppPrecond, Request, STATUS_ERR, STATUS_OK};
use skrynia_native_service::{hash_bytes, p2::Interpreter, wdec::decode_res, wenc::encode_req};

const GUEST: &str = include_str!("guest.wat");

#[test]
fn registered_real_wasm_can_mutate_only_its_own_appdata() {
    let disk = MemDisk::default();
    let reg = Registry::open(&disk, "programs").unwrap();
    let iid = reg
        .register(&disk, &NoLock, &[7; 32], wat::parse_str(GUEST).unwrap())
        .unwrap();
    let key = b"arbitrary-key".to_vec();
    let value = b"not a hardcoded callback value".to_vec();
    let call = |req| {
        decode_res(
            &reg.invoke(&disk, &NoLock, &Interpreter::new(), &iid, &encode_req(&req))
                .unwrap(),
        )
        .unwrap()
    };
    let put = Request::AppPut {
        write_cap: iid.clone(),
        key: key.clone(),
        expect: AppPrecond::Absent,
        value: value.clone(),
    };
    assert_eq!(call(put).status, STATUS_OK);
    assert_eq!(reg.app_get(&disk, &iid, &key).unwrap(), value);
    let get = Request::AppGet {
        read_cap: iid.clone(),
        key: key.clone(),
    };
    assert_eq!(call(get).body, value);
    let foreign = Request::AppGet {
        read_cap: vec![8; 32],
        key: key.clone(),
    };
    assert_eq!(call(foreign).status, STATUS_ERR);
    let del = Request::AppDelete {
        write_cap: iid.clone(),
        key: key.clone(),
        expect: hash_bytes(&value).to_vec(),
    };
    assert_eq!(call(del).status, STATUS_OK);
    assert!(reg.app_get(&disk, &iid, &key).is_err());
}
