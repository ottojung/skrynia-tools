use anyhow::{Result, ensure};
use skrynia_tools::{
    client::SkryniaClient,
    event_set::{EventSet, Insert},
};

fn main() -> Result<()> {
    let url = std::env::var("SKRYNIA_URL")?;
    let token = std::env::var("SKRYNIA_EVENT_TOKEN")?;
    let prefix = std::env::var("SKRYNIA_EVENT_PREFIX")?;
    let client = SkryniaClient::new(&url)?.with_token(token);
    let events = EventSet::open(&client, &prefix)?;
    let first = "00000000000000000000000000000001";
    let second = "00000000000000000000000000000002";
    ensure!(events.insert(first, b"first")? == Insert::Created);
    ensure!(events.insert(first, b"first")? == Insert::AlreadyPresent);
    ensure!(events.insert(first, b"different").is_err());
    ensure!(events.insert(second, b"second")? == Insert::Created);
    let scan = events.scan(4)?;
    ensure!(scan.complete && scan.events.len() == 2);
    ensure!(scan.events[0].bytes == b"first");
    ensure!(scan.events[1].bytes == b"second");
    println!("EventSet live create/list/idempotency integration passed");
    Ok(())
}
