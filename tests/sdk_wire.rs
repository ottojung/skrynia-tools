use skrynia_tools::sdk::{
    TAG_I32, TAG_I64, TypedValue, VERSION, ValType, WireError, decode, encode,
};

#[test]
fn typed_wire_roundtrip_for_mixed_widths_and_empty() {
    let values = [TypedValue::I32(-7), TypedValue::I64(1 << 40)];
    let bytes = encode(&values).unwrap();
    assert_eq!(bytes[0], VERSION);
    assert_eq!(bytes[1], 2);
    assert_eq!(bytes[2], TAG_I32);
    assert_eq!(bytes[7], TAG_I64);
    assert_eq!(
        decode(&bytes, &[ValType::I32, ValType::I64]).unwrap(),
        values.to_vec()
    );
    let empty = encode(&[]).unwrap();
    assert_eq!(empty, [VERSION, 0]);
    assert!(decode(&empty, &[]).unwrap().is_empty());
}

#[test]
fn wire_rejects_malformed_or_mismatched_frames() {
    let encoded = encode(&[TypedValue::I32(1)]).unwrap();
    assert_eq!(decode(&encoded, &[]), Err(WireError::Count));
    assert_eq!(decode(&encoded, &[ValType::I64]), Err(WireError::Tag));
    assert_eq!(decode(&[0x02, 0], &[]), Err(WireError::Version));
    assert_eq!(decode(&[VERSION], &[]), Err(WireError::Truncated));
    let mut trailing = encoded;
    trailing.push(0);
    assert_eq!(decode(&trailing, &[ValType::I32]), Err(WireError::Trailing));
}
