use skrynia_tools::sdk::{
    FFI_MODULE, MODULE, ScalarError, ValType, bulk, cas1, fixed, names, read1, seek, status, write1,
};

#[test]
fn fixed_inventory_is_linear_memory_and_u64_addressed() {
    assert_eq!(MODULE, "skrynia_v3_1");
    assert_eq!(FFI_MODULE, "syscall");
    assert_eq!(names().count(), 21);
    let read = fixed("read").unwrap();
    assert_eq!(read.params, [ValType::I32, ValType::I64, ValType::I64]);
    assert_eq!(read.results, [ValType::I64]);
    let write = fixed("write").unwrap();
    assert_eq!(write.params, [ValType::I32, ValType::I64, ValType::I64]);
    assert_eq!(fixed("read1").unwrap().params, [ValType::I32]);
    assert_eq!(
        fixed("write1").unwrap().params,
        [ValType::I32, ValType::I32]
    );
    assert_eq!(fixed("seek").unwrap().params, [ValType::I32, ValType::I64]);
    assert_eq!(fixed("cas1").unwrap().params.len(), 4);
    assert_eq!(
        fixed("eval").unwrap().params,
        [
            ValType::I64,
            ValType::I64,
            ValType::I32,
            ValType::I32,
            ValType::I32
        ]
    );
    assert!(fixed("copy").is_none());
}

#[test]
fn scalar_and_bulk_encodings_follow_target_abi() {
    assert_eq!(read1(255), Ok(255));
    assert_eq!(read1(-1), Err(ScalarError(-1)));
    assert!(read1(-1).unwrap_err().is_eof());
    assert_eq!(write1(0), Ok(()));
    assert_eq!(seek(12), Ok(12));
    assert_eq!(seek(-6), Err(ScalarError(-6)));
    assert_eq!(cas1(0), Ok(0));
    assert_eq!(status(0), Ok(()));
    assert_eq!(bulk(0), Ok(0));
    assert_eq!(bulk(7), Ok(7));
    assert_eq!(bulk(-5), Err(ScalarError(-5)));
}
