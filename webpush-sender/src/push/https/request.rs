//! WebPush request construction. Every required header is set explicitly;
//! values come only from the already-validated PushRequest.
use super::{IoFailure, PushRequest};
use base64ct::{Base64UrlUnpadded, Encoding};
use reqwest::blocking::Request;
use reqwest::header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE, HeaderName, HeaderValue};

const TOPIC: &str = "Topic";
const TTL: HeaderName = HeaderName::from_static("ttl");

pub fn build(req: &PushRequest) -> Result<Request, IoFailure> {
    let url = format!("https://{}{}", req.host, req.path);
    let mut out = Request::new(reqwest::Method::POST, url.parse().map_err(|_| IoFailure)?);
    let headers = out.headers_mut();
    headers.insert(AUTHORIZATION, value(&req.authorization)?);
    headers.insert(CONTENT_ENCODING, value("aes128gcm")?);
    headers.insert(CONTENT_TYPE, value("application/octet-stream")?);
    headers.insert(TTL, value(&req.ttl.to_string())?);
    let topic = Base64UrlUnpadded::encode_string(&req.topic);
    headers.insert(TOPIC, value(&topic)?);
    *out.body_mut() = Some(req.body.clone().into());
    Ok(out)
}

fn value(text: &str) -> Result<HeaderValue, IoFailure> {
    HeaderValue::from_str(text).map_err(|_| IoFailure)
}
