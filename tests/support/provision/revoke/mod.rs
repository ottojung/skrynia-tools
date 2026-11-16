use super::{fixtures, server_revoke};
mod complete;
mod failure;
mod happy;
mod uncertain;
mod uncertain_grant;
pub fn verify() {
    happy::verify();
}
pub fn failures() {
    failure::verify();
}
pub fn retry() {
    uncertain::verify();
}
