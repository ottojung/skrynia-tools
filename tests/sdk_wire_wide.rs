use skrynia_tools::sdk::{
    COUNT_EXTENDED, TAG_I32, TypedValue, VERSION, ValType, WireError, decode, encode,
};

#[test]
fn extended_count_roundtrips_over_254_mixed_values() {
    let values: Vec<TypedValue> = (0..300)
        .map(|i| {
            if i % 2 == 0 {
                TypedValue::I32(i)
            } else {
                TypedValue::I64(i as i64)
            }
        })
        .collect();
    let bytes = encode(&values).unwrap();
    assert_eq!((bytes[0], bytes[1]), (VERSION, COUNT_EXTENDED));
    assert_eq!(u32::from_le_bytes(bytes[2..6].try_into().unwrap()), 300);
    let types: Vec<ValType> = values.iter().map(|v| v.kind()).collect();
    assert_eq!(decode(&bytes, &types).unwrap(), values);
}

#[test]
fn small_frames_and_publication_ticket_are_byte_identical() {
    let values: Vec<TypedValue> = (0..254).map(TypedValue::I32).collect();
    let bytes = encode(&values).unwrap();
    assert_eq!(bytes[1], 254);
    let types: Vec<ValType> = values.iter().map(|v| v.kind()).collect();
    assert_eq!(decode(&bytes, &types).unwrap(), values);
    assert_eq!(
        encode(&[TypedValue::I32(0x0102_0304)]).unwrap(),
        [1, 1, TAG_I32, 4, 3, 2, 1]
    );
}

#[test]
fn malformed_extended_counts_are_rejected() {
    let one = [ValType::I32];
    let mut frame = vec![VERSION, COUNT_EXTENDED, 1, 0, 0, 0, TAG_I32, 1, 0, 0, 0];
    assert_eq!(decode(&frame, &one), Err(WireError::NonMinimal));
    frame[5] = 2;
    assert_eq!(decode(&frame, &one), Err(WireError::Count));
    frame.truncate(4);
    assert_eq!(decode(&frame, &one), Err(WireError::Truncated));
}
