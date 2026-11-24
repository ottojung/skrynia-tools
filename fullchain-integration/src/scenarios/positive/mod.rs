//! Real full-chain positive: kernel->P0->P1->trusted edge->sender->HTTPS provider.
mod asserts;
mod ctx;
mod more;
mod pump;
mod setup;
pub fn run_positive(art: &std::path::Path, tmp: &std::path::Path) -> Vec<String> {
    let mut c = ctx::Ctx::new(art, tmp);
    setup::bootstrap(&mut c);
    let (rep, sent) = pump::deliver(&mut c);
    let mut ev = asserts::verify(&c, &rep, sent);
    ev.extend(more::restart(&mut c));
    ev.extend(more::revoke(&mut c));
    ev
}
