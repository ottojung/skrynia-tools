use super::{Keys, Pointer, client};
use anyhow::{Result, bail};
use client::Store;
pub fn activate(client: &client::SkryniaClient, name: &str, id: &str) -> Result<Pointer> {
    let keys = Keys::new(name)?;
    Keys::validate(id)?;
    if client.get(&keys.stamp(id)?)?.as_deref() != Some(b"ready".as_slice()) {
        bail!("release is not finalized");
    }
    let key = keys.pointer();
    let before = client.get(&key)?;
    let pointer = Pointer::new(id);
    let value = serde_json::to_vec(&pointer)?;
    match before {
        Some(old) => {
            let hash = client::SkryniaClient::digest(&old);
            if !client.compare_replace(&key, &value, &hash)? {
                bail!("deployment activation conflict; active version changed");
            }
        }
        None => {
            if client.create(&key, &value)? != client::CreateResult::Created {
                bail!("deployment activation conflict; active pointer appeared");
            }
        }
    }
    Ok(pointer)
}
