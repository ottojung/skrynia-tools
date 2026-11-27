use super::{ExampleRng, MAX_BODY, MAX_PLAINTEXT, encrypt, subscription};

#[test]
fn wake_and_maximum_fit_provider_body_limit() {
    for (n, size) in [(32, 135), (MAX_PLAINTEXT, MAX_BODY)] {
        let body = encrypt(&vec![0x5a; n], &subscription(), &ExampleRng).unwrap();
        assert_eq!(body.len(), size);
        assert!(u32::from_be_bytes(body[16..20].try_into().unwrap()) as usize > body.len() - 86);
    }
    assert!(encrypt(&vec![0; MAX_PLAINTEXT + 1], &subscription(), &ExampleRng).is_err());
}
