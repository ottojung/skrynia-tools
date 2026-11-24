use anyhow::{Result, ensure};
use skrynia_tools::{
    cas::snapshot::{Commit, SnapshotStore},
    client::SkryniaClient,
};
use std::collections::BTreeMap;

fn main() -> Result<()> {
    let url = std::env::var("SKRYNIA_URL")?;
    let edge = std::env::var("SKRYNIA_EDGE_CAPABILITY")?;
    let read = std::env::var("SKRYNIA_READ_CAPABILITY")?;
    let write = std::env::var("SKRYNIA_WRITE_CAPABILITY")?;
    let prefix = std::env::var("SKRYNIA_SNAPSHOT_PREFIX")?;
    let client = SkryniaClient::new(&url, &edge, &read, Some(&write))?;
    let store = SnapshotStore::open(&client, &prefix)?;
    store.initialize("00000000000000000000000000000001")?;
    let genesis = store.read()?;
    ensure!(genesis.revision() == 0);
    let mut desired = BTreeMap::new();
    desired.insert("alpha".into(), b"A".to_vec());
    desired.insert("beta".into(), b"B".to_vec());
    ensure!(
        store.commit(&genesis, "00000000000000000000000000000002", &desired)? == Commit::Committed
    );
    let first = store.read()?;
    ensure!(first.revision() == 1 && first.values() == &desired);
    ensure!(
        store.commit(&genesis, "00000000000000000000000000000003", &desired)? == Commit::Conflict
    );
    desired.insert("beta".into(), b"new B".to_vec());
    ensure!(
        store.commit(&first, "00000000000000000000000000000004", &desired)? == Commit::Committed
    );
    let second = store.read()?;
    ensure!(second.revision() == 2 && second.values() == &desired);
    println!("SnapshotStore live CAS and multi-object integration passed");
    Ok(())
}
