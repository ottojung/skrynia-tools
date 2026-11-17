use super::{Keys, Release, client, release};

use anyhow::{Context, Result};
pub fn versions(client: &client::SkryniaClient, name: &str) -> Result<Vec<Release>> {
    let keys = Keys::new(name)?;
    let mut after = None;
    let mut result = Vec::new();
    loop {
        let page = client.list_page(&format!("{}/versions/*", keys.prefix), after.as_deref())?;
        let names = page["keys"].as_array().context("invalid release listing")?;
        for value in names {
            let key = value.as_str().context("invalid release key")?;
            let Some(id) = key
                .strip_prefix(&format!("{}/versions/", keys.prefix))
                .and_then(|v| v.strip_suffix(".ready"))
            else {
                continue;
            };
            result.push(release(client, name, id)?);
        }
        after = page["next"].as_str().map(str::to_owned);
        if after.is_none() {
            break;
        }
    }
    Ok(result)
}
