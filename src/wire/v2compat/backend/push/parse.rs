use super::{CompatError, Events, Keys, SubReq, UpdateReq};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SubBody {
    locations: String,
    #[serde(default)]
    events: Option<Vec<String>>,
    endpoint: String,
    keys: KeysBody,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeysBody {
    p256dh: String,
    auth: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpBody {
    endpoint: String,
    keys: KeysBody,
}

pub fn sub(body: &[u8]) -> Result<SubReq, CompatError> {
    let b: SubBody = serde_json::from_slice(body).map_err(|_| bad())?;
    Ok(SubReq {
        locations: b.locations,
        events: b.events.map(Events).unwrap_or_else(Events::default_set),
        endpoint: b.endpoint,
        keys: Keys {
            p256dh: b.keys.p256dh,
            auth: b.keys.auth,
        },
    })
}
pub fn update(body: &[u8]) -> Result<UpdateReq, CompatError> {
    let b: UpBody = serde_json::from_slice(body).map_err(|_| bad())?;
    Ok(UpdateReq {
        endpoint: b.endpoint,
        keys: Keys {
            p256dh: b.keys.p256dh,
            auth: b.keys.auth,
        },
    })
}
fn bad() -> CompatError {
    CompatError::new(400, "invalid_push_subscription")
}
