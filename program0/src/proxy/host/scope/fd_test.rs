use super::fd::Table;
#[test]
fn recycle_65k_without_stale_alias() {
    let mut t = Table::new();
    let h1 = t.insert(10).unwrap();
    for _ in 0..65_535u32 {
        let h = t.insert(20).unwrap();
        assert_eq!(t.take(h), Some(20));
    }
    assert_eq!(t.get(h1), Some(10));
    assert_eq!(t.take(h1), Some(10));
    assert_eq!(t.get(h1), None);
}
