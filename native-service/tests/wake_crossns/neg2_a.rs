use super::common::Mem;
use super::setup::{hints, iw, wake_setup};
use skrynia_native_service::registry::appdata::{AppErr, WriteAuth, ops};
use skrynia_native_service::wire::AppPrecond;
use skrynia_native_service::{MutLock, NoLock, hash_bytes, read_cap};
struct Revoke<'a>(&'a Mem, String, Vec<u8>);
impl MutLock for Revoke<'_> {
    type Err = ();
    fn try_acquire(&self) -> Result<bool, ()> {
        self.0
            .0
            .lock()
            .unwrap()
            .insert(self.1.clone(), self.2.clone());
        Ok(true)
    }
    fn release(&self) -> Result<(), ()> {
        Ok(())
    }
}
fn hex(b: &[u8; 32]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
type R = Result<[u8; 32], AppErr<(), ()>>;
fn pwl<L: MutLock<Err = ()>>(d: &Mem, l: &L, w: &[u8; 32], g: &[u8; 32], wk: &[u8]) -> R {
    ops::put_wake(
        d,
        l,
        "d",
        WriteAuth::External(w),
        b"k",
        &AppPrecond::Absent,
        b"v",
        g,
        wk,
    )
}
#[test]
fn concurrent_revoke_blocks_wake_mutation() {
    let (d, wa, _, ga, _) = wake_setup();
    let wk = b"shared-wake-v1";
    iw(&d, &[1u8; 32], &[8u8; 32], wk, &[3u8; 32]);
    let rp = format!(
        "d/pushwake/{}/{}.bin",
        hex(&hash_bytes(&[8u8; 32])),
        hex(&hash_bytes(wk))
    );
    let revoked = d.0.lock().unwrap().get(&rp).cloned().unwrap();
    iw(&d, &[1u8; 32], &ga, wk, &wa);
    let path = format!(
        "d/pushwake/{}/{}.bin",
        hex(&hash_bytes(&ga)),
        hex(&hash_bytes(wk))
    );
    let e = pwl(&d, &Revoke(&d, path, revoked), &wa, &ga, wk);
    assert!(matches!(e, Err(AppErr::BadAuth)));
    assert_eq!(hints(&d, &read_cap(&[1u8; 32]), wk), 0);
}
