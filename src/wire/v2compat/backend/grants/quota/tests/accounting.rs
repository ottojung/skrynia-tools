use super::{Usage, fixture};
#[test]
fn metadata_descendants_and_overlap_are_exact() {
    let (authority, mut objects) = fixture();
    let root = authority.usage("root", &objects, &[]).unwrap();
    assert_eq!(
        root,
        Usage {
            bytes: 2048 * 2 + 128 + 100 + 512,
            objects: 1
        }
    );
    let child = authority.usage("child", &objects, &[]).unwrap();
    assert_eq!(
        child,
        Usage {
            bytes: 2048 + 128,
            objects: 0
        }
    );
    objects.insert("team/file".into(), 3);
    let child = authority.usage("child", &objects, &[]).unwrap();
    assert_eq!(
        child,
        Usage {
            bytes: 2048 + 128 + 3 + 512,
            objects: 1
        }
    );
}
#[test]
fn sponsored_subscription_metadata_counts_in_ancestors() {
    let (authority, objects) = fixture();
    let subs = [("child".into(), 2)];
    let child = authority.usage("child", &objects, &subs).unwrap();
    assert_eq!(child.bytes, 2048 + 128 + 8192);
    assert_eq!(child.objects, 2);
    let root = authority.usage("root", &objects, &subs).unwrap();
    assert_eq!(root.objects, 3);
}
