// RFC 8291 Appendix A public test vector, not production key material.
use super::{MAX_BODY, MAX_PLAINTEXT, Rng, Subscription, encrypt};
use base64ct::{Base64UrlUnpadded, Encoding};

struct ExampleRng;
impl Rng for ExampleRng {
    fn fill(&self, out: &mut [u8]) {
        let encoded = match out.len() {
            16 => "DGv6ra1nlYgDCS1FRnbzlw",
            32 => "yfWPiYE-n46HLnH0KqZOF1fJJU3MYrct3AELtAQ-oRw",
            _ => panic!("unexpected RNG request"),
        };
        out.copy_from_slice(&decode(encoded));
    }
}

fn decode(s: &str) -> Vec<u8> {
    Base64UrlUnpadded::decode_vec(s).unwrap()
}

fn subscription() -> Subscription {
    Subscription {
        endpoint: "https://push.example.test/disposable".into(),
        p256dh: decode(concat!(
            "BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcx",
            "aOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4"
        )),
        auth: decode("BTBZMqHH6r4Tts7J_aSIgg"),
    }
}

#[test]
fn rfc8291_appendix_a_exact_ciphertext() {
    let payload = decode("V2hlbiBJIGdyb3cgdXAsIEkgd2FudCB0byBiZSBhIHdhdGVybWVsb24");
    let mut expected = decode(concat!(
        "DGv6ra1nlYgDCS1FRnbzlwAAEABBBP4z",
        "9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6Tlz",
        "AC8wEqKK6PBru3jl7A8"
    ));
    expected.extend(decode(concat!(
        "8pfeW0KbunFT06SuDKoJH9Ql87S1QUrd",
        "irN6GcG7sFz1y1sqLgVi1VhjVkHsUoEsbI_0LpXMuGvnzQ"
    )));
    assert_eq!(
        encrypt(&payload, &subscription(), &ExampleRng).unwrap(),
        expected
    );
}

mod bounds;
