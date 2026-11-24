use crate::wire::AppPrecond;

pub fn match_expect(cur: Option<[u8; 32]>, expect: &AppPrecond) -> bool {
    match (cur, expect) {
        (None, AppPrecond::Absent) => true,
        (Some(h), AppPrecond::Present(e)) => e.as_slice() == h.as_slice(),
        _ => false,
    }
}
